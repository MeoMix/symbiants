use bevy::prelude::*;
use simulation::{
    common::{grid::Grid, position::Position},
    crater_simulation::crater::{AtCrater, Crater},
    nest_simulation::{
        ant::{Ant, AntColor, AntInventory, AntName, AntOrientation, AntRole, Dead},
        element::Element,
    },
};
use std::ops::Add;

use crate::common::{visible_grid::VisibleGrid, ModelViewEntityMap};

#[derive(Component, Copy, Clone)]
pub struct TranslationOffset(pub Vec3);

// TODO: Maybe call this AntView instead?
#[derive(Component)]
pub struct AntSpriteContainer {
    pub sprite_entity: Entity,
    pub label_entity: Entity,
    pub inventory_item_entity: Option<Entity>,
    pub emote_entity: Option<Entity>,
}

/// When an ant model is added to the simulation, render an associated ant sprite.
/// This *only* handles the initial rendering of the ant sprite. Updates are handled by other systems.
/// This does handle rendering the ant's held inventory item, it's role-associated hat, it's name label,
/// and properly draws it as dead if the model is dead when spawned.
/// This does not handle rendering the ant's emote.
/// All of this is a bit dubious because ants aren't expected to spawn dead, or to spawn holding anything, but
/// allows for code reuse when rerendering exists ants after toggling between scenes.
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
        (Added<Ant>, With<AtCrater>),
    >,
    asset_server: Res<AssetServer>,
    elements_query: Query<&Element>,
    crater_query: Query<&Grid, With<Crater>>,
    // element_texture_atlas_handle: Res<ElementTextureAtlasHandle>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let grid = match crater_query.get(visible_grid_entity) {
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
            // &element_texture_atlas_handle,
            &mut model_view_entity_map,
        );
    }
}

/// When user switches to a different scene (Crater->Nest) all Crater views are despawned.
/// Thus, when switching back to Crater, all Ants need to be redrawn once. Their underlying models
/// have not been changed or added, though, so a separate rerender system is needed.
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
        With<AtCrater>,
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    elements_query: Query<&Element>,
    crater_query: Query<&Grid, With<Crater>>,
    // element_texture_atlas_handle: Res<ElementTextureAtlasHandle>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let grid = crater_query.single();

    for (ant_model_entity, position, color, orientation, name, role, inventory, dead) in
        ant_model_query.iter()
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
            // &element_texture_atlas_handle,
            &mut model_view_entity_map,
        );
    }
}

/// Remove resources, etc.
pub fn cleanup_ants() {}

/// Non-System Helper Functions:

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
    // element_texture_atlas_handle: &Res<ElementTextureAtlasHandle>,
    model_view_entity_map: &mut ResMut<ModelViewEntityMap>,
) {
    // TODO: z-index is 1.0 here because ant can get hidden behind sand otherwise.
    let translation_offset = TranslationOffset(Vec3::new(0.0, 0.0, 1.0));

    let (sprite_image, sprite_color) = if dead.is_some() {
        ("images/ant_dead.png", Color::GRAY)
    } else {
        ("images/ant.png", color.0)
    };

    // Spawn AntSprite with child inventory/hat
    let mut ant_sprite = commands.spawn((SpriteBundle {
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
    },));

    // let mut inventory_item_entity = None;

    // ant_sprite.with_children(|parent: &mut ChildBuilder<'_, '_, '_>| {
    //     // if let Some(element_entity) = inventory.0 {
    //     //     let bundle = get_inventory_item_bundle(
    //     //         element_entity,
    //     //         &elements_query,
    //     //         &element_texture_atlas_handle,
    //     //     );

    //     //     inventory_item_entity = Some(parent.spawn(bundle).id());
    //     // }

    //     // if *role == AntRole::Queen {
    //     //     parent.spawn(SpriteBundle {
    //     //         texture: asset_server.load("images/crown.png"),
    //     //         transform: Transform::from_xyz(0.33, 0.33, 1.0),
    //     //         sprite: Sprite {
    //     //             custom_size: Some(Vec2::splat(0.5)),
    //     //             ..default()
    //     //         },
    //     //         ..default()
    //     //     });
    //     // }
    // });

    let sprite_entity = ant_sprite.id();

    let ant_label = commands.spawn(Text2dBundle {
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

    let label_entity = ant_label.id();

    let ant_view_entity = commands
        .spawn((
            AntSpriteContainer {
                sprite_entity,
                label_entity,
                inventory_item_entity: None,
                emote_entity: None,
            },
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
            AtCrater,
        ))
        .push_children(&[sprite_entity, label_entity])
        .id();

    model_view_entity_map.insert(model_entity, ant_view_entity);
}
