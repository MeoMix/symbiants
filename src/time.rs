use bevy::prelude::*;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use std::time::Duration;

use crate::common::register;

pub const DEFAULT_TICKS_PER_SECOND: f32 = 6.0;
pub const FASTFORWARD_TICKS_PER_SECOND: f32 = 6000.0;
pub const SECONDS_PER_HOUR: i64 = 3600;
pub const SECONDS_PER_DAY: i64 = 86_400;

// NOTE: `bevy_reflect` doesn't support DateTime<Utc> without manually implement Reflect (which is hard)
// So, use a timestamp instead and convert to DateTime<Utc> when needed.
// Also, Time/Instant/Duration aren't serializable.
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct GameTime(pub i64);

impl GameTime {
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
/// This allows us to reset back to a user-defined ticks-per-second (adjusted via UI) rather than the default ticks-per-second.
// TODO: probably shouldn't be an f32 (integer) and should maybe be combined with some of these other resources into a single time management resource
#[derive(Resource, Default)]
pub struct TicksPerSecond(pub f32);

// TODO: IsFastForwarding should be expressed as a derived property of PendingTicks.0 > 0
#[derive(Resource, Default)]
pub struct IsFastForwarding(pub bool);

#[derive(Resource, Default)]
pub struct PendingTicks(pub isize);

impl PendingTicks {
    pub fn as_minutes(&self, ticks_per_second: f32) -> f32 {
        (self.0 as f32) / (60.0 / ticks_per_second * 60.0)
    }
}

pub fn initialize_game_time(world: &mut World) {
    register::<GameTime>(world);

    world.init_resource::<GameTime>();
    world.init_resource::<IsFastForwarding>();
    world.init_resource::<PendingTicks>();

    // Control the speed of the simulation by defining how many simulation ticks occur per second.
    world.insert_resource(FixedTime::new_from_secs(1.0 / DEFAULT_TICKS_PER_SECOND));
    world.insert_resource(TicksPerSecond(DEFAULT_TICKS_PER_SECOND));
}

pub fn deinitialize_game_time(world: &mut World) {
    world.remove_resource::<GameTime>();
    world.remove_resource::<IsFastForwarding>();
    world.remove_resource::<PendingTicks>();

    // HACK: This is resetting FixedTime to default, can't remove it entirely or program will crash (FIX?)
    world.resource_mut::<FixedTime>().period =
        Duration::from_secs_f32(1.0 / DEFAULT_TICKS_PER_SECOND);
    world.resource_mut::<TicksPerSecond>().0 = DEFAULT_TICKS_PER_SECOND;
}

/// On startup, determine how much real-world time has passed since the last time the app ran,
/// record this value into FixedTime, and anticipate further processing.
/// Write to FixedTime because, in another scenario where the app is paused not closed, FixedTime
/// will be used by Bevy internally to track how de-synced the FixedUpdate schedule is from real-world time.
pub fn setup_game_time(mut game_time: ResMut<GameTime>, mut fixed_time: ResMut<FixedTime>) {
    // Initialize game_time here, rather than as a Default, so that delta_seconds doesn't grow while idling in main menu
    if game_time.0 == 0 {
        game_time.0 = Utc::now().timestamp_millis();
    } else {
        let delta_seconds = Utc::now()
            .signed_duration_since(game_time.as_datetime())
            .num_seconds();

        fixed_time.tick(Duration::from_secs(delta_seconds as u64));
    }
}

/// Control whether the app runs at the default or fast tick rate.
/// Checks if FixedTime is showing a time de-sync and adjusts tick rate to compensate.
/// Once compensated tick rate has been processed then reset back to default tick rate.
pub fn set_rate_of_time(
    mut fixed_time: ResMut<FixedTime>,
    mut is_fast_forwarding: ResMut<IsFastForwarding>,
    mut pending_ticks: ResMut<PendingTicks>,
    ticks_per_second: Res<TicksPerSecond>,
) {
    if pending_ticks.0 == 0 {
        if !is_fast_forwarding.0 {
            let accumulated_time = fixed_time.accumulated();

            if accumulated_time.as_secs() > 1 {
                // Reset fixed_time to zero and run the main Update schedule. This prevents the UI from becoming unresponsive for large time values.
                // The UI becomes unresponsive because the FixedUpdate schedule, when behind, will run in a loop without yielding until it catches up.
                fixed_time.period = accumulated_time;
                let _ = fixed_time.expend();
                fixed_time.period = Duration::from_secs_f32(1.0 / FASTFORWARD_TICKS_PER_SECOND);
                // NOTE: intentionally do not update TicksPerSecond.

                is_fast_forwarding.0 = true;

                pending_ticks.0 = (ticks_per_second.0 * accumulated_time.as_secs() as f32) as isize;
            }
        } else {
            fixed_time.period = Duration::from_secs_f32(1.0 / ticks_per_second.0);
            is_fast_forwarding.0 = false;
        }
    } else {
        pending_ticks.0 -= 1;
    }
}

/// Increment GameTime by the default tick rate.
/// This is used to track how synchronized GameTime is with real-world time.
/// If app is fast-forwarding time then this system will be called more frequently and will
/// reduce the delta difference between game time and real-world time.
pub fn update_game_time(mut game_time: ResMut<GameTime>, ticks_per_second: Res<TicksPerSecond>) {
    game_time.0 += (1000.0 / ticks_per_second.0) as i64;
}
