use bevy::prelude::*;
use chrono::{LocalResult, TimeZone, Utc, DateTime};
use std::time::Duration;

pub const DEFAULT_TICK_RATE: f32 = 10.0 / 60.0;
pub const FAST_FORWARD_TICK_RATE: f32 = 0.001 / 60.0;
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

#[derive(Resource, Default)]
pub struct IsFastForwarding(pub bool);

#[derive(Resource, Default)]
pub struct PendingTicks(pub isize);

impl PendingTicks {
    pub fn as_minutes(&self) -> f32 {
        (self.0 as f32) / (60.0 * DEFAULT_TICK_RATE) / 60.0
    }
}

// When the app first loads - determine how long user has been away and fast-forward time accordingly.
pub fn setup_fast_forward_time(game_time: Res<GameTime>, mut fixed_time: ResMut<FixedTime>) {
    let delta_seconds = Utc::now()
        .signed_duration_since(game_time.as_datetime())
        .num_seconds();

    // Only one day of time is allowed to be fast-forwarded. If the user skips more time than this - they lose out on simulation.
    let elapsed_seconds = std::cmp::min(delta_seconds, SECONDS_PER_DAY);

    fixed_time.tick(Duration::from_secs(elapsed_seconds as u64));
}

// Determine how many simulation ticks should occur based off of default tick rate.
// Determine the conversation rate between simulation ticks and fast forward time.
// Update tick rate to fast forward time.
// When sufficient number of ticks have occurred, revert back to default tick rate.
pub fn play_time(
    mut fixed_time: ResMut<FixedTime>,
    mut is_fast_forwarding: ResMut<IsFastForwarding>,
    mut pending_ticks: ResMut<PendingTicks>,
) {
    if pending_ticks.0 > 0 {
        // Continue fast-forwarding
        pending_ticks.0 -= 1;
    } else if is_fast_forwarding.0 {
        // Stop fast-forwarding
        fixed_time.period = Duration::from_secs_f32(DEFAULT_TICK_RATE);
        is_fast_forwarding.0 = false;
    } else {
        // Start fast-forwarding
        // If fixed_time.accumulated() is large (more than a few ms) then it's assumed that either the app was closed
        // or the tab the app runs on was hidden. In either case, need to fast-forward time when app is active again.
        let accumulated_time = fixed_time.accumulated();

        if accumulated_time.as_secs() > 1 {
            // Reset fixed_time to zero and run the main Update schedule. This prevents the UI from becoming unresponsive for large time values.
            // The UI becomes unresponsive because the FixedUpdate schedule, when behind, will run in a loop without yielding until it catches up.
            fixed_time.period = accumulated_time;
            let _ = fixed_time.expend();
            fixed_time.period = Duration::from_secs_f32(FAST_FORWARD_TICK_RATE);

            is_fast_forwarding.0 = true;
            pending_ticks.0 = ((1.0 / DEFAULT_TICK_RATE) * accumulated_time.as_secs() as f32) as isize;
        }
    }
}

pub fn update_game_time(mut game_time: ResMut<GameTime>) {
    // Each tick we need to update GameTime by an amount of real-world time equal to the tick rate.
    game_time.0 += (DEFAULT_TICK_RATE * 1000.0) as i64;
}
