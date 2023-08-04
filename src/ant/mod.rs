use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::{
    element::{
        is_element, is_all_element, commands::ElementCommandsExt
    },
    map::{Position, WorldMap},
    world_rng::WorldRng, name_list::NAMES,
};

use self::commands::AntCommandsExt;

use super::{element::Element, settings::Settings};
use bevy::{prelude::*, sprite::Anchor};
use rand::{rngs::StdRng, Rng};

mod commands;
// TODO: maybe don't want this public?
pub mod ui;

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
    ant: Ant,
    position: Position,
    orientation: AntOrientation,
    role: AntRole,
    timer: AntTimer,
    name: AntName,
    color: AntColor,
    hunger: Hunger,
    alive: Alive,
    inventory: AntInventory,
}

impl AntBundle {
    pub fn new(
        position: Position,
        color: Color,
        orientation: AntOrientation,
        inventory: AntInventory,
        role: AntRole,
        name: &str,
        mut rng: &mut StdRng,
    ) -> Self {
        AntBundle {
            ant: Ant,
            position,
            orientation,
            inventory,
            role,
            timer: AntTimer::new(&mut rng),
            name: AntName(name.to_string()),
            color: AntColor(color),
            hunger: Hunger::default(),
            alive: Alive,
        }
    }
}

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
                        translation: Vec3::new(0.5, 0.75, 1.0),
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
                        translation: Vec3::new(0.5, 0.75, 1.0),
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
pub struct Birthing {
    value: usize,
    max: usize,
}

impl Birthing {
    pub fn default() -> Self {
        Self {
            value: 0,
            // TODO: 30 minutes expressed in frame ticks
            max: 6 * 60 * 30,
        }
    }

    pub fn try_increment(&mut self) {
        if self.value < self.max {
            self.value += 1;
        }
    }

    pub fn is_ready(&self) -> bool {
        self.value >= self.max
    }

