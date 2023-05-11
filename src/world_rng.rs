use bevy::prelude::Resource;
use rand::{
    rngs::{OsRng, StdRng},
    Rng, SeedableRng,
};

// Store number generator as a resource so tests can reuse seed.
#[derive(Resource)]
pub struct WorldRng {
    pub rng: StdRng,
}

impl Default for WorldRng {
    fn default() -> Self {
        // TODO: Not sure if StdRng is necessary. I think I read something at one point about how to get Bevy working
        // with a traditional RNG but I can't find it right now.
        Self {
            rng: StdRng::seed_from_u64(OsRng {}.gen()),
        }
    }
}
