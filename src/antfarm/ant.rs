use bevy::prelude::*;

// TODO: Add support for behavior timer.
// TODO: Add support for dynamic names.

#[derive(Bundle)]
pub struct AntSpriteBundle {
    sprite_bundle: SpriteBundle,
    // TODO: is it a code smell that these are pub?
    pub facing: AntFacing,
    pub angle: AntAngle,
    pub behavior: AntBehavior,
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
        let angle_degrees = match angle {
            AntAngle::Zero => 0,
            AntAngle::Ninety => 90,
            AntAngle::OneHundredEighty => 180,
            AntAngle::TwoHundredSeventy => 270,
        };
        // TODO: is this a bad architectural decision? technically I am thinking about mirroring improperly by inverting angle when x is flipped?
        let x_flip = if facing == AntFacing::Left { -1.0 } else { 1.0 };

        let angle_radians = angle_degrees as f32 * std::f32::consts::PI / 180.0 * x_flip;
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

// TODO: it's awkward that these aren't numbers and maybe would be better to use radians instead of degrees?
#[derive(Component, PartialEq)]
pub enum AntAngle {
    Zero,
    Ninety,
    OneHundredEighty,
    TwoHundredSeventy,
}
