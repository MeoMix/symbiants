use bevy::{asset::LoadState, prelude::*};

use simulation::common::element::Element;

use crate::common::{LoadProgress, RenderingLoadProgress};

use super::ElementExposure;

#[derive(Resource)]
pub struct ElementSpriteSheetHandle(pub Handle<Image>);

#[derive(Resource)]
pub struct ElementTextureAtlasLayoutHandle(pub Handle<TextureAtlasLayout>);

pub fn start_load_element_sprite_sheet(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut rendering_load_progress: ResMut<RenderingLoadProgress>,
) {
    rendering_load_progress.element_sprite_sheet = LoadProgress::Loading;

    commands.insert_resource(ElementSpriteSheetHandle(
        asset_server.load::<Image>("textures/element/sprite_sheet.png"),
    ));
}

pub fn check_element_sprite_sheet_loaded(
    element_sprite_sheet_handle: Res<ElementSpriteSheetHandle>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut rendering_load_progress: ResMut<RenderingLoadProgress>,
) {
    let load_state = asset_server.load_state(&element_sprite_sheet_handle.0);

    if load_state == LoadState::Loaded {
        let texture_atlas = TextureAtlasLayout::from_grid(
            Vec2::splat(128.0),
            3,
            16,
            None,
            None,
        );

        commands.insert_resource(ElementTextureAtlasLayoutHandle(
            texture_atlases.add(texture_atlas),
        ));

        // TODO: support failure?
        rendering_load_progress.element_sprite_sheet = LoadProgress::Success;
    }
}

// TODO: super hardcoded to the order they appear in sprite_sheet.png
// Spritesheet is organized as:
// 0 - none exposed
// 1 - north exposed
// 2 - east exposed
// 3 - south exposed
// 4 - west exposed
// 5 - north/east exposed
// 6 - east/south exposed
// 7 - south/west exposed
// 8 - west/north exposed
// 9 - north/south exposed
// 10 - east/west exposed
// 11 - north/east/south exposed
// 12 - east/south/west exposed
// 13 - south/west/north exposed
// 14 - west/north/east exposed
// 15 - all exposed
pub fn get_element_index(exposure: ElementExposure, element: Element) -> usize {
    let row_index = match exposure {
        ElementExposure {
            north: false,
            east: false,
            south: false,
            west: false,
        } => 0,
        ElementExposure {
            north: true,
            east: false,
            south: false,
            west: false,
        } => 1,
        ElementExposure {
            north: false,
            east: true,
            south: false,
            west: false,
        } => 2,
        ElementExposure {
            north: false,
            east: false,
            south: true,
            west: false,
        } => 3,
        ElementExposure {
            north: false,
            east: false,
            south: false,
            west: true,
        } => 4,
        ElementExposure {
            north: true,
            east: true,
            south: false,
            west: false,
        } => 5,
        ElementExposure {
            north: false,
            east: true,
            south: true,
            west: false,
        } => 6,
        ElementExposure {
            north: false,
            east: false,
            south: true,
            west: true,
        } => 7,
        ElementExposure {
            north: true,
            east: false,
            south: false,
            west: true,
        } => 8,
        ElementExposure {
            north: true,
            east: false,
            south: true,
            west: false,
        } => 9,
        ElementExposure {
            north: false,
            east: true,
            south: false,
            west: true,
        } => 10,
        ElementExposure {
            north: true,
            east: true,
            south: true,
            west: false,
        } => 11,
        ElementExposure {
            north: false,
            east: true,
            south: true,
            west: true,
        } => 12,
        ElementExposure {
            north: true,
            east: false,
            south: true,
            west: true,
        } => 13,
        ElementExposure {
            north: true,
            east: true,
            south: false,
            west: true,
        } => 14,
        ElementExposure {
            north: true,
            east: true,
            south: true,
            west: true,
        } => 15,
    };

    let column_index = match element {
        Element::Dirt => 0,
        Element::Food => 1,
        Element::Sand => 2,
        _ => panic!("Element {:?} not supported", element),
    };

    row_index * 3 + column_index
}
