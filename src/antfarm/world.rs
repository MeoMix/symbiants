// TODO: is using super like this bad practice?
use super::ant;
use super::point;
use bevy::prelude::Resource;
use rand::Rng;
use uuid::Uuid;

// TODO: I get the impression this is the wrong way to do this given ECS
pub struct ElementData {
    pub id: Uuid,
    pub location: point::Point,
    pub element: Element,
    pub active: bool,
}

pub enum Element {
    Air,
    Dirt,
    Sand,
}

// TODO: I am not confident this is the right way to represent world state, but following example here: https://github.com/bevyengine/bevy/blob/latest/examples/ecs/ecs_guide.rs
// I feel like this is incorrect because I have nesting going on (ants etc)
#[derive(Resource, Default)]
pub struct World {
    width: i32,
    height: i32,
    elements: Vec<ElementData>,
    surface_level: i32,
    ants: Vec<ant::Ant>,
}

impl World {
    pub fn new(width: i32, height: i32, dirt_percent: f32, ant_count: i32) -> Self {
        let surface_level = (height as f32 - (height as f32 * dirt_percent)).floor() as i32;

        let elements = (0..height)
            .flat_map(|row_index| {
                (0..width).map(move |column_index| {
                    let element = if row_index <= surface_level {
                        Element::Air
                    } else {
                        Element::Dirt
                    };
                    let active = false;

                    ElementData {
                        location: point::Point {
                            x: column_index,
                            y: row_index,
                        },
                        element,
                        active,
                        id: Uuid::new_v4(),
                    }
                })
            })
            .collect::<Vec<_>>();

        let ants = (0..ant_count)
            .map(|_| {
                // Put the ant at a random location along the x-axis that fits within the bounds of the world.
                // TODO: technically old code was .round() and now it's just floored implicitly
                let x = rand::thread_rng().gen_range(0..1000) % width;
                // Put the ant on the dirt.
                let y = surface_level;
                // Randomly position ant facing left or right.
                let facing = if rand::thread_rng().gen_range(0..10) < 5 {
                    ant::Facing::Left
                } else {
                    ant::Facing::Right
                };
                let name = "Test".to_string();

                ant::Ant::new(
                    x,
                    y,
                    ant::Behavior::Wandering,
                    facing,
                    ant::Angle::Zero,
                    name,
                )
            })
            .collect::<Vec<_>>();

        Self {
            width,
            height,
            elements,
            surface_level,
            ants,
        }
    }
}
