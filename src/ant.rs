use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, ops::Add};

use crate::{
    elements::{
        is_all_element, is_element, AirElementBundle, FoodElementBundle, SandElementBundle,
    },
    gravity::loosen_neighboring_sand_and_food,
    map::{Position, WorldMap},
    world_rng::WorldRng,
};

use super::{elements::Element, settings::Settings};
use bevy::{prelude::*, sprite::Anchor};
use rand::{rngs::StdRng, Rng};

// This is what is persisted as JSON.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AntSaveState {
    pub position: Position,
    pub color: AntColor,
    pub orientation: AntOrientation,
    pub inventory: AntInventory,
    pub role: AntRole,
    pub timer: AntTimer,
    pub name: AntName,
}

#[derive(Bundle)]
struct AntBundle {
    position: Position,
    translation_offset: TranslationOffset,
    orientation: AntOrientation,
    role: AntRole,
    timer: AntTimer,
    name: AntName,
    color: AntColor,
    hunger: Hunger,
    alive: Alive,
    inventory: AntInventory,
    sprite_bundle: SpriteBundle,
}

impl AntBundle {
    pub fn new(
        position: Position,
        color: Color,
        orientation: AntOrientation,
        inventory: AntInventory,
        role: AntRole,
        name: &str,
        asset_server: &Res<AssetServer>,
        mut rng: &mut StdRng,
    ) -> Self {
        // TODO: z-index is 1.0 here because ant can get hidden behind sand otherwise. This isn't a good way of achieving this.
        // y-offset is to align ant with the ground, but then ant looks weird when rotated if x isn't adjusted.
        let translation_offset = TranslationOffset(Vec3::new(0.5, -0.5, 1.0));

        Self {
            position,
            translation_offset,
            orientation,
            inventory,
            role,
            timer: AntTimer::new(&mut rng),
            name: AntName(name.to_string()),
            color: AntColor(color),
            hunger: Hunger::default(),
            alive: Alive,
            sprite_bundle: SpriteBundle {
                texture: asset_server.load("images/ant.png"),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(ANT_SCALE, ANT_SCALE)),
                    ..default()
                },
                transform: Transform {
                    translation: position.as_world_position().add(translation_offset.0),
                    rotation: orientation.as_world_rotation(),
                    scale: orientation.as_world_scale(),
                    ..default()
                },
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
struct AntLabelBundle {
    label_bundle: Text2dBundle,
    translation_offset: TranslationOffset,
    label: Label,
}

impl AntLabelBundle {
    pub fn new(
        ant: Entity,
        position: Position,
        name: &str,
        asset_server: &Res<AssetServer>,
    ) -> Self {
        // TODO: z-index is 1.0 here because label gets hidden behind dirt/sand otherwise. This isn't a good way of achieving this.
        let translation_offset = TranslationOffset(Vec3::new(0.5, -1.5, 1.0));

        Self {
            label_bundle: Text2dBundle {
                transform: Transform {
                    translation: position.as_world_position().add(translation_offset.0),
                    scale: Vec3::new(0.01, 0.01, 0.0),
                    ..default()
                },
                text: Text::from_section(
                    name,
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        color: Color::BLACK,
                        font_size: 60.0,
                        ..default()
                    },
                ),
                ..default()
            },
            translation_offset,
            label: Label(ant),
        }
    }
}

// 1.2 is just a feel good number to make ants slightly larger than the elements they dig up
const ANT_SCALE: f32 = 1.2;

#[derive(Component, Copy, Clone)]
pub struct TranslationOffset(pub Vec3);

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AntName(pub String);

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct AntColor(pub Color);

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct AntInventory(pub Option<Element>);

impl AntInventory {
    pub fn get_carrying_bundle(&self) -> Option<CarryingBundle> {
        if self.0 == Some(Element::Sand) {
            return Some(CarryingBundle {
                sprite_bundle: SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(0.5, 0.5, 1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        color: Color::rgb(0.761, 0.698, 0.502),
                        anchor: Anchor::TopLeft,
                        ..default()
                    },
                    ..default()
                },
                element: Element::Sand,
            });
        } else if self.0 == Some(Element::Food) {
            return Some(CarryingBundle {
                sprite_bundle: SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(0.5, 0.5, 1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        color: Color::rgb(0.388, 0.584, 0.294),
                        anchor: Anchor::TopLeft,
                        ..default()
                    },
                    ..default()
                },
                element: Element::Food,
            });
        }

