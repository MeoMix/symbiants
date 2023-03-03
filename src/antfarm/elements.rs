use bevy::{prelude::*, sprite::Anchor};

use super::{
    get_surface_level, get_world_container_transform, settings::Settings, setup_background,
    Element, WorldContainer, WORLD_HEIGHT, WORLD_WIDTH,
};

#[derive(Component, Debug)]
pub struct Position {
    // (0,0) is the top-left corner of the viewport so these values can be represented unsigned
    pub x: usize,
    pub y: usize,
}

// NOTE: This is a two-dimensional array expressed in a single Vector.
// This is to allow for easier shorthand such as Vec.swap and Vec.get.
// Access via vec![y * WORLD_WIDTH + x]
#[derive(Component)]
pub struct Elements2D(pub Vec<Entity>);

#[derive(Component)]
pub struct AffectedByGravity;

#[derive(Bundle)]
pub struct ElementBundle {
    sprite_bundle: SpriteBundle,
    element: Element,
    position: Position,
}

impl ElementBundle {
    pub fn create_sand(position: Vec3, size: Option<Vec2>) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(position.x as f32, -position.y as f32, 1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.761, 0.698, 0.502),
                    anchor: Anchor::TopLeft,
                    custom_size: size,
                    ..default()
                },
                ..default()
            },
            element: Element::Sand,
            position: Position {
                x: position.x as usize,
                // TODO: terrifying that I am using negative sign then casting to usize
                y: -position.y as usize,
            },
        }
    }

    pub fn create_air(position: Vec3) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                // Air is transparent so reveal background
                visibility: Visibility::INVISIBLE,
                transform: Transform {
                    translation: position,
                    ..default()
                },
                sprite: Sprite {
                    // Fully transparent color - could in theory set to something if air was made visible.
                    color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
            element: Element::Air,
            position: Position {
                x: position.x as usize,
                // TODO: terrifying that I am using negative sign then casting to usize
                y: -position.y as usize,
            },
        }
    }

    pub fn create_dirt(position: Vec3) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: position,
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.514, 0.396, 0.224),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
            element: Element::Dirt,
            position: Position {
                x: position.x as usize,
                // TODO: terrifying that I am using negative sign then casting to usize
                y: -position.y as usize,
            },
        }
    }
}

fn create_elements(mut commands: Commands, windows: Res<Windows>, settings: Res<Settings>) {
    // TODO: this feels super wrong
    // Wrap in container and shift to top-left viewport so 0,0 is top-left corner.
    let (translation, scale) = get_world_container_transform(&windows.get_primary().unwrap());
    let world_container_bundle = SpatialBundle {
        transform: Transform {
            translation,
            scale,
            ..default()
        },
        ..default()
    };

    commands
        .spawn((world_container_bundle, WorldContainer))
        .with_children(|parent| {
            setup_background(parent, &settings);

            // Spawn interactive elements - air/dirt. Air isn't visible, background is revealed in its place.
            let mut element_vector_2d = Vec::with_capacity((WORLD_HEIGHT * WORLD_WIDTH) as usize);

            // Test Sand
            let sand_bundles = (0..1).flat_map(|row_index| {
                (0..WORLD_WIDTH).map(move |column_index| {
                    // NOTE: row_index goes negative because 0,0 is top-left corner
                    (
                        ElementBundle::create_sand(
                            Vec3::new(column_index as f32, -(row_index as f32), 1.0),
                            Some(Vec2::ONE),
                        ),
                        AffectedByGravity,
                    )
                })
            });

            for sand_bundle in sand_bundles {
                element_vector_2d.push(parent.spawn(sand_bundle).id());
            }

            // Air & Dirt
            // NOTE: starting at 1 to skip sand
            let air_bundles = (1..get_surface_level(&settings) + 1).flat_map(|row_index| {
                (0..WORLD_WIDTH).map(move |column_index| {
                    // NOTE: row_index goes negative because 0,0 is top-left corner
                    ElementBundle::create_air(Vec3::new(
                        column_index as f32,
                        -(row_index as f32),
                        1.0,
                    ))
                })
            });

            for air_bundle in air_bundles {
                element_vector_2d.push(parent.spawn(air_bundle).id());
            }

            let dirt_bundles =
                ((get_surface_level(&settings) + 1)..WORLD_HEIGHT).flat_map(|row_index| {
                    (0..WORLD_WIDTH).map(move |column_index| {
                        // NOTE: row_index goes negative because 0,0 is top-left corner
                        ElementBundle::create_dirt(Vec3::new(
                            column_index as f32,
                            -(row_index as f32),
                            1.0,
                        ))
                    })
                });

            for dirt_bundle in dirt_bundles {
                element_vector_2d.push(parent.spawn(dirt_bundle).id());
            }

            parent.spawn(Elements2D(element_vector_2d));
        });
}

pub struct ElementsPlugin;
impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_elements);
    }
}
