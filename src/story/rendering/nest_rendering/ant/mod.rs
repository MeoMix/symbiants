use std::ops::Add;

use crate::{
    settings::Settings,
    story::{
        ant::{
            emote::{Emote, EmoteType},
            sleep::Asleep,
            Ant, AntAteFoodEvent, AntColor, AntInventory, AntName, AntOrientation, AntRole, Dead,
        },
        common::position::Position,
        element::{Element, ElementExposure},
        grid::Grid,
        rendering::{
            common::{ModelViewEntityMap, VisibleGrid},
            nest_rendering::element::{get_element_index, ElementTextureAtlasHandle},
        },
        simulation::nest_simulation::nest::{AtNest, Nest},
        story_time::DEFAULT_TICKS_PER_SECOND,
    },
};
use bevy::{prelude::*, utils::HashSet};
use bevy_turborand::{DelegatedRng, GlobalRng};

#[derive(Component, Copy, Clone)]
pub struct TranslationOffset(pub Vec3);

#[derive(Component)]
pub struct AntSprite;

fn spawn_ant_sprite(
    commands: &mut Commands,
    model_entity: Entity,
    position: &Position,
    color: &AntColor,
    name: &AntName,
    orientation: &AntOrientation,
    role: &AntRole,
    inventory: &AntInventory,
    dead: Option<&Dead>,
    asset_server: &Res<AssetServer>,
    elements_query: &Query<&Element>,
    grid: &Grid,
    element_texture_atlas_handle: &Res<ElementTextureAtlasHandle>,
    model_view_entity_map: &mut ResMut<ModelViewEntityMap>,
) {
    // TODO: z-index is 1.0 here because ant can get hidden behind sand otherwise.
    let translation_offset = TranslationOffset(Vec3::new(0.0, 0.0, 1.0));

    let (sprite_image, sprite_color) = if dead.is_some() {
        ("images/ant_dead.png", Color::GRAY)
    } else {
        ("images/ant.png", color.0)
    };

    let ant_view_entity = commands
        .spawn((
            translation_offset,
            SpatialBundle {
                transform: Transform {
                    translation: grid
                        .grid_to_world_position(*position)
                        .add(translation_offset.0),
                    ..default()
                },
                ..default()
            },
            AtNest,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    AntSprite,
                    SpriteBundle {
                        texture: asset_server.load(sprite_image),
                        sprite: Sprite {
                            color: sprite_color,
                            // 1.5 is just a feel good number to make ants slightly larger than the elements they dig up
                            custom_size: Some(Vec2::splat(1.5)),
                            ..default()
                        },
                        transform: Transform {
                            rotation: orientation.as_world_rotation(),
                            scale: orientation.as_world_scale(),
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|parent: &mut ChildBuilder<'_, '_, '_>| {
                    if let Some(bundle) = get_inventory_item_sprite_bundle(
                        inventory,
                        &elements_query,
                        &element_texture_atlas_handle,
                    ) {
                        parent.spawn(bundle);
                    }

                    if *role == AntRole::Queen {
                        parent.spawn(SpriteBundle {
                            texture: asset_server.load("images/crown.png"),
                            transform: Transform::from_xyz(0.33, 0.33, 1.0),
                            sprite: Sprite {
                                custom_size: Some(Vec2::splat(0.5)),
                                ..default()
                            },
                            ..default()
                        });
                    }
                });

            parent.spawn(Text2dBundle {
                transform: Transform {
                    translation: Vec3::new(0.0, -1.0, 1.0),
                    scale: Vec3::new(0.01, 0.01, 0.0),
                    ..default()
                },
                text: Text::from_section(
                    name.0.as_str(),
                    TextStyle {
                        color: Color::WHITE,
                        font_size: 60.0,
                        ..default()
                    },
                ),
                ..default()
            });
        })
        .id();

    model_view_entity_map
        .insert(model_entity, ant_view_entity);
}

