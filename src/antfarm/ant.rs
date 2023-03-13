use bevy::prelude::*;

use super::{elements::ElementBundle, settings::Settings, Root};

// TODO: Add support for behavior timer.
// TODO: Add support for dynamic names.

#[derive(Bundle)]
pub struct AntSpriteBundle {
    sprite_bundle: SpriteBundle,
    facing: AntFacing,
    angle: AntAngle,
    behavior: AntBehavior,
}

#[derive(Bundle)]
pub struct AntLabelBundle {
    text_bundle: Text2dBundle,
}

#[derive(Component)]
pub struct Ant;

impl AntLabelBundle {
    pub fn new(label: String, asset_server: &Res<AssetServer>) -> Self {
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

impl AntSpriteBundle {
    pub fn new(
        color: Color,
        facing: AntFacing,
        angle: AntAngle,
        behavior: AntBehavior,
        asset_server: &Res<AssetServer>,
    ) -> Self {
        // TODO: is this a bad architectural decision? technically I am thinking about mirroring improperly by inverting angle when x is flipped?
        let x_flip = if facing == AntFacing::Left { -1.0 } else { 1.0 };

        let angle_radians = angle as u32 as f32 * std::f32::consts::PI / 180.0 * x_flip;
        let rotation = Quat::from_rotation_z(angle_radians);

        Self {
            sprite_bundle: SpriteBundle {
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
            facing,
            angle,
            behavior,
        }
    }
}

#[derive(Component, PartialEq)]
pub enum AntBehavior {
    Wandering,
    Carrying,
}

#[derive(Component, PartialEq)]
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

// Spawn non-interactive background (sky blue / tunnel brown)
fn setup(
    mut commands: Commands,
    query: Query<Entity, With<Root>>,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
) {
    let Some(mut entity_commands) = commands.get_entity(query.single()) else { panic!("root missing") };

    entity_commands.with_children(|parent| {
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
            (
                Vec3::new(5.0, -5.0, 100.0),
                AntSpriteBundle::new(
                    settings.ant_color,
                    AntFacing::Left,
                    AntAngle::Zero,
                    AntBehavior::Carrying,
                    &asset_server,
                ),
                AntLabelBundle::new("ant1".to_string(), &asset_server),
            ),
            (
                Vec3::new(10.0, -5.0, 1.0),
                AntSpriteBundle::new(
                    settings.ant_color,
                    AntFacing::Left,
                    AntAngle::Ninety,
                    AntBehavior::Carrying,
                    &asset_server,
                ),
                AntLabelBundle::new("ant2".to_string(), &asset_server),
            ),
            (
                Vec3::new(15.0, -5.0, 1.0),
                AntSpriteBundle::new(
                    settings.ant_color,
                    AntFacing::Left,
                    AntAngle::OneHundredEighty,
                    AntBehavior::Carrying,
                    &asset_server,
                ),
                AntLabelBundle::new("ant3".to_string(), &asset_server),
            ),
            (
                Vec3::new(20.0, -5.0, 1.0),
                AntSpriteBundle::new(
                    settings.ant_color,
                    AntFacing::Left,
                    AntAngle::TwoHundredSeventy,
                    AntBehavior::Carrying,
                    &asset_server,
                ),
                AntLabelBundle::new("ant4".to_string(), &asset_server),
            ),
            (
                Vec3::new(25.0, -5.0, 1.0),
                AntSpriteBundle::new(
                    settings.ant_color,
                    AntFacing::Right,
                    AntAngle::Zero,
                    AntBehavior::Carrying,
                    &asset_server,
                ),
                AntLabelBundle::new("ant5".to_string(), &asset_server),
            ),
            (
                Vec3::new(30.0, -5.0, 1.0),
                AntSpriteBundle::new(
                    settings.ant_color,
                    AntFacing::Right,
                    AntAngle::Ninety,
                    AntBehavior::Carrying,
                    &asset_server,
                ),
                AntLabelBundle::new("ant6".to_string(), &asset_server),
            ),
            (
                Vec3::new(35.0, -5.0, 1.0),
                AntSpriteBundle::new(
                    settings.ant_color,
                    AntFacing::Right,
                    AntAngle::OneHundredEighty,
                    AntBehavior::Carrying,
                    &asset_server,
                ),
                AntLabelBundle::new("ant7".to_string(), &asset_server),
            ),
            (
                Vec3::new(40.0, -5.0, 1.0),
                AntSpriteBundle::new(
                    settings.ant_color,
                    AntFacing::Right,
                    AntAngle::TwoHundredSeventy,
                    AntBehavior::Carrying,
                    &asset_server,
                ),
                AntLabelBundle::new("ant8".to_string(), &asset_server),
            ),
        ];

        for ant_bundle in test_ant_bundles {
            let is_carrying = ant_bundle.1.behavior == AntBehavior::Carrying;

            parent
                // Wrap label and ant with common parent to associate their movement, but not their rotation.
                .spawn((
                    SpatialBundle {
                        transform: Transform {
                            translation: ant_bundle.0,
                            ..default()
                        },
                        ..default()
                    },
                    Ant,
                ))
                .with_children(|parent| {
                    // Make sand a child of ant so they share rotation.
                    parent.spawn(ant_bundle.1).with_children(|parent| {
                        if is_carrying {
                            // NOTE: sand carried by ants is not "affected by gravity" intentionally
                            // There might need to be a better way of handling this once ant gravity is implemented
                            parent.spawn(ElementBundle::create_sand(Vec3::new(0.5, 0.5, 0.0)));
                        }
                    });
                    parent.spawn(ant_bundle.2);
                });
        }
    });
}

// TODO: This is probably too aggressive of a plugin architecture, but it's good for practice
impl Plugin for AntsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}
