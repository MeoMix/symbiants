use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    app_state::AppState,
    story::{
        common::position::Position, grid::Grid, rendering::common::{grid_to_tile_pos, VisibleGrid}, simulation::nest_simulation::nest::{AtNest, Nest}, story_time::{StoryTime, TimeInfo}
    },
};

#[derive(Component)]
pub struct BackgroundTilemap;

#[derive(Component)]
pub struct Background;

#[derive(Component)]
pub struct SkyBackground;

#[derive(Component)]
pub struct TunnelBackground;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

fn interpolate_color(color_a: Color, color_b: Color, t: f32) -> Color {
    let r = lerp(color_a.r(), color_b.r(), t);
    let g = lerp(color_a.g(), color_b.g(), t);
    let b = lerp(color_a.b(), color_b.b(), t);
    let a = lerp(color_a.a(), color_b.a(), t);

    Color::rgba(r, g, b, a)
}

// TODO: Instead of using sunrise/sunset, consider swapping to altitude and using the sun's altitude directly to define key moments.
fn get_sky_gradient_color(
    current_decimal_hours: f32,
    sunrise_decimal_hours: f32,
    sunset_decimal_hours: f32,
) -> (Color, Color) {
    let midnight = Color::rgba(0.0471, 0.0353, 0.0392, 1.0);
    let predawn = Color::rgba(0.0471, 0.0353, 0.0392, 1.0);
    let dawn = Color::rgba(0.0863, 0.1059, 0.2118, 1.0);
    let morning = Color::rgba(0.329, 0.508, 0.622, 1.0);
    let noon = Color::rgba(0.529, 0.808, 0.922, 1.0);
    let evening = Color::rgba(0.329, 0.508, 0.622, 1.0);
    let dusk = Color::rgba(0.0863, 0.1059, 0.2118, 1.0);
    let postdusk = Color::rgba(0.0471, 0.0353, 0.0392, 1.0);

    // Note these are shifted slightly from "true center" to try and make it so that
    // the sun rises and sets at times that would make sense for a person's daily routine.
    let key_moments = [
        (0.0, midnight),
        (sunrise_decimal_hours - 2.0, predawn),
        (sunrise_decimal_hours, dawn),
        (sunrise_decimal_hours + 0.5, morning),
        (12.0, noon),
        (sunset_decimal_hours - 0.5, evening),
        (sunset_decimal_hours, dusk),
        (sunset_decimal_hours + 2.0, postdusk),
    ];

    let (idx, &(start_time, start_color)) = key_moments
        .iter()
        .enumerate()
        .find(|&(i, &(time, _))| {
            i == key_moments.len() - 1
                || (current_decimal_hours >= time && current_decimal_hours < key_moments[i + 1].0)
        })
        .unwrap();

    let (end_time, end_color) = if idx == key_moments.len() - 1 {
        (24.0 + key_moments[0].0, key_moments[0].1)
    } else {
        key_moments[idx + 1]
    };

    let progress = (current_decimal_hours - start_time) / (end_time - start_time);

    let north_color;
    let south_color;
    if end_time <= 12.0 {
        north_color = interpolate_color(start_color, end_color, progress);

        south_color = interpolate_color(start_color, end_color, 1.0 - (1.0 - progress).powf(3.0));
    } else {
        north_color = interpolate_color(start_color, end_color, progress);

        south_color = interpolate_color(start_color, end_color, 1.0 - (1.0 - progress).powf(3.0));
    }

    (north_color, south_color)
}

