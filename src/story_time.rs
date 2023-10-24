use bevy::prelude::*;
use bevy_save::SaveableRegistry;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use js_sys::Date;
use std::time::Duration;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::common::register;

pub const DEFAULT_TICKS_PER_SECOND: isize = 10;
pub const MAX_USER_TICKS_PER_SECOND: isize = 1_500;
pub const MAX_SYSTEM_TICKS_PER_SECOND: isize = 50_000;
pub const SECONDS_PER_HOUR: isize = 3_600;
pub const SECONDS_PER_DAY: isize = 86_400;

#[wasm_bindgen]
pub fn seconds_into_day() -> f64 {
    let date = Date::new_0();

    let hours = date.get_hours();
    let minutes = date.get_minutes();
    let seconds = date.get_seconds();

    let total_seconds = (hours as f64 * 3600.0) + (minutes as f64 * 60.0) + seconds as f64;

    total_seconds
}

// NOTE: `bevy_reflect` doesn't support DateTime<Utc> without manually implement Reflect (which is hard)
// So, use a timestamp instead and convert to DateTime<Utc> when needed.
// Also, Time/Instant/Duration aren't serializable.
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct StoryRealWorldTime(pub i64);

impl StoryRealWorldTime {
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

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct StoryElapsedTicks {
    value: i64,
    pub is_real_time: bool,
    real_time_offset: isize,
    demo_time_offset: isize,
}

impl Default for StoryElapsedTicks {
    fn default() -> Self {
        StoryElapsedTicks {
            value: 0,
            is_real_time: false,
            // Real time wants to know how many seconds into the real world day have passed when the story started.
            real_time_offset: seconds_into_day() as isize,
            // Offset by an assumption that, for Sandbox Mode, the story starts at 8AM the first day not at Midnight.
            demo_time_offset: 8 * SECONDS_PER_HOUR,
        }
    }
}

#[derive(Default)]
pub struct TimeInfo {
    pub days: isize,
    pub hours: isize,
    pub minutes: isize,
}

impl StoryElapsedTicks {
    // TODO: Could use an enum or something
    pub fn is_nighttime(&self) -> bool {
        let time_info = self.as_time_info();

        time_info.hours < 6 || time_info.hours >= 22
    }

    fn offset(&self) -> isize {
        if self.is_real_time {
            self.real_time_offset
        } else {
            self.demo_time_offset
        }
    }

    pub fn as_time_info(&self) -> TimeInfo {
        let seconds_total =
            self.value as f32 / DEFAULT_TICKS_PER_SECOND as f32 + self.offset() as f32;
        let days = (seconds_total / SECONDS_PER_DAY as f32).floor() as isize;

        // Calculate hours and minutes
        let hours_total = (seconds_total % SECONDS_PER_DAY as f32) / SECONDS_PER_HOUR as f32;
        let hours = hours_total.floor() as isize;
        let minutes = ((hours_total - hours as f32) * 60.0).floor() as isize;

        TimeInfo {
            days,
            hours,
            minutes,
        }
    }
}

/// Store TicksPerSecond separately from FixedTime because when we're fast forwarding time we won't update TicksPerSecond.
/// This enables resetting back to a user-defined ticks-per-second (adjusted via UI) rather than the default ticks-per-second.
#[derive(Resource)]
pub struct TicksPerSecond(pub isize);

impl Default for TicksPerSecond {
    fn default() -> Self {
        TicksPerSecond(DEFAULT_TICKS_PER_SECOND)
    }
}

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
    register::<StoryRealWorldTime>(&app_type_registry, &mut saveable_registry);
    register::<StoryElapsedTicks>(&app_type_registry, &mut saveable_registry);
}

// TODO: awkward timing for this - need to have resources available before calling load (why?)
pub fn pre_setup_story_time(mut commands: Commands) {
    commands.init_resource::<StoryRealWorldTime>();
    commands.init_resource::<StoryElapsedTicks>();
    commands.init_resource::<FastForwardingStateInfo>();
    commands.init_resource::<TicksPerSecond>();
    commands.insert_resource(FixedTime::new_from_secs(
        1.0 / DEFAULT_TICKS_PER_SECOND as f32,
    ));
}

