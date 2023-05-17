use bevy::prelude::*;
use chrono::Utc;
use gloo_storage::{LocalStorage, Storage};
use std::time::Duration;

use crate::map::{WorldSaveState, LOCAL_STORAGE_KEY};

// Normally run at 6fps but fast forward at 600fps
pub const DEFAULT_TICK_RATE: f32 = 10.0 / 60.0;
pub const FAST_FORWARD_TICK_RATE: f32 = 0.01 / 60.0;

#[derive(Resource)]
pub struct IsFastForwarding(pub bool);

// Determine how many simulation ticks should occur based off of default tick rate.
// Determine the conversation rate between simulation ticks and fast forward time.
// Update tick rate to fast forward time.
// When sufficient number of ticks have occurred, revert back to default tick rate.
pub fn setup_fast_forward_time_system(
    mut fixed_time: ResMut<FixedTime>,
    mut is_fast_forwarding: ResMut<IsFastForwarding>,
) {
    if let Ok(saved_state) = LocalStorage::get::<WorldSaveState>(LOCAL_STORAGE_KEY) {
        let delta_seconds = Utc::now()
            .signed_duration_since(saved_state.time_stamp)
            .num_seconds();

        if delta_seconds > 0 {
            // If delta is 10 seconds then at normal tick rate of 6 ticks per second, we need to step forward 60 ticks of time.
            // To fast forward 60 ticks of time at 60 ticks per second we need to step forward 60/60 = 1 second.
            let missed_ticks = (60.0 * DEFAULT_TICK_RATE) * delta_seconds as f32;

            let fast_forward_delta_seconds = missed_ticks / (1.0 / FAST_FORWARD_TICK_RATE);

            fixed_time.period = Duration::from_secs_f32(FAST_FORWARD_TICK_RATE);
            fixed_time.tick(Duration::new(fast_forward_delta_seconds as u64, 0));
            is_fast_forwarding.0 = true;
        }
    }
}

pub fn play_time_system(
    mut fixed_time: ResMut<FixedTime>,
    mut is_fast_forwarding: ResMut<IsFastForwarding>,
) {
    // If fast forwarding and done fast forwarding then stop fast forwarding and play at normal tick rate
    if is_fast_forwarding.0
        && fixed_time
            .accumulated()
            .checked_sub(fixed_time.period)
            .is_none()
    {
        fixed_time.period = Duration::from_secs_f32(DEFAULT_TICK_RATE);
        is_fast_forwarding.0 = false;
    }
}