pub fn update_sky_background(
    mut sky_tile_query: Query<(&mut TileColor, &Position), With<SkyBackground>>,
    // TODO: Relying on Local<> while trying to support "Reset Sandbox" without the ability to remove systems entirely is challenging.
    // Probably rewrite this to be a resource instead of a Local.
    mut last_run_time_info: Local<TimeInfo>,
    app_state: Res<State<AppState>>,
    nest_query: Query<&Nest>,
    // Optional due to running during cleanup
    visible_grid: Option<Res<VisibleGrid>>,
    story_time: Option<Res<StoryTime>>,
) {
    // Reset local when in initializing to prevent data retention issue when clicking "Reset" in Sandbox Mode
    if *app_state == AppState::Cleanup {
        *last_run_time_info = TimeInfo::default();
        return;
    }

    let visible_grid_entity = match visible_grid.unwrap().0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    let story_time = story_time.unwrap();
    let nest = nest_query.single();
    let time_info = story_time.as_time_info();

    // Update the sky's colors once a minute of elapsed *story time* not real-world time.
    if time_info.days() == last_run_time_info.days()
        && time_info.hours() == last_run_time_info.hours()
        // Check if difference between time_info and last_run_time_info minutes is 1
        && (time_info.minutes() - last_run_time_info.minutes()).abs() < 1
    {
        return;
    }

    let current_decimal_hours = story_time.as_time_info().get_decimal_hours();
    let (sunrise_decimal_hours, sunset_decimal_hours) =
        story_time.get_sunrise_sunset_decimal_hours();

    let (north_color, south_color) = get_sky_gradient_color(
        current_decimal_hours,
        sunrise_decimal_hours,
        sunset_decimal_hours,
    );
    for (mut tile_color, position) in sky_tile_query.iter_mut() {
        let t_y: f32 = position.y as f32 / nest.surface_level() as f32;
        let color = interpolate_color(north_color, south_color, t_y);

        *tile_color = color.into();
    }

    *last_run_time_info = time_info;
}

pub fn spawn_background_tilemap(mut commands: Commands, nest_query: Query<&Grid, With<Nest>>) {
    let grid = nest_query.single();

    let map_size = TilemapSize {
        x: grid.width() as u32,
        y: grid.height() as u32,
    };
    let grid_size = TilemapGridSize { x: 1.0, y: 1.0 };
    let map_type = TilemapType::default();

    commands.spawn((
        BackgroundTilemap,
        TilemapBundle {
            grid_size,
            size: map_size,
            storage: TileStorage::empty(map_size),
            physical_tile_size: TilemapPhysicalTileSize { x: 1.0, y: 1.0 },
            // Doesn't need to be 128x128 here since not reading from spritesheet.
            tile_size: TilemapTileSize { x: 1.0, y: 1.0 },
            map_type: TilemapType::Square,
            // Background tiles go at z: 0 because they should render behind elements/ants.
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            ..Default::default()
        },
    ));
}

// Spawn non-interactive background (sky blue / tunnel brown)
pub fn spawn_background(
    mut commands: Commands,
    nest_query: Query<(&Grid, &Nest)>,
    mut tilemap_query: Query<(Entity, &mut TileStorage), With<BackgroundTilemap>>,
    story_time: Res<StoryTime>,
) {
    let (grid, nest) = nest_query.single();
    let air_height = nest.surface_level() + 1;

    let (tilemap_entity, mut tile_storage) = tilemap_query.single_mut();

    let current_decimal_hours = story_time.as_time_info().get_decimal_hours();
    let (sunrise_decimal_hours, sunset_decimal_hours) =
        story_time.get_sunrise_sunset_decimal_hours();

    let (north_color, south_color) = get_sky_gradient_color(
        current_decimal_hours,
        sunrise_decimal_hours,
        sunset_decimal_hours,
    );

    let width = grid.width();
    let height = air_height;

    for x in 0..width {
        for y in 0..height {
            let position = Position::new(x, y);

            let t_y: f32 = position.y as f32 / nest.surface_level() as f32;
            let color = interpolate_color(north_color, south_color, t_y);
            let tile_pos = grid_to_tile_pos(grid, position);

            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        color: color.into(),
                        ..default()
                    },
                    position,
                    SkyBackground,
                    Background,
                    AtNest,
                ))
                .id();

            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    // Create background sprites
    let width = grid.width();
    let height = grid.height() - air_height;
    let y_offset = air_height;

    let top_color: Color = Color::rgba(0.373, 0.290, 0.165, 1.0);
    let bottom_color = Color::rgba(0.24, 0.186, 0.106, 1.0);

    for x in 0..width {
        for y in 0..height {
            let position = Position::new(x, y + y_offset);

            let color = interpolate_color(top_color, bottom_color, y as f32 / height as f32);

            let tile_pos = grid_to_tile_pos(grid, position);

            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        color: color.into(),
                        ..default()
                    },
                    position,
                    TunnelBackground,
                    Background,
                    AtNest,
                ))
                .id();

            tile_storage.set(&tile_pos, tile_entity);
        }
    }
}

pub fn cleanup_background() {
    // TODO: Cleanup anything else related to background here.
}