/// On startup, determine how much real-world time has passed since the last time the app ran,
/// record this value into FixedTime, and anticipate further processing.
/// Write to FixedTime because, in another scenario where the app is paused not closed, FixedTime
/// will be used by Bevy internally to track how de-synced the FixedUpdate schedule is from real-world time.
pub fn setup_story_time(
    mut story_real_world_time: ResMut<StoryRealWorldTime>,
    mut fixed_time: ResMut<FixedTime>,
    mut next_story_playback_state: ResMut<NextState<StoryPlaybackState>>,
    mut story_elapsed_ticks: ResMut<StoryElapsedTicks>,
    ticks_per_second: Res<TicksPerSecond>,
) {
    // Setup story_real_world_time here, rather than as a Default, so that delta_seconds doesn't grow while idling in main menu
    if story_real_world_time.0 == 0 {
        story_real_world_time.0 = Utc::now().timestamp_millis();
    } else {
        let mut delta_seconds = Utc::now()
            .signed_duration_since(story_real_world_time.as_datetime())
            .num_seconds();

        let seconds_past_max = delta_seconds as isize - SECONDS_PER_DAY;

        if seconds_past_max > 0 {
            // Increment elapsed ticks by the amount not being simulated to keep game clock synced with real-world clock
            if story_elapsed_ticks.is_real_time {
                let missed_ticks = seconds_past_max * ticks_per_second.0;
                story_elapsed_ticks.value += missed_ticks as i64;
            }

            // Enforce a max of 24 hours because it's impossible to quickly simulate an arbitrary amount of time missed.
            delta_seconds = SECONDS_PER_DAY as i64;
        }

        // If we are tracking real world time then determine how many ticks the time past 24hrs represents
        // add that to the "elapsed ticks" tracker so that real-world time in game advances.
        fixed_time.tick(Duration::from_secs(delta_seconds as u64));
    }

    next_story_playback_state.set(StoryPlaybackState::Playing);
}

pub fn teardown_story_time(mut commands: Commands) {
    commands.remove_resource::<StoryRealWorldTime>();
    commands.remove_resource::<StoryElapsedTicks>();
    commands.remove_resource::<FastForwardingStateInfo>();
    commands.remove_resource::<TicksPerSecond>();

    // `FixedTime` is managed by Bevy and can't be removed with panic occurring.
    commands.insert_resource(FixedTime::default());
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
        if *story_playback_state == StoryPlaybackState::FastForwarding {
            fixed_time.period = Duration::from_secs_f32(1.0 / (ticks_per_second.0 as f32));

            next_story_playback_state.set(StoryPlaybackState::Playing);
            fast_forward_state_info.initial_pending_ticks = 0;
        } else {
            let accumulated_time = fixed_time.accumulated();

            if accumulated_time.as_secs() > 1 {
                // Reset fixed_time to zero and run the main Update schedule. This prevents the UI from becoming unresponsive for large time values.
                // The UI becomes unresponsive because the FixedUpdate schedule, when behind, will run in a loop without yielding until it catches up.
                fixed_time.period = accumulated_time;
                let _ = fixed_time.expend();
                fixed_time.period =
                    Duration::from_secs_f32(1.0 / (MAX_SYSTEM_TICKS_PER_SECOND as f32));
                // Rely on FastForwarding state, rather than updating TicksPerSecond, so when exiting FastForwarding it's possible to restore to user-defined TicksPerSecond.

                // There's nothing to fast forward through if the simulation was paused while tab wasn't focused.
                if *story_playback_state != StoryPlaybackState::Paused {
                    next_story_playback_state.set(StoryPlaybackState::FastForwarding);

                    let ticks = (ticks_per_second.0 as u64 * accumulated_time.as_secs()) as isize;
                    fast_forward_state_info.pending_ticks = ticks;
                    fast_forward_state_info.initial_pending_ticks = ticks;
                }
            }
        }
    } else {
        fast_forward_state_info.pending_ticks -= 1;
    }
}

// Track real-world time to be able to derive how much time elapsed while app was closed.
// Keep this updated, rather than capture JIT, because running Bevy systems JIT as app closing isn't viable.
pub fn update_story_real_world_time(mut story_real_world_time: ResMut<StoryRealWorldTime>) {
    story_real_world_time.0 = Utc::now().timestamp_millis();
}

// Track in-game time by counting elapsed ticks.
pub fn update_story_elapsed_ticks(mut story_elapsed_ticks: ResMut<StoryElapsedTicks>) {
    story_elapsed_ticks.value += 1;
}

pub fn update_time_scale(
    mut fixed_time: ResMut<FixedTime>,
    ticks_per_second: Res<TicksPerSecond>,
    next_story_playback_state: Res<NextState<StoryPlaybackState>>,
) {
    // Don't unintentionally overwrite fixed_time.period when shifting into FastForwarding.
    if next_story_playback_state.0 == Some(StoryPlaybackState::FastForwarding) {
        return;
    }

    fixed_time.period = Duration::from_secs_f32(1.0 / (ticks_per_second.0 as f32));
}
