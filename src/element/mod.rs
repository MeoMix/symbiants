use crate::gravity::Unstable;

use super::map::{Position, WorldMap};
use bevy::{ecs::system::Command, prelude::*, sprite::Anchor};
use serde::{Deserialize, Serialize};

// This is what's persisted as JSON.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ElementSaveState {
    pub element: Element,
    pub position: Position,
}

// TODO: I am a little suspicious about using MORE marker components to indicate more clearly the exact type of an element
// rather than relying on properties of the element, but I am going with this for now for simplicity and performance.
// As an example, say a piece of sand falls down and air moves up. I want to mark elements adjacent to the air as unstable.
// I can read the position of the sand and see it has changed, but Bevy does not provide the ability to know the previous value.
// So, I need to read the position of the air because its current value is what is useful to me.
// How do I find that air? Well, I could iterate over all elements (slow), I could iterate over all not-Unstable elements (air is never unstable, but same goes for dirt and sometimes sand/food) (also slow)
// or I could tag air with a component that indicates it is air (fast).
// Another option could be to emit an event when I move the dirt saying where it moved from.
#[derive(Component)]
pub struct Air;

#[derive(Bundle)]
pub struct AirElementBundle {
    element: Element,
    position: Position,
    air: Air,
}

impl AirElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            element: Element::Air,
            air: Air,
            position,
        }
    }
}

#[derive(Bundle)]
pub struct DirtElementBundle {
    element: Element,
    position: Position,
}

impl DirtElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            element: Element::Dirt,
            position,
        }
    }
}

#[derive(Component)]
pub struct Crushable;

#[derive(Bundle)]
pub struct SandElementBundle {
    element: Element,
    position: Position,
    crushable: Crushable,
    unstable: Unstable,
}

impl SandElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            element: Element::Sand,
            position,
            crushable: Crushable,
            unstable: Unstable,
        }
    }
}

#[derive(Bundle)]
pub struct FoodElementBundle {
    element: Element,
    position: Position,
    unstable: Unstable,
}

impl FoodElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            element: Element::Food,
            position,
            unstable: Unstable,
        }
    }
}

#[derive(Component, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Element {
    Air,
    Dirt,
    Sand,
    Food,
}

pub fn is_element(
    world_map: &WorldMap,
    elements_query: &Query<&Element>,
    position: &Position,
    search_element: &Element,
) -> bool {
    world_map.get_element(*position).map_or(false, |&element| {
        elements_query
            .get(element)
            .map_or(false, |queried_element| *queried_element == *search_element)
    })
}

// Returns true if every element in `positions` matches the provided Element type.
// NOTE: This returns true if given 0 positions.
pub fn is_all_element(
    world_map: &WorldMap,
    elements_query: &Query<&Element>,
    positions: &[Position],
    search_element: &Element,
) -> bool {
    positions
        .iter()
        .all(|position| is_element(world_map, elements_query, position, search_element))
}

// Spawn interactive elements - air/dirt/sand. Air isn't visible, background is revealed in its place.
pub fn setup_elements(mut commands: Commands, world_map: Res<WorldMap>) {
    for &ElementSaveState { element, position } in world_map.initial_state().elements.iter() {
        commands.spawn_element(element, position);
    }
}

pub fn on_spawn_element(
    mut commands: Commands,
    elements: Query<(Entity, &Position, &Element), Added<Element>>,
) {
    for (entity, position, element) in &elements {
        let color = match element {
            // Air is transparent - reveals background color such as tunnel or sky
            Element::Air => Color::rgba(0.0, 0.0, 0.0, 0.0),
            Element::Dirt => Color::rgb(0.514, 0.396, 0.224),
            Element::Sand => Color::rgb(0.761, 0.698, 0.502),
            Element::Food => Color::rgb(0.388, 0.584, 0.294),
        };

        commands.entity(entity).insert(SpriteBundle {
            transform: Transform::from_translation(position.as_world_position()),
            sprite: Sprite {
                color,
                anchor: Anchor::TopLeft,
                ..default()
            },
            ..default()
        });
    }
}

