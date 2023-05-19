use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, ops::Add};

use crate::{
    elements::{is_all_element, AirElementBundle, SandElementBundle},
    gravity::loosen_neighboring_sand,
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
    pub behavior: AntBehavior,
    pub timer: AntTimer,
    pub name: AntName,
}

// TODO: This seems like an ANTi-pattern (heh heh heh), it's sort of like a bundle, but it needs parent/child relationships so it can't just be spawned as a bundle itself
struct Ant {
    position: Position,
    transform_offset: TransformOffset,
    orientation: AntOrientation,
    behavior: AntBehavior,
    timer: AntTimer,
    name: AntName,
    color: AntColor,
    sprite_bundle: SpriteBundle,
    label_bundle: Text2dBundle,
}

impl Ant {
    pub fn new(
        position: Position,
        color: Color,
        orientation: AntOrientation,
        behavior: AntBehavior,
        name: &str,
        asset_server: &Res<AssetServer>,
        mut rng: &mut StdRng,
    ) -> Self {
        let transform_offset = TransformOffset(Vec3::new(0.5, -0.5, 0.0));
        let x_flip = if orientation.facing == Facing::Left {
            -1.0
        } else {
            1.0
        };
        let translation = Vec3::new(position.x as f32, -position.y as f32, 1.0);

        Self {
            position,
            transform_offset,
            orientation,
            behavior,
            timer: AntTimer::new(&behavior, &mut rng),
            name: AntName(name.to_string()),
            color: AntColor(color),
            sprite_bundle: SpriteBundle {
                texture: asset_server.load("images/ant.png"),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(ANT_SCALE, ANT_SCALE)),
                    ..default()
                },
                transform: Transform {
                    translation: translation.add(transform_offset.0),
                    rotation: Quat::from_rotation_z(orientation.angle.as_radians()),
                    scale: Vec3::new(x_flip, 1.0, 1.0),
                    ..default()
                },
                ..default()
            },
            label_bundle: Text2dBundle {
                transform: Transform {
                    translation: Vec3::new(-ANT_SCALE / 4.0, -1.5, 1.0),
                    scale: Vec3::new(0.05, 0.05, 0.0),
                    ..default()
                },
                text: Text::from_section(
                    name,
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        color: Color::rgb(0.0, 0.0, 0.0),
                        font_size: 12.0,
                        ..default()
                    },
                ),
                ..default()
            },
        }
    }
}

// 1.2 is just a feel good number to make ants slightly larger than the elements they dig up
const ANT_SCALE: f32 = 1.2;