        None
    }
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Alive;

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Hunger {
    value: usize,
    max: usize,
}

impl Hunger {
    pub fn default() -> Self {
        Self {
            value: 0,
            // TODO: this is 6 * 60 * 60 * 24 which is 1 day expressed in frame ticks
            max: 518400,
        }
    }

    pub fn try_increment(&mut self) {
        if self.value < self.max {
            self.value += 1;
        }
    }

    pub fn as_percent(&self) -> usize {
        self.value / self.max
    }

    pub fn is_hungry(&self) -> bool {
        self.value >= self.max / 2
    }

    pub fn is_starving(&self) -> bool {
        self.value >= self.max
    }

    pub fn reset(&mut self) {
        self.value = 0;
    }
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum AntRole {
    Worker,
    Queen,
}

#[derive(Bundle)]
pub struct CarryingBundle {
    sprite_bundle: SpriteBundle,
    element: Element,
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct AntTimer(pub isize);

impl AntTimer {
    pub fn new(rng: &mut StdRng) -> Self {
        Self(rng.gen_range(5..7))
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Facing {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Angle {
    Zero = 0,
    Ninety = 90,
    OneHundredEighty = 180,
    TwoHundredSeventy = 270,
}

impl Angle {
    pub fn as_radians(self) -> f32 {
        (self as isize as f32) * PI / 180.0
    }

    /**
     * Rotation is a value from 0 to 3. A value of 1 is a 90 degree counter-clockwise rotation. Negative values are accepted.
     * Examples:
     *  rotate(0, -1); // 270
     *  rotate(0, 1); // 90
     */
    pub fn rotate(self, rotation: i32) -> Self {
        let angles = [
            Angle::Zero,
            Angle::Ninety,
            Angle::OneHundredEighty,
            Angle::TwoHundredSeventy,
        ];

        let rotated_index = (angles.iter().position(|&a| a == self).unwrap() as i32 - rotation)
            % angles.len() as i32;
        angles[((rotated_index + angles.len() as i32) % angles.len() as i32) as usize]
    }
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct AntOrientation {
    facing: Facing,
    angle: Angle,
}

impl AntOrientation {
    pub fn new(facing: Facing, angle: Angle) -> Self {
        Self { facing, angle }
    }

    // Convert AntOrientation to Transform.Scale, z-index is naively set to 1 for now
    pub fn as_world_scale(&self) -> Vec3 {
        Vec3 {
            x: if self.get_facing() == Facing::Left {
                -1.0
            } else {
                1.0
            },
            y: 1.0,
            z: 1.0,
        }
    }

    pub fn as_world_rotation(&self) -> Quat {
        Quat::from_rotation_z(self.get_angle().as_radians())
    }

    pub fn get_facing(self) -> Facing {
        self.facing
    }

    pub fn get_angle(self) -> Angle {
        self.angle
    }

    pub fn turn_around(self) -> Self {
        let facing = if self.facing == Facing::Left {
            Facing::Right
        } else {
            Facing::Left
        };

        Self::new(facing, self.angle)
    }

    pub fn flip_onto_back(self) -> Self {
        self.rotate_towards_back().rotate_towards_back()
    }

    pub fn rotate_towards_feet(self) -> Self {
        let rotation = if self.facing == Facing::Left { -1 } else { 1 };

        Self::new(self.facing, self.angle.rotate(rotation))
    }

    pub fn rotate_towards_back(self) -> Self {
        let rotation = if self.facing == Facing::Left { 1 } else { -1 };

        Self::new(self.facing, self.angle.rotate(rotation))
    }

    pub fn get_forward_delta(self) -> Position {
        let delta = match self.angle {
            Angle::Zero => Position::X,
            Angle::Ninety => Position::NEG_Y,
            Angle::OneHundredEighty => Position::NEG_X,
            Angle::TwoHundredSeventy => Position::Y,
        };

        if self.facing == Facing::Left {
            delta * Position::NEG_ONE
        } else {
            delta
        }
    }
}

#[derive(Component)]
pub struct Label(pub Entity);

pub fn setup_ants(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
    world_map: ResMut<WorldMap>,
    mut world_rng: ResMut<WorldRng>,
) {
    for ant_save_state in world_map.initial_state().ants.iter() {
        let entity = commands
            .spawn(AntBundle::new(
                ant_save_state.position,
                settings.ant_color,
                ant_save_state.orientation,
                ant_save_state.inventory,
                ant_save_state.role,
                ant_save_state.name.0.as_str(),
                &asset_server,
                &mut world_rng.0,
            ))
            .with_children(|parent| {
                if let Some(bundle) = ant_save_state.inventory.get_carrying_bundle() {
                    parent.spawn(bundle);
                }

                // if ant_save_state.role == AntRole::Queen {
                //     parent.spawn(SpriteBundle {
                //         texture: asset_server.load("images/crown.png"),
                //         transform: Transform {
                //             translation: Vec3::new(0.25, 0.5, 1.0),
                //             ..default()
                //         },
                //         sprite: Sprite {
                //             custom_size: Some(Vec2::new(0.5, 0.5)),
                //             ..default()
                //         },
                //         ..default()
                //     });
                // }
            })
            .id();

        commands.spawn(AntLabelBundle::new(
            entity,
            ant_save_state.position,
            ant_save_state.name.0.as_str(),
            &asset_server,
        ));
    }
}

fn is_valid_location(
    orientation: AntOrientation,
    position: Position,
    elements_query: &Query<&Element>,
    world_map: &ResMut<WorldMap>,
) -> bool {
    // Need air at the ants' body for it to be a legal ant location.
    let Some(entity) = world_map.get_element(position) else { return false };

    // NOTE: this can occur due to `spawn` not affecting query on current frame
    let Ok(element) = elements_query.get(*entity) else { return false; };

    if *element != Element::Air {
        return false;
    }

    // Get the location beneath the ants' feet and check for air
    let foot_position = position + orientation.rotate_towards_feet().get_forward_delta();
    let Some(entity) = world_map.get_element(foot_position) else { return false };
    // NOTE: this can occur due to `spawn` not affecting query on current frame
    let Ok(element) = elements_query.get(*entity) else { return false; };

    if *element == Element::Air {
        return false;
    }

    true
}

fn do_drop(
    mut inventory: Mut<AntInventory>,
    position: Position,
    elements_query: &Query<&Element>,
    world_map: &mut ResMut<WorldMap>,
    commands: &mut Commands,
) {
    let Some(entity) = world_map.get_element(position) else { return; };
    // NOTE: this can occur due to `spawn` not affecting query on current frame
    let Ok(element) = elements_query.get(*entity) else { return; };

    if *element == Element::Air {
        commands.entity(*entity).despawn();

        // Drop inventory
        if inventory.0 == Some(Element::Food) {
            let food_entity = commands.spawn(FoodElementBundle::new(position)).id();
            world_map.set_element(position, food_entity);
        } else if inventory.0 == Some(Element::Sand) {
            let sand_entity = commands.spawn(SandElementBundle::new(position)).id();
            world_map.set_element(position, sand_entity);
        }

        *inventory = AntInventory(None);
    }
}

fn do_turn(
    mut orientation: Mut<AntOrientation>,
    inventory: Mut<AntInventory>,
    position: Mut<Position>,
    elements_query: &Query<&Element>,
    world_map: &mut ResMut<WorldMap>,
    world_rng: &mut ResMut<WorldRng>,
    commands: &mut Commands,
) {
    // First try turning perpendicularly towards the ant's back. If that fails, try turning around.
    let back_orientation = orientation.rotate_towards_back();
    if is_valid_location(back_orientation, *position, elements_query, world_map) {
        *orientation = back_orientation;
        return;
    }

    let opposite_orientation = orientation.turn_around();
    if is_valid_location(opposite_orientation, *position, elements_query, world_map) {
        *orientation = opposite_orientation;
        return;
    }

    // Randomly turn in a valid different when unable to simply turn around.
    let facings = [Facing::Left, Facing::Right];
    let angles = [
        Angle::Zero,
        Angle::Ninety,
        Angle::OneHundredEighty,
        Angle::TwoHundredSeventy,
    ];
    let facing_orientations = facings
        .iter()
        .flat_map(|facing| {
            angles
                .iter()
                .map(move |angle| AntOrientation::new(*facing, *angle))
        })
        .collect::<Vec<_>>();

    let valid_facing_orientations = facing_orientations
        .iter()
        .filter(|&inner_orientation| {
            if inner_orientation.facing == orientation.facing
                && inner_orientation.angle == orientation.angle
            {
                return false;
            }

            is_valid_location(*inner_orientation, *position, elements_query, world_map)
        })
        .collect::<Vec<_>>();

    if valid_facing_orientations.len() > 0 {
        let valid_facing_orientation =
            valid_facing_orientations[world_rng.0.gen_range(0..valid_facing_orientations.len())];

        *orientation = *valid_facing_orientation;
        return;
    }

    // No legal direction? Trapped! Drop if carrying and turn randomly in an attempt to dig out.
    if inventory.0 != None {
        if let Some(entity) = world_map.get_element(*position) {
            // TODO: maybe this should exit early rather than allowing for turning since relevant state has been mutated
            // NOTE: this can occur due to `spawn` not affecting query on current frame
            if let Ok(element) = elements_query.get(*entity) {
                if *element == Element::Air {
                    do_drop(inventory, *position, elements_query, world_map, commands);
                }
            }
        }
    }

    let random_facing_orientation =
        facing_orientations[world_rng.0.gen_range(0..facing_orientations.len())];
    *orientation = random_facing_orientation;
}

fn do_dig(
    is_forced_forward: bool,
    mut inventory: Mut<AntInventory>,
    orientation: Mut<AntOrientation>,
    position: Mut<Position>,
    elements_query: &Query<&Element>,
    world_map: &mut ResMut<WorldMap>,
    commands: &mut Commands,
) {
    let dig_position = if is_forced_forward {
        orientation.get_forward_delta() + *position
    } else {
        orientation.rotate_towards_feet().get_forward_delta() + *position
    };

    let Some(entity) = world_map.get_element(dig_position) else { return };
    // NOTE: this can occur due to `spawn` not affecting query on current frame
    let Ok(element) = elements_query.get(*entity) else { return };

    if *element == Element::Dirt || *element == Element::Sand || *element == Element::Food {
        commands.entity(*entity).despawn();

        // Dig up dirt/sand/food and replace with air
        let air_entity = commands.spawn(AirElementBundle::new(dig_position)).id();
        world_map.set_element(dig_position, air_entity);

        loosen_neighboring_sand_and_food(dig_position, world_map, elements_query, commands);

        if *element == Element::Food {
            *inventory = AntInventory(Some(Element::Food));
        } else {
            // NOTE: the act of digging up dirt converts it to sand (intentionally)
            *inventory = AntInventory(Some(Element::Sand));
        }
    }
}

fn do_move(
    mut orientation: Mut<AntOrientation>,
    inventory: Mut<AntInventory>,
    mut position: Mut<Position>,
    elements_query: &Query<&Element>,
    world_map: &mut ResMut<WorldMap>,
    world_rng: &mut ResMut<WorldRng>,
    settings: &Res<Settings>,
    commands: &mut Commands,
) {
    let new_position = *position + orientation.get_forward_delta();

    if !world_map.is_within_bounds(&new_position) {
        // Hit an edge - need to turn.
        do_turn(
            orientation,
            inventory,
            position,
            elements_query,
            world_map,
            world_rng,
            commands,
        );
        return;
    }

    // Check if hitting dirt or sand and, if so, consider digging through it.
    let entity = world_map.get_element(new_position).unwrap();
    // NOTE: this can occur due to `spawn` not affecting query on current frame
    let Ok(element) = elements_query.get(*entity) else { return };

    // NOTE: this is more like "for all elements that have mass - i.e. not air" which holds for now but is a baked-in assumption
    if *element != Element::Air {
        // If ant is wandering *below ground level* and bumps into sand or has a chance to dig, dig.
        if inventory.0 == None
            && position.y > *world_map.surface_level()
            && (*element == Element::Sand
                || world_rng.0.gen::<f32>() < settings.probabilities.below_surface_dig)
        {
            do_dig(
                true,
                inventory,
                orientation,
                position,
                &elements_query,
                world_map,
                commands,
            );
            return;
        } else {
            do_turn(
                orientation,
                inventory,
                position,
                elements_query,
                world_map,
                world_rng,
                commands,
            );
        }

        return;
    }

    // We can move forward.  But first, check footing.
    let foot_orientation = orientation.rotate_towards_feet();
    let foot_position = new_position + foot_orientation.get_forward_delta();

    if let Some(foot_entity) = world_map.get_element(foot_position) {
        // NOTE: this can occur due to `spawn` not affecting query on current frame
        let Ok(foot_element) = elements_query.get(*foot_entity) else { return };

        if *foot_element == Element::Air {
            // If ant moves straight forward, it will be standing over air. Instead, turn into the air and remain standing on current block
            // Ant will try to fill the gap with sand if possible.
            let should_drop_sand = inventory.0 == Some(Element::Sand)
                && position.y <= *world_map.surface_level()
                && world_rng.0.gen::<f32>() < settings.probabilities.above_surface_drop;

            if should_drop_sand {
                do_drop(inventory, *position, &elements_query, world_map, commands);
            } else {
                // Update position and angle
                *position = foot_position;
                *orientation = foot_orientation;
            }
            return;
        }

        // Just move forward
        *position = new_position;
    }
}

pub fn ants_hunger_system(
    mut ants_hunger_query: Query<
        (
            Entity,
            &mut Hunger,
            &mut Handle<Image>,
            &mut AntOrientation,
            &Position,
            &AntInventory,
        ),
        With<Alive>,
    >,
    elements_query: Query<&Element>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut world_map: ResMut<WorldMap>,
) {
    for (entity, mut hunger, mut handle, mut orientation, position, inventory) in
        ants_hunger_query.iter_mut()
    {
        hunger.try_increment();

        // If ant is holding or adjacent to food then eat the food and reset hunger.
        if hunger.is_hungry() {
            if inventory.0 == None {
                // Check position in front of ant for food
                let food_position = *position + orientation.get_forward_delta();
                if is_element(&world_map, &elements_query, &food_position, &Element::Food) {
                    // Eat the food
                    let food_entity = world_map.get_element(food_position).unwrap();

                    commands.entity(*food_entity).despawn();
                    world_map.set_element(
                        food_position,
                        commands.spawn(AirElementBundle::new(food_position)).id(),
                    );

                    hunger.reset();
                }
            }
        }

        if hunger.is_starving() {
            commands.entity(entity).remove::<Alive>();

            // TODO: prefer respond to Alive removal and/or responding to addition of Dead instead of inline code here
            *handle = asset_server.load("images/ant_dead.png");
            *orientation = orientation.flip_onto_back();
        }
    }
}

// TODO: untangle mutability, many functions accept mutable but do not mutate which seems wrong
// TODO: first pass is going to be just dumping all code into one system, but suspect want multiple systems
pub fn move_ants_system(
    mut ants_query: Query<
        (
            &mut AntOrientation,
            &mut AntInventory,
            &mut AntTimer,
            &mut Position,
        ),
        With<Alive>,
    >,
    elements_query: Query<&Element>,
    mut world_map: ResMut<WorldMap>,
    settings: Res<Settings>,
    mut world_rng: ResMut<WorldRng>,
    mut commands: Commands,
) {
    for (orientation, inventory, mut timer, position) in ants_query.iter_mut() {
        if timer.0 > 0 {
            timer.0 -= 1;
            continue;
        }

        *timer = AntTimer::new(&mut world_rng.0);

        let below_feet_position = *position + orientation.rotate_towards_feet().get_forward_delta();

        let is_air_beneath_feet = is_all_element(
            &world_map,
            &elements_query,
            &[below_feet_position],
            &Element::Air,
        );

        if is_air_beneath_feet {
            // Whoops, whatever we were walking on disappeared.
            let below_position = *position + Position::Y;
            let is_air_below = is_all_element(
                &world_map,
                &elements_query,
                &[below_position],
                &Element::Air,
            );

            // Gravity system will handle things if going to fall
            if is_air_below {
                continue;
            }

            // Not falling? Try turning
            do_turn(
                orientation,
                inventory,
                position,
                &elements_query,
                &mut world_map,
                &mut world_rng,
                &mut commands,
            );

            continue;
        }

        if inventory.0 == None {
            if world_rng.0.gen::<f32>() < settings.probabilities.random_dig {
                do_dig(
                    false,
                    inventory,
                    orientation,
                    position,
                    &elements_query,
                    &mut world_map,
                    &mut commands,
                )
            } else if world_rng.0.gen::<f32>() < settings.probabilities.random_turn {
                do_turn(
                    orientation,
                    inventory,
                    position,
                    &elements_query,
                    &mut world_map,
                    &mut world_rng,
                    &mut commands,
                );
            } else {
                do_move(
                    orientation,
                    inventory,
                    position,
                    &elements_query,
                    &mut world_map,
                    &mut world_rng,
                    &settings,
                    &mut commands,
                );
            }
        } else {
            if world_rng.0.gen::<f32>() < settings.probabilities.random_drop {
                do_drop(
                    inventory,
                    *position,
                    &elements_query,
                    &mut world_map,
                    &mut commands,
                );
            } else {
                do_move(
                    orientation,
                    inventory,
                    position,
                    &elements_query,
                    &mut world_map,
                    &mut world_rng,
                    &settings,
                    &mut commands,
                );
            }
        }
    }
}

// TODO: tests
