// TODO: For now I am just ripping from Bevy's fixed_timestep, but I should revisit this and reduce complexity.
use bevy::prelude::*;
use bevy::utils::Duration;

use super::simulation::SimulationUpdate;

/// The amount of time that must pass before the fixed timestep schedule is run again.
#[derive(Resource, Debug)]
pub struct SimulationTime {
    accumulated: Duration,
    /// Defaults to 1/60th of a second.
    /// To configure this value, simply mutate or overwrite this resource.
    pub period: Duration,
}

impl SimulationTime {
    /// Creates a new [`SimulationTime`] struct with a period specified in `f32` seconds
    pub fn new_from_secs(period: f32) -> Self {
        SimulationTime {
            accumulated: Duration::ZERO,
            period: Duration::from_secs_f32(period),
        }
    }

    /// Adds the `delta_time` to the accumulated time so far.
    pub fn tick(&mut self, delta_time: Duration) {
        self.accumulated += delta_time;
    }

    /// Returns the current amount of accumulated time
    pub fn accumulated(&self) -> Duration {
        self.accumulated
    }

    /// Expends one `period` of accumulated time.
    ///
    /// [`Err(SimulationUpdateError`)] will be returned if there is
    /// not enough accumulated time to span an entire period.
    pub fn expend(&mut self) -> Result<(), SimulationUpdateError> {
        if let Some(new_value) = self.accumulated.checked_sub(self.period) {
            self.accumulated = new_value;
            Ok(())
        } else {
            Err(SimulationUpdateError::NotEnoughTime {
                accumulated: self.accumulated,
                period: self.period,
            })
        }
    }
}

impl Default for SimulationTime {
    fn default() -> Self {
        SimulationTime {
            accumulated: Duration::ZERO,
            period: Duration::from_secs_f32(1. / 60.),
        }
    }
}

/// An error returned when working with [`SimulationTime`].
#[derive(Debug)]
pub enum SimulationUpdateError {
    // #[error("At least one period worth of time must be accumulated.")]
    NotEnoughTime {
        accumulated: Duration,
        period: Duration,
    },
}

pub fn run_simulation_update_schedule(world: &mut World) {
    // Tick the time
    let delta_time = world.resource::<Time>().delta();
    let mut simulation_time = world.resource_mut::<SimulationTime>();
    simulation_time.tick(delta_time);

    // Run the schedule until we run out of accumulated time
    let _ = world.try_schedule_scope(SimulationUpdate, |world, schedule| {
        while world.resource_mut::<SimulationTime>().expend().is_ok() {
            schedule.run(world);
        }
    });
}