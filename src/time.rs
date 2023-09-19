use bevy::prelude::*;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use std::time::Duration;

use crate::common::register;

pub const DEFAULT_SECONDS_PER_TICK: f32 = 10.0 / 60.0;
pub const FASTFORWARD_SECONDS_PER_TICK: f32 = 0.001 / 60.0;
pub const SECONDS_PER_HOUR: i64 = 3600;
pub const SECONDS_PER_DAY: i64 = 86_400;

// NOTE: `bevy_reflect` doesn't support DateTime<Utc> without manually implement Reflect (which is hard)
// So, use a timestamp instead and convert to DateTime<Utc> when needed.
// Also, Time/Instant/Duration aren't serializable.
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct GameTime(pub i64);

impl Default for GameTime {
    fn default() -> Self {
        GameTime(Utc::now().timestamp_millis())
    }
}

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

// TODO: IsFastForwarding should be expressed as a derived property of PendingTicks.0 > 0
#[derive(Resource, Default)]
pub struct IsFastForwarding(pub bool);

#[derive(Resource, Default)]
pub struct PendingTicks(pub isize);

impl PendingTicks {
    pub fn as_minutes(&self) -> f32 {
        (self.0 as f32) / (60.0 * DEFAULT_SECONDS_PER_TICK) / 60.0
    }
}

pub fn initialize_game_time(world: &mut World) {
    register::<GameTime>(world);

    world.init_resource::<GameTime>();
    world.init_resource::<IsFastForwarding>();
    world.init_resource::<PendingTicks>();
}

pub fn deinitialize_game_time(world: &mut World) {
    world.remove_resource::<GameTime>();
    world.remove_resource::<IsFastForwarding>();
    world.remove_resource::<PendingTicks>();
}

/// On startup, determine how much real-world time has passed since the last time the app ran,
/// record this value into FixedTime, and anticipate further processing.
/// Write to FixedTime because, in another scenario where the app is paused not closed, FixedTime
/// will be used by Bevy internally to track how de-synced the FixedUpdate schedule is from real-world time.
pub fn setup_game_time(game_time: Res<GameTime>, mut fixed_time: ResMut<FixedTime>) {
    let delta_seconds = Utc::now()
        .signed_duration_since(game_time.as_datetime())
        .num_seconds();

    // FIXME: I doubt this enforcement works with the new GameTime approach because expectation is to sync within 1s of real-world time.
    // Limit fast-forward to one day of time
    let elapsed_seconds = std::cmp::min(delta_seconds, SECONDS_PER_DAY);

    fixed_time.tick(Duration::from_secs(elapsed_seconds as u64));
}

/// Control whether the app runs at the default or fast tick rate.
/// Checks if FixedTime is showing a time de-sync and adjusts tick rate to compensate.
/// Once compensated tick rate has been processed then reset back to default tick rate.
pub fn set_rate_of_time(
    mut fixed_time: ResMut<FixedTime>,
    mut is_fast_forwarding: ResMut<IsFastForwarding>,
    mut pending_ticks: ResMut<PendingTicks>,
) {
    if pending_ticks.0 == 0 {
        if !is_fast_forwarding.0 {
            let accumulated_time = fixed_time.accumulated();

            if accumulated_time.as_secs() > 1 {
                // Reset fixed_time to zero and run the main Update schedule. This prevents the UI from becoming unresponsive for large time values.
                // The UI becomes unresponsive because the FixedUpdate schedule, when behind, will run in a loop without yielding until it catches up.
                fixed_time.period = accumulated_time;
                let _ = fixed_time.expend();
                fixed_time.period = Duration::from_secs_f32(FASTFORWARD_SECONDS_PER_TICK);

                is_fast_forwarding.0 = true;
                pending_ticks.0 =
                    ((1.0 / DEFAULT_SECONDS_PER_TICK) * accumulated_time.as_secs() as f32) as isize;
            }
        } else {
            fixed_time.period = Duration::from_secs_f32(DEFAULT_SECONDS_PER_TICK);
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
pub fn update_game_time(mut game_time: ResMut<GameTime>) {
    game_time.0 += (DEFAULT_SECONDS_PER_TICK * 1000.0) as i64;
}
