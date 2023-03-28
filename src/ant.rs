use crate::map::Position;

use super::{elements::Element, settings::Settings};
use bevy::{prelude::*, sprite::Anchor};

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

#[derive(Component, PartialEq, Copy, Clone)]
pub enum AntFacing {
    Left,
    Right,
}

#[derive(Component, PartialEq, Copy, Clone)]
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
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, settings: Res<Settings>) {
    // let ant_bundles = (0..8).map(|_| {
    //     // Put the ant at a random location along the x-axis that fits within the bounds of the world.
    //     // TODO: technically old code was .round() and now it's just floored implicitly
    //     let x = rand::thread_rng().gen_range(0..1000) as f32 % WORLD_WIDTH as f32;
    //     // Put the ant on the dirt.
    //     let y = SURFACE_LEVEL as f32;

    //     // Randomly position ant facing left or right.
    //     let facing = if rand::thread_rng().gen_range(0..10) < 5 {
    //         Facing::Left
    //     } else {
    //         Facing::Right
    //     };

    //     (
    //         Vec3::new(x, -y, 0.0),
    //         AntSpriteBundle::new(
    //             settings.ant_color,
    //             facing,
    //             Angle::Zero,
    //             Behavior::Wandering,
    //             &asset_server,
    //         ),
    //         AntLabelBundle::new("Test Name".to_string(), &asset_server),
    //     )
    // });

    let test_ant_bundles = [
        create_ant(
            Position::new(5, 5),
            settings.ant_color,
            AntFacing::Left,
            AntAngle::Zero,
            AntBehavior::Carrying,
            "ant1".to_string(),
            &asset_server,
        ),
        create_ant(
            Position::new(10, 5),
            settings.ant_color,
            AntFacing::Left,
            AntAngle::Ninety,
            AntBehavior::Carrying,
            "ant2".to_string(),
            &asset_server,
        ),
        create_ant(
            Position::new(15, 5),
            settings.ant_color,
            AntFacing::Left,
            AntAngle::OneHundredEighty,
            AntBehavior::Carrying,
            "ant3".to_string(),
            &asset_server,
        ),
        create_ant(
            Position::new(20, 5),
            settings.ant_color,
            AntFacing::Left,
            AntAngle::TwoHundredSeventy,
            AntBehavior::Carrying,
            "ant4".to_string(),
            &asset_server,
        ),
        create_ant(
            Position::new(25, 5),
            settings.ant_color,
            AntFacing::Right,
            AntAngle::Zero,
            AntBehavior::Carrying,
            "ant5".to_string(),
            &asset_server,
        ),
        create_ant(
            Position::new(30, 5),
            settings.ant_color,
            AntFacing::Right,
            AntAngle::Ninety,
            AntBehavior::Carrying,
            "ant6".to_string(),
            &asset_server,
        ),
        create_ant(
            Position::new(35, 5),
            settings.ant_color,
            AntFacing::Right,
            AntAngle::OneHundredEighty,
            AntBehavior::Carrying,
            "ant7".to_string(),
            &asset_server,
        ),
        create_ant(
            Position::new(40, 5),
            settings.ant_color,
            AntFacing::Right,
            AntAngle::TwoHundredSeventy,
            AntBehavior::Carrying,
            "ant8".to_string(),
            &asset_server,
        ),
    ];

    for (position, facing, angle, behavior, sprite, label) in test_ant_bundles {
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

impl Plugin for AntsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}