pub fn on_spawn_ant(
    mut commands: Commands,
    ants_query: Query<
        (
            Entity,
            &Position,
            &AntColor,
            &AntOrientation,
            &AntName,
            &AntRole,
            &AntInventory,
            Option<&Dead>,
        ),
        Added<Ant>,
    >,
    asset_server: Res<AssetServer>,
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    element_texture_atlas_handle: Res<ElementTextureAtlasHandle>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let grid = match nest_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

    for (ant_model_entity, position, color, orientation, name, role, inventory, dead) in &ants_query
    {
        spawn_ant_sprite(
            &mut commands,
            ant_model_entity,
            position,
            color,
            name,
            orientation,
            role,
            inventory,
            dead,
            &asset_server,
            &elements_query,
            &grid,
            &element_texture_atlas_handle,
            &mut model_view_entity_map,
        );
    }
}

pub fn rerender_ants(
    ant_model_query: Query<
        (
            Entity,
            &Position,
            &AntColor,
            &AntOrientation,
            &AntName,
            &AntRole,
            &AntInventory,
            Option<&Dead>,
        ),
        With<AtNest>,
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    element_texture_atlas_handle: Res<ElementTextureAtlasHandle>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let grid = nest_query.single();

    for (ant_model_entity, position, color, orientation, name, role, inventory, dead) in
        ant_model_query.iter()
    {
        if let Some(ant_view_entity) = model_view_entity_map.remove(&ant_model_entity) {
            commands.entity(ant_view_entity).despawn_recursive();
        }

        spawn_ant_sprite(
            &mut commands,
            ant_model_entity,
            position,
            color,
            name,
            orientation,
            role,
            inventory,
            dead,
            &asset_server,
            &elements_query,
            &grid,
            &element_texture_atlas_handle,
            &mut model_view_entity_map,
        );
    }
}

pub fn on_despawn_ant(
    mut removed: RemovedComponents<Ant>,
    mut commands: Commands,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let model_entities = &mut removed.read().collect::<HashSet<_>>();

    for model_entity in model_entities.iter() {
        if let Some(view_entity) = model_view_entity_map.remove(model_entity) {
            commands.entity(view_entity).despawn_recursive();
        }
    }
}

