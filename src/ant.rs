use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, ops::Add};

use crate::{
    elements::{is_all_element, ElementBundle},
    map::{Position, WorldMap},
    world_rng::WorldRng,
};

use super::{elements::Element, settings::Settings};
use bevy::{prelude::*, sprite::Anchor};
use rand::Rng;

// TODO: Add support for behavior timer.

pub fn get_delta(facing: AntFacing, angle: AntAngle) -> Position {
    let delta = match angle {
        AntAngle::Zero => Position::X,
        AntAngle::Ninety => Position::NEG_Y,
        AntAngle::OneHundredEighty => Position::NEG_X,
        AntAngle::TwoHundredSeventy => Position::Y,
    };

    if facing == AntFacing::Left {
        delta * Position::NEG_ONE
    } else {
        delta
    }
}

/**
 * Rotation is a value from 0 to 3. A value of 1 is a 90 degree counter-clockwise rotation. Negative values are accepted.
 * Examples:
 *  getRotatedAngle(0, -1); // 270
 *  getRotatedAngle(0, 1); // 90
 */
pub fn get_rotated_angle(angle: AntAngle, rotation: i32) -> AntAngle {
    let angles = [
        AntAngle::Zero,
        AntAngle::Ninety,
        AntAngle::OneHundredEighty,
        AntAngle::TwoHundredSeventy,
    ];

    let rotated_index =
        (angles.iter().position(|&a| a == angle).unwrap() as i32 - rotation) % angles.len() as i32;
    angles[((rotated_index + angles.len() as i32) % angles.len() as i32) as usize]
}

// This is what is persisted as JSON.
#[derive(Serialize, Deserialize, Debug)]
pub struct AntSaveState {
    pub position: Position,
    pub color: AntColor,
    pub facing: AntFacing,
    pub angle: AntAngle,
    pub behavior: AntBehavior,
    pub name: AntName,
}

// TODO: This seems like an anti-pattern, it's sort of like a bundle, but it needs parent/child relationships so it can't just be spawned as a bundle itself
struct Ant {
    position: Position,
    transform_offset: TransformOffset,
    facing: AntFacing,
    angle: AntAngle,
    behavior: AntBehavior,
    name: AntName,
    color: AntColor,
    sprite_bundle: SpriteBundle,
    label_bundle: Text2dBundle,
}