    pub fn reset(&mut self) {
        self.value = 0;
    }
}

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

    pub fn as_percent(&self) -> f64 {
        ((self.value as f64) / (self.max as f64) * 100.0).round()
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

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Ant;

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
        Self(rng.gen_range(3..5))
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

    pub fn is_horizontal(self) -> bool {
        self.angle == Angle::Zero || self.angle == Angle::OneHundredEighty
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
    settings: Res<Settings>,
    world_map: ResMut<WorldMap>,
    mut world_rng: ResMut<WorldRng>,
) {
    for ant_save_state in world_map.initial_state().ants.iter() {
        commands
            .spawn(AntBundle::new(
                ant_save_state.position,
                settings.ant_color,
                ant_save_state.orientation,
                ant_save_state.inventory,
                ant_save_state.role,
                ant_save_state.name.0.as_str(),
                &mut world_rng.0,
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
    let Ok(element) = elements_query.get(*entity) else { panic!("is_valid_location - expected entity to exist") };
    

    if *element != Element::Air {
        return false;
    }

    // Get the location beneath the ants' feet and check for air
    let foot_position = position + orientation.rotate_towards_feet().get_forward_delta();
    let Some(entity) = world_map.get_element(foot_position) else { return false };
    let Ok(element) = elements_query.get(*entity) else { panic!("is_valid_location - expected entity to exist") };

    if *element == Element::Air {
        return false;
    }

    true
}

fn turn(
    mut orientation: Mut<AntOrientation>,
    // inventory: Mut<AntInventory>,
    position: Mut<Position>,
    elements_query: &Query<&Element>,
    world_map: &mut ResMut<WorldMap>,
    world_rng: &mut ResMut<WorldRng>,
    // commands: &mut Commands,
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

    info!("TRAPPED");
    // panic!();

    // No legal direction? Trapped! Drop if carrying and turn randomly in an attempt to dig out.
    // if inventory.0 != None {
    //     if let Some(entity) = world_map.get_element(*position) {
    //         // TODO: maybe this should exit early rather than allowing for turning since relevant state has been mutated
    //         let Ok(element) = elements_query.get(*entity) else { panic!("turn - expected entity to exist") };
    //         if *element == Element::Air {
    //             let target_element_entity = world_map.get_element_expect(*position);
    //             commands.drop(ant_entity, *position, *target_element_entity);
    //         }
    //     }
    // }

    let random_facing_orientation =
        facing_orientations[world_rng.0.gen_range(0..facing_orientations.len())];
    *orientation = random_facing_orientation;
}

fn act(
    mut orientation: Mut<AntOrientation>,
    inventory: Mut<AntInventory>,
    mut position: Mut<Position>,
    role: &AntRole,
    ant_entity: Entity,
    elements_query: &Query<&Element>,
    world_map: &mut ResMut<WorldMap>,
    world_rng: &mut ResMut<WorldRng>,
    settings: &Res<Settings>,
    commands: &mut Commands,
) {
    // Propose taking a step forward, but check validity and alternative actions before stepping forward.
    let new_position = *position + orientation.get_forward_delta();

    if !world_map.is_within_bounds(&new_position) {
        // Hit an edge - need to turn.
        turn(
            orientation,
            // inventory,
            position,
            elements_query,
            world_map,
            world_rng,
            // commands,
        );
        return;
    }

    // Check if hitting a solid element and, if so, consider digging through it.
    let entity = world_map.get_element(new_position).unwrap();
    let Ok(element) = elements_query.get(*entity) else { panic!("act - expected entity to exist") };

    if *element != Element::Air {
        // Consider digging / picking up the element under various circumstances.
        if inventory.0 == None {
            // When above ground, prioritize picking up food
            let dig_food = *element == Element::Food && !world_map.is_below_surface(&position);
            // When underground, prioritize clearing out sand and allow for digging tunnels through dirt. Leave food underground.
            let dig_sand = *element == Element::Sand && world_map.is_below_surface(&position);
            let dig_dirt = *element == Element::Dirt
                && world_map.is_below_surface(&position)
                && world_rng.0.gen::<f32>() < settings.probabilities.below_surface_dirt_dig;


            // TODO: Make this a little less rigid - it's weird seeing always a straight line down 8.
            let mut queen_dig = *element == Element::Dirt && world_map.is_below_surface(&position) && *role == AntRole::Queen;

            if position.y - world_map.surface_level() > 8 {
                if world_rng.0.gen::<f32>() > settings.probabilities.below_surface_queen_nest_dig {
                    queen_dig = false;
                }

                // Once at sufficient depth it's not guaranteed that queen will want to continue digging deeper.
            } 

                // if *role == AntRole::Queen {
                //     if inventory.0 == None {
                //         if world_map.is_below_surface(&position) &&  {

            if dig_food || dig_sand || dig_dirt || queen_dig {
                let target_position = *position + orientation.get_forward_delta();
                let target_element_entity = *world_map.get_element_expect(target_position);
                commands.dig(ant_entity, target_position, target_element_entity);
                return;
            }
        }

        // Decided to not dig through and can't walk through, so just turn.
        turn(
            orientation,
            // inventory,
            position,
            elements_query,
            world_map,
            world_rng,
            // commands,
        );

        return;
    }

    // There is an air gap directly ahead of the ant. Consider dropping inventory.
    // Avoid dropping inventory when facing upwards since it'll fall on the ant.
    if inventory.0 != None && orientation.is_horizontal()  {
        // Prioritize dropping sand above ground and food below ground.
        let drop_sand = inventory.0 == Some(Element::Sand)
            && !world_map.is_below_surface(&new_position)
            && world_rng.0.gen::<f32>() < settings.probabilities.above_surface_sand_drop;

        let drop_food = inventory.0 == Some(Element::Food)
            && world_map.is_below_surface(&new_position)
            && world_rng.0.gen::<f32>() < settings.probabilities.below_surface_food_drop;

        if drop_sand || drop_food {
            // Drop inventory in front of ant
            let target_element_entity = world_map.get_element_expect(new_position);
            commands.drop(ant_entity, new_position, *target_element_entity);
   
            if *role == AntRole::Queen {
                turn(
                    orientation,
                    // inventory,
                    position,
                    elements_query,
                    world_map,
                    world_rng,
                    // commands,
                );
            }

            return;
        }
    }

    // Decided to not drop inventory. Check footing and move forward if possible.
    let foot_orientation = orientation.rotate_towards_feet();
    let foot_position = new_position + foot_orientation.get_forward_delta();

    if let Some(foot_entity) = world_map.get_element(foot_position) {
        let Ok(foot_element) = elements_query.get(*foot_entity) else { panic!("act - expected entity to exist") };

        if *foot_element == Element::Air {
            // If ant moves straight forward, it will be standing over air. Instead, turn into the air and remain standing on current block
            *position = foot_position;
            *orientation = foot_orientation;
        } else {
            // Just move forward
            *position = new_position;
        }
    }
}

pub fn ants_birthing_system(
    mut ants_birthing_query: Query<
        (
            &mut Birthing,
            &Position,
            &AntColor,
            &AntOrientation,
        ),
        With<Alive>,
    >,
    mut commands: Commands,
    mut world_rng: ResMut<WorldRng>,
) {
    for (mut birthing, position, color, orientation) in ants_birthing_query.iter_mut()
    {
        birthing.try_increment();

        if birthing.is_ready() {
            // Randomly position ant facing left or right.
            let facing = if world_rng.0.gen_bool(0.5) {
                Facing::Left
            } else {
                Facing::Right
            };

            
            let name: &str = NAMES[world_rng.0.gen_range(0..NAMES.len())].clone();

            let behind_position = *position + orientation.turn_around().get_forward_delta();

            // Spawn worker ant (TODO: egg instead)
            commands.spawn(AntBundle::new(
                behind_position,
                color.0,
                AntOrientation::new(facing, Angle::Zero),
                AntInventory(None),
                AntRole::Worker,
                name,
                &mut world_rng.0,
            ));

            birthing.reset();
        }
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
    world_map: Res<WorldMap>,
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
                    info!("replace_element4: {:?}", position);

                    // TODO: command for eating food
                    commands.replace_element(food_position, *food_entity, Element::Air);

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
            &AntRole,
            Entity,
        ),
        With<Alive>,
    >,
    elements_query: Query<&Element>,
    mut world_map: ResMut<WorldMap>,
    settings: Res<Settings>,
    mut world_rng: ResMut<WorldRng>,
    mut commands: Commands,
) {
    // TODO: Check if ant position has changed already and, if so, skip it - ant is only allowed to be moved on its own if another system didn't move it this frame.
    // This will allow for systems to run not in parallel, but in a non-deterministic order while exhibiting desirable behavior.

    for (orientation, inventory, mut timer, position, role, ant_entity) in ants_query.iter_mut() {
        if timer.0 > 0 {
            timer.0 -= 1;
            continue;
        }

        *timer = AntTimer::new(&mut world_rng.0);

        let below_feet_position = *position + orientation.rotate_towards_feet().get_forward_delta();
        let is_air_beneath_feet = is_element(
            &world_map,
            &elements_query,
            &below_feet_position,
            &Element::Air,
        );

        if is_air_beneath_feet {
            // Whoops, whatever we were walking on disappeared.
            let below_position = *position + Position::Y;
            let is_air_below =
                is_element(&world_map, &elements_query, &below_position, &Element::Air);

            // Gravity system will handle things if going to fall
            if is_air_below {
                continue;
            }

            // Not falling? Try turning
            turn(
                orientation,
                // inventory,
                position,
                &elements_query,
                &mut world_map,
                &mut world_rng,
                // &mut commands,
            );

            continue;
        }

        // TODO: queen specific logic
        if *role == AntRole::Queen {
            if !world_map.is_below_surface(&position) && !world_map.has_started_nest() {
                if inventory.0 == None {
                    if world_rng.0.gen::<f32>() < settings.probabilities.above_surface_queen_nest_dig {

                        let target_position = *position + orientation.rotate_towards_feet().get_forward_delta();
                        let target_element_entity = *world_map.get_element_expect(target_position);
                        commands.dig(ant_entity, target_position, target_element_entity);

                        // TODO: replace this with pheromones - queen should be able to find her way back to dig site via pheromones rather than
                        // enforcing nest generation probabilistically
                        world_map.start_nest();
        
                        continue;
                    } 
                }
            }

            if position.y - world_map.surface_level() > 8 && !world_map.is_nested() {
                // Check if the queen is sufficiently surounded by space while being deep underground and, if so, decide to start nesting.

                let left_position = *position + Position::NEG_X;
                //let left_above_position = *position + Position::new(-1, -1);
                let above_position = *position + Position::new(0, -1);
                let right_position = *position + Position::X;
                //let right_above_position = *position + Position::new(1, -1);

                let has_valid_air_nest = is_all_element(
                    &world_map,
                    &elements_query,
                    &[left_position, *position, above_position, right_position],
                    &Element::Air,
                );

                //let left_below_position = *position + Position::new(-1, 1);
                let below_position = *position + Position::new(0, 1);
                //let right_below_position = *position + Position::new(1, 1);
                // Make sure there's stable place for ant child to be born
                let behind_position = *position + orientation.turn_around().get_forward_delta();
                let behind_below_position = behind_position + Position::new(0, 1);

                let has_valid_dirt_nest = is_all_element(
                    &world_map,
                    &elements_query,
                    &[below_position, behind_below_position],
                    &Element::Dirt,
                );

                if has_valid_air_nest && has_valid_dirt_nest {
                    info!("NESTED");
                    world_map.mark_nested();

                    // Spawn birthing component on QueenAnt
                    commands.entity(ant_entity).insert(Birthing::default());

                    if inventory.0 != None {
                        let target_position = *position + orientation.get_forward_delta();
                        let target_element_entity = world_map.get_element_expect(target_position);
                        commands.drop(ant_entity, target_position, *target_element_entity);
                    }

                    continue;
                }
            }

            if world_map.is_nested() { 
                continue;
            }
        }
        
        if world_rng.0.gen::<f32>() < settings.probabilities.random_turn {
            turn(
                orientation,
                // inventory,
                position,
                &elements_query,
                &mut world_map,
                &mut world_rng,
                // &mut commands,
            );

            continue;
        }

        // Add some randomness to worker behavior to make more lively, need to avoid applying this to queen because
        // too much randomness can kill her before she can nest
        if *role == AntRole::Worker {
            if inventory.0 == None {
                // Randomly dig downwards / perpendicular to current orientation
                if world_rng.0.gen::<f32>() < settings.probabilities.random_dig && world_map.is_below_surface(&position) {
                    let target_position = *position + orientation.rotate_towards_feet().get_forward_delta();
                    let target_element_entity = *world_map.get_element_expect(target_position);
                    commands.dig(ant_entity, target_position, target_element_entity);
    
                    continue;
                }
            } else {
                if world_rng.0.gen::<f32>() < settings.probabilities.random_drop {
                    let target_element_entity = world_map.get_element_expect(*position);
                    commands.drop(ant_entity, *position, *target_element_entity);
    
                    continue;
                }
            }
        }

        act(
            orientation,
            inventory,
            position,
            role,
            ant_entity,
            &elements_query,
            &mut world_map,
            &mut world_rng,
            &settings,
            &mut commands,
        );
    }
}

// TODO: tests