pub fn on_update_ant_inventory(
    mut commands: Commands,
    ant_model_query: Query<(Entity, &AntInventory), Changed<AntInventory>>,
    ant_view_query: Query<Option<&Children>>,
    ant_sprite_view_query: Query<&mut Sprite, With<AntSprite>>,
    inventory_item_sprite_query: Query<&InventoryItemSprite>,
    elements_query: Query<&Element>,
    element_texture_atlas_handle: Res<ElementTextureAtlasHandle>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    nest_query: Query<&Grid, With<Nest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (ant_model_entity, inventory) in ant_model_query.iter() {
        if let Some(&ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            if let Some(inventory_item_bundle) = get_inventory_item_sprite_bundle(
                &inventory,
                &elements_query,
                &element_texture_atlas_handle,
            ) {
                if let Ok(children) = ant_view_query.get(ant_view_entity) {
                    if let Some(children) = children {
                        let ant_sprite_entity = children
                            .iter()
                            .find(|&&child| ant_sprite_view_query.get(child).is_ok())
                            .unwrap();

                        commands.entity(*ant_sprite_entity).with_children(
                            |ant_sprite: &mut ChildBuilder| {
                                // TODO: store entity somewhere and despawn using it rather than searching
                                ant_sprite.spawn(inventory_item_bundle);
                            },
                        );
                    }
                }
            } else {
                if let Ok(children) = ant_view_query.get(ant_view_entity) {
                    if let Some(children) = children {
                        let ant_sprite_entity = children
                            .iter()
                            .find(|&&child| ant_sprite_view_query.get(child).is_ok())
                            .unwrap();

                        if let Ok(children) = ant_view_query.get(*ant_sprite_entity) {
                            if let Some(children) = children {
                                for &child in children.iter().filter(|&&child| {
                                    inventory_item_sprite_query.get(child).is_ok()
                                }) {
                                    // Surprisingly, Bevy doesn't fix parent/child relationship when despawning children, so do it manually.
                                    commands.entity(child).remove_parent();
                                    commands.entity(child).despawn();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct InventoryItemSprite;

#[derive(Bundle)]
pub struct AntHeldElementSpriteBundle {
    sprite_sheet_bundle: SpriteSheetBundle,
    inventory_item_sprite: InventoryItemSprite,
}

fn get_inventory_item_sprite_bundle(
    inventory: &AntInventory,
    elements_query: &Query<&Element>,
    element_texture_atlas_handle: &Res<ElementTextureAtlasHandle>,
) -> Option<AntHeldElementSpriteBundle> {
    let element_entity = match inventory.0 {
        Some(element_entity) => element_entity,
        None => return None,
    };

    let element = elements_query.get(element_entity).unwrap();

    let element_exposure = ElementExposure {
        north: true,
        east: true,
        south: true,
        west: true,
    };

    let mut sprite = TextureAtlasSprite::new(get_element_index(element_exposure, *element));
    sprite.custom_size = Some(Vec2::splat(1.0));

    let sprite_sheet_bundle = SpriteSheetBundle {
        transform: Transform::from_xyz(1.0, 0.25, 1.0),
        sprite,
        texture_atlas: element_texture_atlas_handle.0.clone(),
        ..default()
    };

    Some(AntHeldElementSpriteBundle {
        sprite_sheet_bundle,
        inventory_item_sprite: InventoryItemSprite,
    })
}

pub fn on_update_ant_position(
    ant_model_query: Query<(Entity, &Position), (With<Ant>, Changed<Position>)>,
    mut ant_view_query: Query<(&mut Transform, &TranslationOffset)>,
    nest_query: Query<&Grid, With<Nest>>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let grid = match nest_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

    for (ant_model_entity, position) in ant_model_query.iter() {
        if let Some(&ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            if let Ok((mut transform, translation_offset)) = ant_view_query.get_mut(ant_view_entity)
            {
                transform.translation = grid
                    .grid_to_world_position(*position)
                    .add(translation_offset.0);
            }
        }
    }
}

pub fn on_update_ant_color(
    ant_model_query: Query<(Entity, &AntColor), (Changed<AntColor>, Without<Dead>)>,
    ant_view_query: Query<&Children>,
    mut ant_sprite_view_query: Query<&mut Sprite, With<AntSprite>>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
    nest_query: Query<&Grid, With<Nest>>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (ant_model_entity, color) in ant_model_query.iter() {
        if let Some(ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            if let Ok(children) = ant_view_query.get(*ant_view_entity) {
                if let Some(ant_sprite_entity) = children
                    .iter()
                    .find(|&&child| ant_sprite_view_query.get(child).is_ok())
                {
                    if let Ok(mut sprite) = ant_sprite_view_query.get_mut(*ant_sprite_entity) {
                        sprite.color = color.0;
                    }
                }
            }
        }
    }
}

pub fn on_update_ant_orientation(
    ant_model_query: Query<(Entity, &AntOrientation), Changed<AntOrientation>>,
    ant_view_query: Query<&Children>,
    mut ant_sprite_view_query: Query<&mut Transform, With<AntSprite>>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
    nest_query: Query<&Grid, With<Nest>>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (ant_model_entity, orientation) in ant_model_query.iter() {
        if let Some(ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            if let Ok(children) = ant_view_query.get(*ant_view_entity) {
                if let Some(ant_sprite_entity) = children
                    .iter()
                    .find(|&&child| ant_sprite_view_query.get(child).is_ok())
                {
                    if let Ok(mut transform) = ant_sprite_view_query.get_mut(*ant_sprite_entity) {
                        transform.scale = orientation.as_world_scale();
                        transform.rotation = orientation.as_world_rotation();
                    }
                }
            }
        }
    }
}

pub fn on_added_ant_dead(
    ant_model_query: Query<Entity, Added<Dead>>,
    ant_view_query: Query<&Children>,
    mut ant_sprite_view_query: Query<(&mut Handle<Image>, &mut Sprite), With<AntSprite>>,
    asset_server: Res<AssetServer>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    nest_query: Query<&Grid, With<Nest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    for ant_model_entity in ant_model_query.iter() {
        if let Some(ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            if let Ok(children) = ant_view_query.get(*ant_view_entity) {
                if let Some(ant_sprite_entity) = children
                    .iter()
                    .find(|&&child| ant_sprite_view_query.get(child).is_ok())
                {
                    if let Ok((mut image_handle, mut sprite)) =
                        ant_sprite_view_query.get_mut(*ant_sprite_entity)
                    {
                        *image_handle = asset_server.load("images/ant_dead.png");

                        // Apply gray tint to dead ants.
                        sprite.color = Color::GRAY;
                    }
                }
            }
        }
    }
}

// TODO: Invert ownership would make this O(1) instead of O(n).
pub fn on_removed_emote(
    mut removed: RemovedComponents<Emote>,
    emote_view_query: Query<(Entity, &EmoteSprite)>,
    mut commands: Commands,
    nest_query: Query<&Grid, With<Nest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    let emoting_view_entities = &mut removed.read().collect::<HashSet<_>>();

    for (emote_view_entity, emote_sprite) in emote_view_query.iter() {
        if emoting_view_entities.contains(&emote_sprite.parent_entity) {
            // Surprisingly, Bevy doesn't fix parent/child relationship when despawning children, so do it manually.
            commands.entity(emote_view_entity).remove_parent().despawn();
        }
    }
}

pub fn on_tick_emote(
    mut ant_view_query: Query<(Entity, &mut Emote), With<AtNest>>,
    mut commands: Commands,
    settings: Res<Settings>,
) {
    for (ant_view_entity, mut emote) in ant_view_query.iter_mut() {
        let rate_of_emote_expire =
            emote.max() / (settings.emote_duration * DEFAULT_TICKS_PER_SECOND) as f32;
        emote.tick(rate_of_emote_expire);

        if emote.is_expired() {
            commands.entity(ant_view_entity).remove::<Emote>();
        }
    }
}

#[derive(Component)]
pub struct EmoteSprite {
    parent_entity: Entity,
}

pub fn on_added_ant_emote(
    ant_view_query: Query<(Entity, &Emote, &Children), Added<Emote>>,
    ant_sprite_view_query: Query<&AntSprite>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    nest_query: Query<&Grid, With<Nest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (ant_view_entity, emote, children) in ant_view_query.iter() {
        if let Some(ant_sprite_view_entity) = children
            .iter()
            .find(|&&child| ant_sprite_view_query.get(child).is_ok())
        {
            commands
                .entity(*ant_sprite_view_entity)
                .with_children(|parent| {
                    let texture = match emote.emote_type() {
                        EmoteType::Asleep => asset_server.load("images/zzz.png"),
                        EmoteType::FoodLove => asset_server.load("images/foodlove.png"),
                    };

                    parent.spawn((
                        EmoteSprite {
                            parent_entity: ant_view_entity,
                        },
                        SpriteBundle {
                            transform: Transform::from_xyz(0.75, 1.0, 1.0),
                            sprite: Sprite {
                                custom_size: Some(Vec2::splat(1.0)),
                                ..default()
                            },
                            texture,
                            ..default()
                        },
                    ));
                });
        }
    }
}

pub fn on_ant_ate_food(
    mut ant_action_events: EventReader<AntAteFoodEvent>,
    mut commands: Commands,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    for AntAteFoodEvent(ant_model_entity) in ant_action_events.read() {
        let ant_view_entity = match model_view_entity_map.get(ant_model_entity) {
            Some(ant_view_entity) => *ant_view_entity,
            None => continue,
        };

        commands
            .entity(ant_view_entity)
            .insert(Emote::new(EmoteType::FoodLove));
    }
}

pub fn ants_sleep_emote(
    ants_query: Query<Entity, (With<Asleep>, With<AtNest>)>,
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
    settings: Res<Settings>,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    for ant_model_entity in ants_query.iter() {
        if rng.f32() < settings.probabilities.sleep_emote {
            let ant_view_entity = match model_view_entity_map.get(&ant_model_entity) {
                Some(ant_view_entity) => *ant_view_entity,
                None => continue,
            };

            commands
                .entity(ant_view_entity)
                .insert(Emote::new(EmoteType::Asleep));
        }
    }
}

pub fn on_ant_wake_up(
    // TODO: Maybe this should be event-driven and have an AntWakeUpEvent instead? That way this won't run when ants are getting destroyed.
    mut removed: RemovedComponents<Asleep>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    mut commands: Commands,
) {
    for ant_model_entity in removed.read() {
        if let Some(view_entity) = model_view_entity_map.get(&ant_model_entity) {
            // TODO: This code isn't great because it's presumptuous. There's not a guarantee that sleeping ant was showing an emote
            // and, if it is showing an emote, maybe that emote isn't the Asleep emote. Still, this assumption holds for now.
            commands.entity(*view_entity).remove::<Emote>();
        }
    }
}

pub fn cleanup_ants() {
    // TODO: Cleanup anything else related to ants here.
}