impl Ant {
    pub fn new(
        position: Position,
        color: Color,
        facing: AntFacing,
        angle: AntAngle,
        behavior: AntBehavior,
        name: &str,
        asset_server: &Res<AssetServer>,
    ) -> Self {
        let transform_offset = TransformOffset(Vec3::new(0.5, -0.5, 0.0));
        let x_flip = if facing == AntFacing::Left { -1.0 } else { 1.0 };
        let translation = Vec3::new(position.x as f32, -position.y as f32, 1.0);

        Self {
            position,
            transform_offset,
            facing,
            angle,
            behavior,
            name: AntName(name.to_string()),
            color: AntColor(color),
            sprite_bundle: SpriteBundle {
                // TODO: alient-cake-addict creates a handle on a resource for this instead
                texture: asset_server.load("images/ant.png"),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(ANT_SCALE, ANT_SCALE)),
                    ..default()
                },
                transform: Transform {
                    translation: translation.add(transform_offset.0),
                    rotation: Quat::from_rotation_z(angle.as_radians()),
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

// TODO: copy cant be implemented for String?
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
pub enum AntFacing {
    Left,
    Right,
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum AntAngle {
    Zero = 0,
    Ninety = 90,
    OneHundredEighty = 180,
    TwoHundredSeventy = 270,
}

impl AntAngle {
    pub fn as_radians(&self) -> f32 {
        (*self as isize as f32) * PI / 180.0
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
) {
    let ants = world_map
        .initial_state
        .ants
        .iter()
        .map(|ant_save_state| {
            Ant::new(
                ant_save_state.position,
                settings.ant_color,
                ant_save_state.facing,
                ant_save_state.angle,
                ant_save_state.behavior,
                ant_save_state.name.0.as_str(),
                &asset_server,
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
                        ant.facing,
                        ant.angle,
                        ant.behavior,
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
    facing: AntFacing,
    angle: AntAngle,
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
    let rotation = if facing == AntFacing::Left { -1 } else { 1 };
    let foot_position = position + get_delta(facing, get_rotated_angle(angle, rotation));
    // TODO: returning true here seems awkward, but I want the ants to climb the sides of the container?
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
        let sand_entity = commands
            .spawn(ElementBundle::create(Element::Sand, position))
            .id();

        world_map.elements.insert(position, sand_entity);

        // TODO: need to update timer
        *behavior = AntBehavior::Wandering;
    }
}

fn do_turn(
    mut facing: Mut<AntFacing>,
    mut angle: Mut<AntAngle>,
    behavior: Mut<AntBehavior>,
    position: Mut<Position>,
    elements_query: &Query<&Element>,
    world_map: &mut ResMut<WorldMap>,
    world_rng: &mut ResMut<WorldRng>,
    commands: &mut Commands,
) {
    // First try turning perpendicularly towards the ant's back. If that fails, try turning around.
    let rotation = if *facing == AntFacing::Left { 1 } else { -1 };
    let back_angle = get_rotated_angle(*angle, rotation);
    if is_valid_location(*facing, back_angle, *position, elements_query, world_map) {
        *angle = back_angle;
        return;
    }

    let opposite_facing = if *facing == AntFacing::Left {
        AntFacing::Right
    } else {
        AntFacing::Left
    };

    if is_valid_location(
        opposite_facing,
        *angle,
        *position,
        elements_query,
        world_map,
    ) {
        *facing = opposite_facing;
        return;
    }

    // Randomly turn in a valid different when unable to simply turn around.
    let facings = [AntFacing::Left, AntFacing::Right];
    let angles = [
        AntAngle::Zero,
        AntAngle::Ninety,
        AntAngle::OneHundredEighty,
        AntAngle::TwoHundredSeventy,
    ];
    let facing_angles = facings
        .iter()
        .flat_map(|facing| angles.iter().map(move |angle| (*facing, *angle)))
        .collect::<Vec<_>>();

    let valid_facing_angles = facing_angles
        .iter()
        .filter(|(inner_facing, inner_angle)| {
            if *inner_facing == *facing && *inner_angle == *angle {
                return false;
            }

            is_valid_location(
                *inner_facing,
                *inner_angle,
                *position,
                elements_query,
                world_map,
            )
        })
        .collect::<Vec<_>>();

    if valid_facing_angles.len() > 0 {
        let valid_facing_angle =
            valid_facing_angles[world_rng.rng.gen_range(0..valid_facing_angles.len())];

        *facing = valid_facing_angle.0;
        *angle = valid_facing_angle.1;
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

    let random_facing_angle = facing_angles[world_rng.rng.gen_range(0..facing_angles.len())];
    *facing = random_facing_angle.0;
    *angle = random_facing_angle.1;
}

fn do_dig(
    is_forced_forward: bool,
    mut behavior: Mut<AntBehavior>,
    facing: Mut<AntFacing>,
    angle: Mut<AntAngle>,
    position: Mut<Position>,
    elements_query: &Query<&Element>,
    world_map: &mut ResMut<WorldMap>,
    commands: &mut Commands,
) {
    let dig_angle = if is_forced_forward {
        *angle
    } else {
        let rotation = if *facing == AntFacing::Left { -1 } else { 1 };
        get_rotated_angle(*angle, rotation)
    };

    let dig_position = *position + get_delta(*facing, dig_angle);

    let Some(entity) = world_map.elements.get(&dig_position) else { return };
    // NOTE: this can occur due to `spawn` not affecting query on current frame
    let Ok(element) = elements_query.get(*entity) else { return };

    if *element == Element::Dirt || *element == Element::Sand {
        commands.entity(*entity).despawn();

        // Dig up dirt/sand and replace with air
        let air_entity = commands
            .spawn(ElementBundle::create(Element::Air, dig_position))
            .id();

        world_map.elements.insert(dig_position, air_entity);

        // TODO: timer
        *behavior = AntBehavior::Carrying;
    }
}

// TODO: repent for my sins. I'm just throwing & and &mut everywhere to get this shit to compile lol.
fn do_move(
    facing: Mut<AntFacing>,
    mut angle: Mut<AntAngle>,
    behavior: Mut<AntBehavior>,
    mut position: Mut<Position>,
    elements_query: &Query<&Element>,
    world_map: &mut ResMut<WorldMap>,
    world_rng: &mut ResMut<WorldRng>,
    settings: &Res<Settings>,
    commands: &mut Commands,
) {
    let delta = get_delta(*facing, *angle);
    let new_position = *position + delta;

    if !world_map.is_within_bounds(&new_position) {
        // Hit an edge - need to turn.
        do_turn(
            facing,
            angle,
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
    let target_entity = world_map.elements.get(&new_position).unwrap();
    // NOTE: this can occur due to `spawn` not affecting query on current frame
    let Ok(target_element) = elements_query.get(*target_entity) else { return };

    if *target_element == Element::Dirt || *target_element == Element::Sand {
        // If ant is wandering *below ground level* and bumps into sand or has a chance to dig, dig.
        if *behavior == AntBehavior::Wandering
            && position.y > *world_map.surface_level()
            && (*target_element == Element::Sand
                || world_rng.rng.gen::<f32>() < settings.probabilities.below_surface_dig)
        {
            do_dig(
                false,
                behavior,
                facing,
                angle,
                position,
                &elements_query,
                world_map,
                commands,
            );
            return;
        } else {
            do_turn(
                facing,
                angle,
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
    let rotation = if *facing == AntFacing::Left { -1 } else { 1 };
    let target_foot_angle = get_rotated_angle(*angle, rotation);
    let target_foot_position = new_position + get_delta(*facing, target_foot_angle);

    if let Some(target_foot_entity) = world_map.elements.get(&target_foot_position) {
        // NOTE: this can occur due to `spawn` not affecting query on current frame
        let Ok(target_foot_element) = elements_query.get(*target_foot_entity) else { return };

        if *target_foot_element == Element::Air {
            // If ant moves straight forward, it will be standing over air. Instead, turn into the air and remain standing on current block
            // Ant will try to fill the gap with sand if possible.

            let should_drop_sand = *behavior == AntBehavior::Carrying
                && position.y <= *world_map.surface_level()
                && world_rng.rng.gen::<f32>() < settings.probabilities.above_surface_drop;

            if should_drop_sand {
                do_drop(behavior, *position, &elements_query, world_map, commands);
            } else {
                // Update position and angle
                *position = target_foot_position;
                *angle = target_foot_angle;
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
        &mut AntFacing,
        &mut AntAngle,
        &mut AntBehavior,
        &mut Position,
    )>,
    elements_query: Query<&Element>,
    mut world_map: ResMut<WorldMap>,
    settings: Res<Settings>,
    mut world_rng: ResMut<WorldRng>,
    mut commands: Commands,
) {
    for (facing, angle, behavior, position) in ants_query.iter_mut() {
        // TODO: prefer this not copy/pasted logic, should be able to easily check if there is air underneath a unit's feet
        let rotation = if *facing == AntFacing::Left { -1 } else { 1 };
        let foot_delta = get_delta(*facing, get_rotated_angle(*angle, rotation));
        let below_feet_position = *position + foot_delta;

        let is_air_beneath_feet = is_all_element(
            &world_map,
            &elements_query,
            &vec![below_feet_position],
            Element::Air,
        );

        if is_air_beneath_feet {
            // Whoops, whatever we were walking on disappeared.
            let below_position = *position + Position::Y;
            let is_air_below = is_all_element(
                &world_map,
                &elements_query,
                &vec![below_position],
                Element::Air,
            );

            // Gravity system will handle things if going to fall
            if is_air_below {
                continue;
            } else {
                // Not falling? Try turning

                do_turn(
                    facing,
                    angle,
                    behavior,
                    position,
                    &elements_query,
                    &mut world_map,
                    &mut world_rng,
                    &mut commands,
                );

                continue;
            }
        }

        match *behavior {
            AntBehavior::Wandering => {
                // Wandering:
                // - Update timer to reflect behavior change
                // - Maybe behavior should be its own system?

                if world_rng.rng.gen::<f32>() < settings.probabilities.random_dig {
                    do_dig(
                        false,
                        behavior,
                        facing,
                        angle,
                        position,
                        &elements_query,
                        &mut world_map,
                        &mut commands,
                    )
                } else if world_rng.rng.gen::<f32>() < settings.probabilities.random_turn {
                    do_turn(
                        facing,
                        angle,
                        behavior,
                        position,
                        &elements_query,
                        &mut world_map,
                        &mut world_rng,
                        &mut commands,
                    );
                } else {
                    do_move(
                        facing,
                        angle,
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
                if world_rng.rng.gen::<f32>() < settings.probabilities.random_drop {
                    do_drop(
                        behavior,
                        *position,
                        &elements_query,
                        &mut world_map,
                        &mut commands,
                    );
                } else {
                    do_move(
                        facing,
                        angle,
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

// TODO: tests
