use bevy::prelude::*;
use chrono::Utc;
use gloo_storage::{LocalStorage, Storage};
use std::time::Duration;

use crate::map::{WorldSaveState, LOCAL_STORAGE_KEY};

// Normally run at 6fps but fast forward at 6000fps
pub const DEFAULT_TICK_RATE: f32 = 10.0 / 60.0;
pub const FAST_FORWARD_TICK_RATE: f32 = 0.001 / 60.0;
pub const SECONDS_IN_DAY: i64 = 86_400;

#[derive(Resource)]
pub struct IsFastForwarding(pub bool);

#[derive(Resource)]
pub struct PendingTicks(pub isize);

// Determine how many simulation ticks should occur based off of default tick rate.
// Determine the conversation rate between simulation ticks and fast forward time.
// Update tick rate to fast forward time.
// When sufficient number of ticks have occurred, revert back to default tick rate.
pub fn setup_fast_forward_time_system(
    mut fixed_time: ResMut<FixedTime>,
    mut is_fast_forwarding: ResMut<IsFastForwarding>,
    mut pending_ticks: ResMut<PendingTicks>,
) {
    if let Ok(saved_state) = LocalStorage::get::<WorldSaveState>(LOCAL_STORAGE_KEY) {
        let delta_seconds = Utc::now()
            .signed_duration_since(saved_state.time_stamp)
            .num_seconds();

        // Only one day of time is allowed to be fast-forwarded. If the user skips more time than this - they lose out on simulation.
        let elapsed_seconds = std::cmp::min(delta_seconds, SECONDS_IN_DAY);

        if elapsed_seconds > 0 {
            fixed_time.period = Duration::from_secs_f32(FAST_FORWARD_TICK_RATE);
            is_fast_forwarding.0 = true;
            pending_ticks.0 = ((60.0 * DEFAULT_TICK_RATE) * elapsed_seconds as f32) as isize;
        }
    }
}

pub fn play_time_system(
    mut fixed_time: ResMut<FixedTime>,
    mut is_fast_forwarding: ResMut<IsFastForwarding>,
    mut pending_ticks: ResMut<PendingTicks>,
) {
    // If fast forwarding and done fast forwarding then stop fast forwarding and play at normal tick rate
    if pending_ticks.0 > 0 {
        pending_ticks.0 -= 1;
    } else if is_fast_forwarding.0 {
        fixed_time.period = Duration::from_secs_f32(DEFAULT_TICK_RATE);
        is_fast_forwarding.0 = false;
    }
}
