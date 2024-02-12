use super::Initiative;
use crate::common::Zone;
use bevy::prelude::*;
use bevy_turborand::GlobalRng;

// Each ant maintains an internal timer that determines when it will act next.
// This adds a little realism by varying when movements occur and allows for flexibility
// in the simulation run speed.
pub fn ants_initiative<Z: Zone>(
    mut alive_ants_query: Query<&mut Initiative, With<Z>>,
    mut rng: ResMut<GlobalRng>,
) {
    for mut initiative in alive_ants_query.iter_mut() {
        if initiative.timer > 0 {
            initiative.timer -= 1;

            if initiative.timer == 0 {
                initiative.has_action = true;
                initiative.has_movement = true;
            }

            continue;
        }

        *initiative = Initiative::new(&mut rng.reborrow());
    }
}