pub trait ElementCommandsExt {
    fn replace_element(&mut self, position: Position, target_element: Entity, element: Element);
    fn spawn_element(&mut self, element: Element, position: Position);
    fn toggle_element_unstable(&mut self, target_element_entity: Entity, position: Position, toggle: bool);
}

impl<'w, 's> ElementCommandsExt for Commands<'w, 's> {
    fn replace_element(&mut self, position: Position, target_element: Entity, element: Element) {
        self.add(ReplaceElementCommand {
            position,
            target_element,
            element,
        })
    }

    fn spawn_element(&mut self, element: Element, position: Position) {
        self.add(SpawnElementCommand { element, position })
    }

    
    fn toggle_element_unstable(&mut self, target_element_entity: Entity, position: Position, toggle: bool) {
        self.add(MarkElementUnstableCommand { target_element_entity, position, toggle })
    }
}

struct ReplaceElementCommand {
    target_element: Entity,
    element: Element,
    position: Position,
}

impl Command for ReplaceElementCommand {
    fn apply(self, world: &mut World) {
        // The act of replacing an element with another element is delayed because spawn/despawn are queued actions which
        // apply after a system finishes running. It's possible for two writes to occur to the same location during a given
        // system run and, in this scenario, overwrites should not occur because validity checks have already been performed.
        // So, we anticipate the entity to be destroyed and confirm it still exists in the position expected. Otherwise, no-op.
        let world_map = world.resource::<WorldMap>();

        let existing_entity = match world_map.get_element(self.position) {
            Some(entity) => entity,
            None => {
                info!("No entity found at position {:?}", self.position);
                return;
            }
        };

        if *existing_entity != self.target_element {
            info!("Existing entity doesn't match the current entity.");
            return;
        }

        info!("Despawning entity {:?} at position {:?}", existing_entity, self.position);

        world.entity_mut(*existing_entity).despawn();

        let entity = spawn_element(self.element, self.position, world);
        let mut world_map = world.resource_mut::<WorldMap>();

        world_map.set_element(self.position, entity);
    }
}

struct SpawnElementCommand {
    element: Element,
    position: Position,
}

impl Command for SpawnElementCommand {
    fn apply(self, world: &mut World) {
        if let Some(existing_entity) = world.resource::<WorldMap>().get_element(self.position) {
            info!(
                "Entity {:?} already exists at position {:?}",
                existing_entity, self.position
            );
            return;
        }

        let entity = spawn_element(self.element, self.position, world);
        world
            .resource_mut::<WorldMap>()
            .set_element(self.position, entity);
    }
}

pub fn spawn_element(element: Element, position: Position, world: &mut World) -> Entity {
    match element {
        Element::Air => world.spawn(AirElementBundle::new(position)).id(),
        Element::Dirt => world.spawn(DirtElementBundle::new(position)).id(),
        Element::Sand => world.spawn(SandElementBundle::new(position)).id(),
        Element::Food => world.spawn(FoodElementBundle::new(position)).id(),
    }
}

// TODO: Make this more generic so singular operations against elements can be generally safeguarded?
struct MarkElementUnstableCommand {
    target_element_entity: Entity,
    position: Position,
    toggle: bool,
}

impl Command for MarkElementUnstableCommand {
    fn apply(self, world: &mut World) {
        let world_map = world.resource::<WorldMap>();
        let element_entity = match world_map.get_element(self.position) {
            Some(entity) => *entity,
            None => {
                info!("No entity found at position {:?}", self.position);
                return;
            }
        };

        if element_entity != self.target_element_entity {
            info!("Entity at position does not match expected entity.");
            return;
        }

        if self.toggle {
            world.entity_mut(element_entity).insert(Unstable);
        } else {
            world.entity_mut(element_entity).remove::<Unstable>();
        }
    }
}