#[derive(Component, Copy, Clone)]
pub struct TransformOffset(pub Vec3);

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AntName(pub String);

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct AntColor(pub Color);

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum AntBehavior {
    Wandering,
    Carrying,
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct AntTimer(pub isize);

impl AntTimer {
    pub fn new(behavior: &AntBehavior, rng: &mut StdRng) -> Self {
        let timing_factor = match behavior {
            AntBehavior::Wandering => 3,
            AntBehavior::Carrying => 4,
        };

        Self(timing_factor + rng.gen_range(0..3))
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
pub struct LabelContainer;

// Spawn non-interactive background (sky blue / tunnel brown)
pub fn setup_ants(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
    world_map: ResMut<WorldMap>,
    mut world_rng: ResMut<WorldRng>,
) {
    let ants = world_map
        .initial_state
        .ants
        .iter()
        .map(|ant_save_state| {
            Ant::new(
                ant_save_state.position,
                settings.ant_color,
                ant_save_state.orientation,
                ant_save_state.behavior,
                ant_save_state.name.0.as_str(),
                &asset_server,
                &mut world_rng.0,
            )
        })
        .collect::<Vec<_>>();

    for ant in ants {
        commands
            // Wrap label and ant with common parent to allow a system to easily associate their position.
            // Don't mess with parent transform to avoid rotating label.
            .spawn(SpatialBundle::default())
            .with_children(|ant_label_container| {
                // Spawn a container for the ant sprite and sand so there's strong correlation between position and translation.
                ant_label_container
                    .spawn((
                        ant.sprite_bundle,
                        ant.position,
                        ant.transform_offset,
                        ant.orientation,
                        ant.behavior,
                        ant.timer,
                        ant.name,
                        ant.color,
                    ))
                    .with_children(|parent| {
                        if ant.behavior == AntBehavior::Carrying {
                            // Make sand a child of ant so they share rotation.
                            parent.spawn((
                                SpriteBundle {
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
                                Element::Sand,
                            ));
                        }
                    });

                // Put the label in its own container so offset on label itself isn't overwritten when updating position
                ant_label_container
                    .spawn((
                        SpatialBundle {
                            transform: Transform {
                                translation: Vec3::new(
                                    ant.position.x as f32,
                                    -ant.position.y as f32,
                                    1.0,
                                ),
                                ..default()
                            },
                            ..default()
                        },
                        LabelContainer,
                    ))
                    .with_children(|label_container| {
                        label_container.spawn(ant.label_bundle);
                    });
            });
    }
}

fn is_valid_location(
    orientation: AntOrientation,
    position: Position,
    elements_query: &Query<&Element>,
    world_map: &ResMut<WorldMap>,
) -> bool {
    // Need air at the ants' body for it to be a legal ant location.
    let Some(entity) = world_map.elements.get(&position) else { return false };
    // NOTE: this can occur due to `spawn` not affecting query on current frame
    let Ok(element) = elements_query.get(*entity) else { return false; };

    if *element != Element::Air {
        return false;
    }

    // Get the location beneath the ants' feet and check for air
    let foot_position = position + orientation.rotate_towards_feet().get_forward_delta();
    let Some(entity) = world_map.elements.get(&foot_position) else { return false };
    // NOTE: this can occur due to `spawn` not affecting query on current frame
    let Ok(element) = elements_query.get(*entity) else { return false; };

    if *element == Element::Air {
        return false;
    }

    true
}

fn do_drop(
    mut behavior: Mut<AntBehavior>,
    position: Position,
    elements_query: &Query<&Element>,
    world_map: &mut ResMut<WorldMap>,
    commands: &mut Commands,
) {
    let Some(entity) = world_map.elements.get(&position) else { return; };
    // NOTE: this can occur due to `spawn` not affecting query on current frame
    let Ok(element) = elements_query.get(*entity) else { return; };

    if *element == Element::Air {
        commands.entity(*entity).despawn();

        // Drop sand on air
        let sand_entity = commands.spawn(SandElementBundle::new(position)).id();

        world_map.elements.insert(position, sand_entity);

        *behavior = AntBehavior::Wandering;
    }
}

fn do_turn(
    mut orientation: Mut<AntOrientation>,
    behavior: Mut<AntBehavior>,
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

    // No legal direction? Trapped! Drop sand and turn randomly in an attempt to dig out.
    if *behavior == AntBehavior::Carrying {
        if let Some(entity) = world_map.elements.get(&position) {
            // TODO: maybe this should exit early rather than allowing for turning since relevant state has been mutated
            // NOTE: this can occur due to `spawn` not affecting query on current frame
            if let Ok(element) = elements_query.get(*entity) {
                if *element == Element::Air {
                    do_drop(behavior, *position, elements_query, world_map, commands);
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
    mut behavior: Mut<AntBehavior>,
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

    let Some(entity) = world_map.elements.get(&dig_position) else { return };
    // NOTE: this can occur due to `spawn` not affecting query on current frame
    let Ok(element) = elements_query.get(*entity) else { return };

    if *element == Element::Dirt || *element == Element::Sand {
        commands.entity(*entity).despawn();

        // Dig up dirt/sand and replace with air
        let air_entity = commands.spawn(AirElementBundle::new(dig_position)).id();
        world_map.elements.insert(dig_position, air_entity);

        loosen_neighboring_sand(dig_position, world_map, elements_query, commands);
        *behavior = AntBehavior::Carrying;
    }
}

fn do_move(
    mut orientation: Mut<AntOrientation>,
    behavior: Mut<AntBehavior>,
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
            behavior,
            position,
            elements_query,
            world_map,
            world_rng,
            commands,
        );
        return;
    }

    // Check if hitting dirt or sand and, if so, consider digging through it.
    let entity = world_map.elements.get(&new_position).unwrap();
    // NOTE: this can occur due to `spawn` not affecting query on current frame
    let Ok(element) = elements_query.get(*entity) else { return };

    if *element == Element::Dirt || *element == Element::Sand {
        // If ant is wandering *below ground level* and bumps into sand or has a chance to dig, dig.
        if *behavior == AntBehavior::Wandering
            && position.y > *world_map.surface_level()
            && (*element == Element::Sand
                || world_rng.0.gen::<f32>() < settings.probabilities.below_surface_dig)
        {
            do_dig(
                true,
                behavior,
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
                behavior,
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

    if let Some(foot_entity) = world_map.elements.get(&foot_position) {
        // NOTE: this can occur due to `spawn` not affecting query on current frame
        let Ok(foot_element) = elements_query.get(*foot_entity) else { return };

        if *foot_element == Element::Air {
            // If ant moves straight forward, it will be standing over air. Instead, turn into the air and remain standing on current block
            // Ant will try to fill the gap with sand if possible.
            let should_drop_sand = *behavior == AntBehavior::Carrying
                && position.y <= *world_map.surface_level()
                && world_rng.0.gen::<f32>() < settings.probabilities.above_surface_drop;

            if should_drop_sand {
                do_drop(behavior, *position, &elements_query, world_map, commands);
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

// TODO: untangle mutability, many functions accept mutable but do not mutate which seems wrong
// TODO: first pass is going to be just dumping all code into one system, but suspect want multiple systems
pub fn move_ants_system(
    mut ants_query: Query<(
        &mut AntOrientation,
        &mut AntBehavior,
        &AntTimer,
        &mut Position,
    )>,
    elements_query: Query<&Element>,
    mut world_map: ResMut<WorldMap>,
    settings: Res<Settings>,
    mut world_rng: ResMut<WorldRng>,
    mut commands: Commands,
) {
    for (orientation, behavior, timer, position) in ants_query.iter_mut() {
        if timer.0 > 0 {
            continue;
        }

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
                behavior,
                position,
                &elements_query,
                &mut world_map,
                &mut world_rng,
                &mut commands,
            );

            continue;
        }

        match *behavior {
            AntBehavior::Wandering => {
                if world_rng.0.gen::<f32>() < settings.probabilities.random_dig {
                    do_dig(
                        false,
                        behavior,
                        orientation,
                        position,
                        &elements_query,
                        &mut world_map,
                        &mut commands,
                    )
                } else if world_rng.0.gen::<f32>() < settings.probabilities.random_turn {
                    do_turn(
                        orientation,
                        behavior,
                        position,
                        &elements_query,
                        &mut world_map,
                        &mut world_rng,
                        &mut commands,
                    );
                } else {
                    do_move(
                        orientation,
                        behavior,
                        position,
                        &elements_query,
                        &mut world_map,
                        &mut world_rng,
                        &settings,
                        &mut commands,
                    );
                }
            }
            AntBehavior::Carrying => {
                if world_rng.0.gen::<f32>() < settings.probabilities.random_drop {
                    do_drop(
                        behavior,
                        *position,
                        &elements_query,
                        &mut world_map,
                        &mut commands,
                    );
                } else {
                    do_move(
                        orientation,
                        behavior,
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
}

pub fn update_ant_timer_system(
    mut ants_query: Query<(Ref<AntBehavior>, &mut AntTimer)>,
    mut world_rng: ResMut<WorldRng>,
) {
    for (behavior, mut timer) in ants_query.iter_mut() {
        if behavior.is_changed() {
            *timer = AntTimer::new(&behavior, &mut world_rng.0);
        } else if timer.0 > 0 {
            timer.0 -= 1;
        }
    }
}

// TODO: tests