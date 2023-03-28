use bevy::prelude::Resource;
use rand::rngs::StdRng;

// Store number generator as a resource so tests can reuse seed.
#[derive(Resource)]
pub struct WorldRng {
    pub rng: StdRng,
}
