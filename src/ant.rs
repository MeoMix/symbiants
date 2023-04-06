use crate::{
    elements::{is_all_element, ElementBundle},
    map::{Position, WorldMap},
    world_rng::{self, WorldRng},
};

use super::{elements::Element, settings::Settings};
use bevy::{prelude::*, sprite::Anchor};
use rand::Rng;

// TODO: Add support for behavior timer.
// TODO: Add support for dynamic names.

// TODO: get_delta should probably not be coupled to 'facing'?
pub fn get_delta(facing: AntFacing, angle: AntAngle) -> Position {
    match angle {
        AntAngle::Zero | AntAngle::OneHundredEighty => match facing {
            AntFacing::Right => Position {
                x: if angle == AntAngle::Zero { 1 } else { -1 },
                y: 0,
            },
            AntFacing::Left => Position {
                x: if angle == AntAngle::Zero { -1 } else { 1 },
                y: 0,
            },
        },
        _ => Position {
            x: 0,
            y: if angle == AntAngle::Ninety { -1 } else { 1 },
        },
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

#[derive(Bundle)]
struct AntLabelBundle {
    text_bundle: Text2dBundle,
}

impl AntLabelBundle {
    fn new(label: String, asset_server: &Res<AssetServer>) -> Self {
        Self {
            text_bundle: Text2dBundle {
                transform: Transform {
                    translation: Vec3::new(-ANT_SCALE / 4.0, -1.5, 100.0),
                    scale: Vec3::new(0.05, 0.05, 0.0),
                    ..default()
                },
                text: Text::from_section(
                    label,
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

#[derive(Bundle)]
struct AntSpriteBundle {
    sprite_bundle: SpriteBundle,
}

impl AntSpriteBundle {
    fn new(
        color: Color,
        facing: AntFacing,
        angle: AntAngle,
        asset_server: &Res<AssetServer>,
    ) -> Self {
        // TODO: is this a bad architectural decision? technically I am thinking about mirroring improperly by inverting angle when x is flipped?
        let x_flip = if facing == AntFacing::Left { -1.0 } else { 1.0 };

        let angle_radians = angle as u32 as f32 * std::f32::consts::PI / 180.0 * x_flip;
        let rotation = Quat::from_rotation_z(angle_radians);

        Self {
            sprite_bundle: SpriteBundle {
                // TODO: alient-cake-addict creates a handle on a resource for this instead
                texture: asset_server.load("images/ant.png"),
                transform: Transform {
                    rotation,
                    scale: Vec3::new(x_flip, 1.0, 1.0),
                    translation: Vec3::new(0.5, -0.5, 100.0),
                    ..default()
                },
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(ANT_SCALE, ANT_SCALE)),
                    ..default()
                },
                ..default()
            },
        }
    }
}

#[derive(Component, PartialEq, Copy, Clone)]
enum AntBehavior {
    Wandering,
    Carrying,
}

#[derive(Component, Debug, PartialEq, Copy, Clone)]
pub enum AntFacing {
    Left,
    Right,
}

#[derive(Component, Debug, PartialEq, Copy, Clone)]
pub enum AntAngle {
    Zero = 0,
    Ninety = 90,
    OneHundredEighty = 180,
    TwoHundredSeventy = 270,
}

pub struct AntsPlugin;

fn create_ant(
    position: Position,
    color: Color,
    facing: AntFacing,
    angle: AntAngle,
    behavior: AntBehavior,
    name: String,
    asset_server: &Res<AssetServer>,
) -> (
    Position,
    AntFacing,
    AntAngle,
    AntBehavior,
    AntSpriteBundle,
    AntLabelBundle,
) {
    (
        position,
        facing,
        angle,
        behavior,
        AntSpriteBundle::new(color, facing, angle, &asset_server),
        AntLabelBundle::new(name, &asset_server),
    )
}

// Spawn non-interactive background (sky blue / tunnel brown)
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
    world_map: ResMut<WorldMap>,
    mut world_rng: ResMut<WorldRng>,
) {
    let ant_bundles = (0..8).map(|_| {
        // Put the ant at a random location along the x-axis that fits within the bounds of the world.
        // TODO: technically old code was .round() and now it's just floored implicitly
        let x = world_rng.rng.gen_range(0..1000) % world_map.width();
        // Put the ant on the dirt.
        let &y = world_map.surface_level();

        // Randomly position ant facing left or right.
        let facing = if rand::thread_rng().gen_range(0..10) < 5 {
            AntFacing::Left
        } else {
            AntFacing::Right
        };

        create_ant(
            Position::new(x, y),
            settings.ant_color,
            facing,
            AntAngle::Zero,
            AntBehavior::Wandering,
            "Test Name".to_string(),
            &asset_server,
        )
    });

    // let test_ant_bundles = [
    //     create_ant(
    //         Position::new(5, 5),
    //         settings.ant_color,
    //         AntFacing::Left,
    //         AntAngle::Zero,
    //         AntBehavior::Carrying,
    //         "ant1".to_string(),
    //         &asset_server,
    //     ),
    // create_ant(
    //     Position::new(10, 5),
    //     settings.ant_color,
    //     AntFacing::Left,
    //     AntAngle::Ninety,
    //     AntBehavior::Carrying,
    //     "ant2".to_string(),
    //     &asset_server,
    // ),
    // create_ant(
    //     Position::new(15, 5),
    //     settings.ant_color,
    //     AntFacing::Left,
    //     AntAngle::OneHundredEighty,
    //     AntBehavior::Carrying,
    //     "ant3".to_string(),
    //     &asset_server,
    // ),
    // create_ant(
    //     Position::new(20, 5),
    //     settings.ant_color,
    //     AntFacing::Left,
    //     AntAngle::TwoHundredSeventy,
    //     AntBehavior::Carrying,
    //     "ant4".to_string(),
    //     &asset_server,
    // ),
    // create_ant(
    //     Position::new(25, 5),
    //     settings.ant_color,
    //     AntFacing::Right,
    //     AntAngle::Zero,
    //     AntBehavior::Carrying,
    //     "ant5".to_string(),
    //     &asset_server,
    // ),
    // create_ant(
    //     Position::new(30, 5),
    //     settings.ant_color,
    //     AntFacing::Right,
    //     AntAngle::Ninety,
    //     AntBehavior::Carrying,
    //     "ant6".to_string(),
    //     &asset_server,
    // ),
    // create_ant(
    //     Position::new(35, 5),
    //     settings.ant_color,
    //     AntFacing::Right,
    //     AntAngle::OneHundredEighty,
    //     AntBehavior::Carrying,
    //     "ant7".to_string(),
    //     &asset_server,
    // ),
    // create_ant(
    //     Position::new(40, 5),
    //     settings.ant_color,
    //     AntFacing::Right,
    //     AntAngle::TwoHundredSeventy,
    //     AntBehavior::Carrying,
    //     "ant8".to_string(),
    //     &asset_server,
    // ),
    // ];

    for (position, facing, angle, behavior, sprite, label) in ant_bundles {
        // The view of the model position is just an inversion along the y-axis.
        let translation = Vec3::new(position.x as f32, -position.y as f32, 1.0);
        let is_carrying = behavior == AntBehavior::Carrying;

        commands
            // Wrap label and ant with common parent to associate their movement, but not their rotation.
            .spawn((
                SpatialBundle {
                    transform: Transform {
                        translation,
                        ..default()
                    },
                    ..default()
                },
                position,
                facing,
                angle,
                behavior,
            ))
            .with_children(|parent| {
                // Make sand a child of ant so they share rotation.
                parent.spawn(sprite).with_children(|parent| {
                    if is_carrying {
                        // NOTE: sand carried by ants is not "affected by gravity" intentionally
                        // There might need to be a better way of handling this once ant gravity is implemented
                        // TODO: It seems like this logic should share ElementBundle create_sand but need to re-think position
                        // otherwise maybe this shouldn't be a true SandSprite and instead be a sprite change on the ant itself?
                        parent.spawn((
                            SpriteBundle {
                                transform: Transform {
                                    translation: Vec3::new(0.5, 0.5, 0.0),
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
                parent.spawn(label);
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
    let element = elements_query.get(*entity).unwrap();

    if *element != Element::Air {
        return false;
    }

    // Get the location beneath the ants' feet and check for air
    let foot_position = position + get_delta(facing, get_rotated_angle(angle, 1));
    // TODO: returning true here seems awkward, but I want the ants to climb the sides of the container?
    let Some(entity) = world_map.elements.get(&foot_position) else { return false };
    let element = elements_query.get(*entity).unwrap();

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
    let element = elements_query.get(*entity).unwrap();

    if *element == Element::Air {
        // TODO: it seems like I should do this...
        // commands.entity(*entity).despawn();

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
    mut behavior: Mut<AntBehavior>,
    mut position: Mut<Position>,
    elements_query: &Query<&Element>,
    world_map: &mut ResMut<WorldMap>,
    world_rng: &mut ResMut<WorldRng>,
    commands: &mut Commands,
) {
    // First try turning perpendicularly towards the ant's back. If that fails, try turning around.
    let back_angle = get_rotated_angle(*angle, -1);
    if is_valid_location(*facing, back_angle, *position, elements_query, world_map) {
        *angle = back_angle;
        return;
    }

    // TODO: discovered this code was wrong when porting, want to flip facing not flip turn around angle
    // let turn_around_angle = get_rotated_angle(*angle, -2);
    // info!("turn_around_angle: {:?}", turn_around_angle);
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
        return;
    }

    // No legal direction? Trapped! Drop sand and turn randomly in an attempt to dig out.
    if *behavior == AntBehavior::Carrying {
        if let Some(entity) = world_map.elements.get(&position) {
            let element = elements_query.get(*entity).unwrap();

            if *element == Element::Air {
                do_drop(behavior, *position, elements_query, world_map, commands);
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
        get_rotated_angle(*angle, 1)
    };

    let dig_position = *position + get_delta(*facing, dig_angle);

    let Some(entity) = world_map.elements.get(&dig_position) else { return };
    let element = elements_query.get(*entity).unwrap();

    if *element == Element::Dirt || *element == Element::Sand {
        info!("digging NOW");
        // TODO: it seems like I should do this...
        // commands.entity(*entity).despawn();

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

    info!(
        "target_entity and position: {:?} {:?}",
        target_entity, new_position
    );

    let target_element = elements_query.get(*target_entity).unwrap();

    if *target_element == Element::Dirt || *target_element == Element::Sand {
        // If ant is wandering *below ground level* and bumps into sand or has a chance to dig, dig.
        if *behavior == AntBehavior::Wandering
            && position.y > *world_map.surface_level()
            && (*target_element == Element::Sand
                || world_rng.rng.gen::<f32>() < settings.probabilities.below_surface_dig)
        {
            info!("below surface dig");
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
            info!("above surface turn");
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
    let target_foot_angle = get_rotated_angle(*angle, 1);
    let target_foot_position = new_position + get_delta(*facing, target_foot_angle);

    if let Some(target_foot_entity) = world_map.elements.get(&target_foot_position) {
        info!(
            "target_foot_entity and position: {:?} {:?}",
            target_foot_entity, target_foot_position
        );

        let target_foot_element = elements_query.get(*target_foot_entity).unwrap();

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
fn move_ant(
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
        let foot_delta = get_delta(*facing, get_rotated_angle(*angle, 1));
        let below_feet_position = *position + foot_delta;

        let is_air_beneath_feet = is_all_element(
            &world_map,
            &elements_query,
            &vec![below_feet_position],
            Element::Air,
        );

        if is_air_beneath_feet {
            info!("is_air_beneath_feet");
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

                // TODO: This is tricky because it could mutate behavior which isn't OK
                // do_turn(
                //     facing,
                //     angle,
                //     behavior,
                //     position,
                //     &elements_query,
                //     &mut world_map,
                //     &mut world_rng,
                //     &mut commands,
                // );
            }
        }

        if *behavior == AntBehavior::Wandering {
            // Wandering:
            // - Update timer to reflect behavior change
            // - Maybe behavior should be its own system?

            if world_rng.rng.gen::<f32>() < settings.probabilities.random_dig {
                info!("dig!!!");
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
        } else if *behavior == AntBehavior::Carrying {
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
        } else {
            info!("error - unsupported behavior");
        }
    }
}

impl Plugin for AntsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
        // TODO: Does ordering of move_ant / gravity matter? If so, need to rearchitect to support
        app.add_system(move_ant.in_schedule(CoreSchedule::FixedUpdate));
    }
}
