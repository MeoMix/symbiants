use bevy::prelude::*;
use bevy_save::SaveableRegistry;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use std::time::Duration;

use crate::common::register;

pub const DEFAULT_TICKS_PER_SECOND: isize = 6;
pub const MAX_USER_TICKS_PER_SECOND: isize = 1_200;
pub const MAX_SYSTEM_TICKS_PER_SECOND: isize = 12_000;
pub const SECONDS_PER_HOUR: isize = 3_600;
pub const SECONDS_PER_DAY: isize = 86_400;

// NOTE: `bevy_reflect` doesn't support DateTime<Utc> without manually implement Reflect (which is hard)
// So, use a timestamp instead and convert to DateTime<Utc> when needed.
// Also, Time/Instant/Duration aren't serializable.
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct StoryTime(pub i64);

impl StoryTime {
    pub fn as_datetime(&self) -> DateTime<Utc> {
        match Utc.timestamp_millis_opt(self.0) {
            LocalResult::Single(datetime) => datetime,
            LocalResult::Ambiguous(a, b) => {
                panic!("Ambiguous DateTime<Utc> values: {} and {}", a, b);
            }
            LocalResult::None => {
                panic!("Invalid timestamp");
            }
        }
    }
}

/// Store TicksPerSecond separately from FixedTime because when we're fast forwarding time we won't update TicksPerSecond.
/// This enables resetting back to a user-defined ticks-per-second (adjusted via UI) rather than the default ticks-per-second.
#[derive(Resource, Default)]
pub struct TicksPerSecond(pub isize);

#[derive(Resource, Default)]
pub struct FastForwardingStateInfo {
    pub initial_pending_ticks: isize,
    pub pending_ticks: isize,
}

#[derive(States, Default, Hash, Clone, Copy, Eq, PartialEq, Debug)]
pub enum StoryPlaybackState {
    #[default]
    Stopped,
    Paused,
    Playing,
    FastForwarding,
}

pub fn register_story_time(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
) {
    register::<StoryTime>(&app_type_registry, &mut saveable_registry);
}

// TODO: awkward timing for this - need to have resources available before calling try_load_from_save
pub fn pre_setup_story_time(mut commands: Commands) {
    commands.init_resource::<StoryTime>();
    commands.init_resource::<FastForwardingStateInfo>();

    // Control the speed of the simulation by defining how many simulation ticks occur per second.
    commands.insert_resource(FixedTime::new_from_secs(1.0 / DEFAULT_TICKS_PER_SECOND as f32));
    commands.insert_resource(TicksPerSecond(DEFAULT_TICKS_PER_SECOND));
}

/// On startup, determine how much real-world time has passed since the last time the app ran,
/// record this value into FixedTime, and anticipate further processing.
/// Write to FixedTime because, in another scenario where the app is paused not closed, FixedTime
/// will be used by Bevy internally to track how de-synced the FixedUpdate schedule is from real-world time.
pub fn setup_story_time(
    mut story_time: ResMut<StoryTime>,
    mut fixed_time: ResMut<FixedTime>,
    mut next_story_playback_state: ResMut<NextState<StoryPlaybackState>>,
) {
    // Setup story_time here, rather than as a Default, so that delta_seconds doesn't grow while idling in main menu
    if story_time.0 == 0 {
        story_time.0 = Utc::now().timestamp_millis();
    } else {
        let delta_seconds = Utc::now()
            .signed_duration_since(story_time.as_datetime())
            .num_seconds();

        fixed_time.tick(Duration::from_secs(delta_seconds as u64));
    }

    next_story_playback_state.set(StoryPlaybackState::Playing);
}

pub fn teardown_story_time(
    mut commands: Commands,
    mut fixed_time: ResMut<FixedTime>,
    mut ticks_per_second: ResMut<TicksPerSecond>,
) {
    commands.remove_resource::<StoryTime>();
    commands.remove_resource::<FastForwardingStateInfo>();

    // HACK: This is resetting FixedTime to default, can't remove it entirely or program will crash (FIX?)
    fixed_time.period = Duration::from_secs_f32(1.0 / DEFAULT_TICKS_PER_SECOND as f32);
    ticks_per_second.0 = DEFAULT_TICKS_PER_SECOND;
}

/// Control whether the app runs at the default or fast tick rate.
/// Checks if FixedTime is showing a time de-sync and adjusts tick rate to compensate.
/// Once compensated tick rate has been processed then reset back to default tick rate.
pub fn set_rate_of_time(
    mut fixed_time: ResMut<FixedTime>,
    mut fast_forward_state_info: ResMut<FastForwardingStateInfo>,
    ticks_per_second: Res<TicksPerSecond>,
    story_playback_state: Res<State<StoryPlaybackState>>,
    mut next_story_playback_state: ResMut<NextState<StoryPlaybackState>>,
) {
    if fast_forward_state_info.pending_ticks == 0 {
        if story_playback_state.get() == &StoryPlaybackState::FastForwarding {
            fixed_time.period = Duration::from_secs_f32(1.0 / ticks_per_second.0 as f32);

            next_story_playback_state.set(StoryPlaybackState::Playing);
            fast_forward_state_info.initial_pending_ticks = 0;
        } else {
            let accumulated_time = fixed_time.accumulated();

            if accumulated_time.as_secs() > 1 {
                // Reset fixed_time to zero and run the main Update schedule. This prevents the UI from becoming unresponsive for large time values.
                // The UI becomes unresponsive because the FixedUpdate schedule, when behind, will run in a loop without yielding until it catches up.
                fixed_time.period = accumulated_time;
                let _ = fixed_time.expend();
                fixed_time.period = Duration::from_secs_f32(1.0 / MAX_SYSTEM_TICKS_PER_SECOND as f32);
                // Rely on FastForwarding state, rather than updating TicksPerSecond, so when exiting FastForwarding it's possible to restore to user-defined TicksPerSecond.

                next_story_playback_state.set(StoryPlaybackState::FastForwarding);

                let ticks = (ticks_per_second.0 as u64 * accumulated_time.as_secs()) as isize;
                fast_forward_state_info.pending_ticks = ticks;
                fast_forward_state_info.initial_pending_ticks = ticks;
            }
        }
    } else {
        fast_forward_state_info.pending_ticks -= 1;
    }
}

/// Increment StoryTime by the default tick rate.
/// This is used to track how synchronized StoryTime is with real-world time.
/// If app is fast-forwarding time then this system will be called more frequently and will
/// reduce the delta difference between game time and real-world time.
pub fn update_story_time(mut story_time: ResMut<StoryTime>, ticks_per_second: Res<TicksPerSecond>) {
    story_time.0 += (1000.0 / ticks_per_second.0 as f32) as i64;
}

pub fn update_time_scale(
    mut fixed_time: ResMut<FixedTime>,
    ticks_per_second: Res<TicksPerSecond>,
    story_playback_state: Res<State<StoryPlaybackState>>,
) {
    if story_playback_state.get() == &StoryPlaybackState::FastForwarding {
        return;
    }

    fixed_time.period = Duration::from_secs_f32(1.0 / ticks_per_second.0 as f32);
}
