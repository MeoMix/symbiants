use bevy::prelude::*;

use crate::{
    story_state::StoryState,
    story_time::{StoryElapsedTicks, TimeInfo},
    world_map::{position::Position, WorldMap},
};

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

fn get_sky_gradient_color(hour: isize, minute: isize) -> Color {
    let current_time = hour as f32 + minute as f32 / 60.0;

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
        (6.0, predawn),
        (8.0, dawn),
        (8.5, morning),
        (12.0, noon),
        (19.5, evening),
        (20.0, dusk),
        (22.0, postdusk),
    ];

    let (idx, &(start_time, start_color)) = key_moments
        .iter()
        .enumerate()
        .find(|&(i, &(time, _))| {
            i == key_moments.len() - 1
                || (current_time >= time && current_time < key_moments[i + 1].0)
        })
        .unwrap();

    let (end_time, end_color) = if idx == key_moments.len() - 1 {
        (24.0 + key_moments[0].0, key_moments[0].1)
    } else {
        key_moments[idx + 1]
    };

    interpolate_color(
        start_color,
        end_color,
        (current_time - start_time) / (end_time - start_time),
    )
}

fn create_sky_sprites(
    width: isize,
    height: isize,
    world_map: &Res<WorldMap>,
    elapsed_ticks: &Res<StoryElapsedTicks>,
) -> Vec<(SpriteBundle, SkyBackground)> {
    let mut sky_sprites = vec![];

    let time_info = elapsed_ticks.as_time_info();

    let color = get_sky_gradient_color(time_info.hours, time_info.minutes);

    for x in 0..width {
        for y in 0..height {
            // TODO: Careful, if position is spawned here then it'll get saved automatically even though this is a view-only concern.
            let position = Position::new(x, y);

            let mut world_position = position.as_world_position(&world_map);
            // Background needs z-index of 0 as it should be the bottom layer and not cover sprites
            world_position.z = 0.0;

            let sky_sprite = SpriteBundle {
                transform: Transform::from_translation(world_position),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(1.0)),
                    ..default()
                },
                ..default()
            };

            sky_sprites.push((sky_sprite, SkyBackground));
        }
    }

    sky_sprites
}

fn create_tunnel_sprites(
    width: isize,
    height: isize,
    y_offset: isize,
    world_map: &Res<WorldMap>,
) -> Vec<(SpriteBundle, TunnelBackground)> {
    let mut tunnel_sprites = vec![];

    let top_color: Color = Color::rgba(0.373, 0.290, 0.165, 1.0);
    let bottom_color = Color::rgba(0.24, 0.186, 0.106, 1.0);

    for x in 0..width {
        for y in 0..height {
            let position = Position::new(x, y + y_offset);

            let mut world_position = position.as_world_position(&world_map);
            // Background needs z-index of 0 as it should be the bottom layer and not cover sprites
            world_position.z = 0.0;

            let color = interpolate_color(top_color, bottom_color, y as f32 / height as f32);

            let tunnel_sprite = SpriteBundle {
                transform: Transform::from_translation(world_position),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(1.0)),
                    ..default()
                },
                ..default()
            };

            tunnel_sprites.push((tunnel_sprite, TunnelBackground));
        }
    }

    tunnel_sprites
}

pub fn update_sky_background(
    mut sky_sprite_query: Query<&mut Sprite, With<SkyBackground>>,
    mut last_run_time_info: Local<TimeInfo>,
    elapsed_ticks: Res<StoryElapsedTicks>,
    story_state: Res<State<StoryState>>,
) {
    // Reset local when in initializing to prevent data retention issue when clicking "Reset" in Sandbox Mode
    if *story_state == StoryState::Initializing {
        *last_run_time_info = TimeInfo::default();
        return;
    }

    let time_info = elapsed_ticks.as_time_info();

    // Update the sky's colors once every 10 minutes of elapsed *story time* not real-world time.
    if time_info.days == last_run_time_info.days
        && time_info.hours == last_run_time_info.hours
        // Check if difference between time_info and last_run_time_info minutes is 10
        && (time_info.minutes - last_run_time_info.minutes).abs() < 10
    {
        return;
    }

    let color = get_sky_gradient_color(time_info.hours, time_info.minutes);
    for mut sprite in sky_sprite_query.iter_mut() {
        sprite.color = color;
    }

    *last_run_time_info = time_info;
}

// Spawn non-interactive background (sky blue / tunnel brown)
pub fn setup_background(
    mut commands: Commands,
    world_map: Res<WorldMap>,
    elapsed_ticks: Res<StoryElapsedTicks>,
) {
    let air_height = *world_map.surface_level() + 1;

    commands.spawn_batch(create_sky_sprites(
        *world_map.width(),
        air_height,
        &world_map,
        &elapsed_ticks,
    ));

    commands.spawn_batch(create_tunnel_sprites(
        *world_map.width(),
        *world_map.height() - air_height,
        air_height,
        &world_map,
    ));
}

pub fn teardown_background(
    background_query: Query<Entity, Or<(With<TunnelBackground>, With<SkyBackground>)>>,
    mut commands: Commands,
) {
    for background_entity in background_query.iter() {
        commands.entity(background_entity).despawn();
    }
}
