use bevy::prelude::*;

use crate::{
    common::position::Position,
    nest::Nest,
    story_state::StoryState,
    story_time::{StoryTime, TimeInfo},
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

fn create_sky_sprites(
    width: isize,
    height: isize,
    nest: &Res<Nest>,
    story_time: &Res<StoryTime>,
) -> Vec<(SpriteBundle, Position, SkyBackground)> {
    let mut sky_sprites = vec![];

    let current_decimal_hours = story_time.as_time_info().get_decimal_hours();
    let (sunrise_decimal_hours, sunset_decimal_hours) =
        story_time.get_sunrise_sunset_decimal_hours();

    let (north_color, south_color) = get_sky_gradient_color(
        current_decimal_hours,
        sunrise_decimal_hours,
        sunset_decimal_hours,
    );

    for x in 0..width {
        for y in 0..height {
            let position = Position::new(x, y);

            let mut world_position = nest.as_world_position(position);
            // Background needs z-index of 0 as it should be the bottom layer and not cover sprites
            world_position.z = 0.0;

            let t_y: f32 = position.y as f32 / *nest.surface_level() as f32;
            let color = interpolate_color(north_color, south_color, t_y);

            let sky_sprite = SpriteBundle {
                transform: Transform::from_translation(world_position),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(1.0)),
                    ..default()
                },
                ..default()
            };

            sky_sprites.push((sky_sprite, position, SkyBackground));
        }
    }

    sky_sprites
}

fn create_tunnel_sprites(
    width: isize,
    height: isize,
    y_offset: isize,
    nest: &Res<Nest>,
) -> Vec<(SpriteBundle, Position, TunnelBackground)> {
    let mut tunnel_sprites = vec![];

    let top_color: Color = Color::rgba(0.373, 0.290, 0.165, 1.0);
    let bottom_color = Color::rgba(0.24, 0.186, 0.106, 1.0);

    for x in 0..width {
        for y in 0..height {
            let position = Position::new(x, y + y_offset);

            let mut world_position = nest.as_world_position(position);
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

            tunnel_sprites.push((tunnel_sprite, position, TunnelBackground));
        }
    }

    tunnel_sprites
}

pub fn update_sky_background(
    mut sky_sprite_query: Query<(&mut Sprite, &Position), With<SkyBackground>>,
    mut last_run_time_info: Local<TimeInfo>,
    story_time: Res<StoryTime>,
    story_state: Res<State<StoryState>>,
    // TODO: Option because need to run during Initializing to reset but Nest is gone already.
    // maybe could find a way to reset this at the same time Nest is getting despawned?
    nest: Option<Res<Nest>>,
) {
    // Reset local when in initializing to prevent data retention issue when clicking "Reset" in Sandbox Mode
    if *story_state == StoryState::Initializing {
        *last_run_time_info = TimeInfo::default();
        return;
    }

    let nest = match nest {
        Some(nest) => nest,
        None => panic!("expected world map to exist at this point"),
    };

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
    for (mut sprite, position) in sky_sprite_query.iter_mut() {
        let t_y: f32 = position.y as f32 / *nest.surface_level() as f32;
        let color = interpolate_color(north_color, south_color, t_y);

        sprite.color = color;
    }

    *last_run_time_info = time_info;
}

// Spawn non-interactive background (sky blue / tunnel brown)
pub fn setup_background(mut commands: Commands, nest: Res<Nest>, story_time: Res<StoryTime>) {
    let air_height = *nest.surface_level() + 1;

    commands.spawn_batch(create_sky_sprites(
        *nest.width(),
        air_height,
        &nest,
        &story_time,
    ));

    commands.spawn_batch(create_tunnel_sprites(
        *nest.width(),
        *nest.height() - air_height,
        air_height,
        &nest,
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
