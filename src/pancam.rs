use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    input::touch::Touch,
    math::vec2,
    prelude::*,
    render::camera::CameraProjection,
    window::PrimaryWindow,
};

use crate::mouse::IsPointerCaptured;

/// Plugin that adds the necessary systems for `PanCam` components to work
#[derive(Default)]
pub struct PanCamPlugin;

/// System set to allow ordering of `PanCamPlugin`
#[derive(Debug, Clone, Copy, SystemSet, PartialEq, Eq, Hash)]
pub struct PanCamSystemSet;

impl Plugin for PanCamPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TouchTracker::default())
            .insert_resource(TouchCameraConfig::default());

        app.add_systems(
            Update,
            (
                camera_mouse_pan,
                camera_mouse_zoom,
                camera_touch_pan_zoom,
                auto_clamp_translation,
            )
                .in_set(PanCamSystemSet)
                .run_if(resource_equals(IsPointerCaptured(false))),
        )
        .register_type::<PanCam>();
    }
}

// Whenever the orthographic projection area size changes - reclamp the camera translation
// NOTE: I added this because when the game starts, user is clicking a button to start the game,
// and this pancam logic races against the button click logic. This can result in a mismeasurement
// because a command hasn't been applied yet.

// This workaround wouldn't be hacky if there was no visual delay, but I still see a slight stutter in the UI
// so I assume this isn't a great solution. Better than having the camera be misaligned though.
fn auto_clamp_translation(
    mut query: Query<
        (&PanCam, &mut Transform, &OrthographicProjection),
        Changed<OrthographicProjection>,
    >,
    mut last_size: Local<Option<Vec2>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(query_result) = query.get_single_mut() {
        let (cam, mut transform, projection) = query_result;

        let proj_size = projection.area.size();

        if Some(proj_size) == *last_size {
            return;
        }

        let window = primary_window.single();
        let window_size = Vec2::new(window.width(), window.height());

        clamp_translation(cam, projection, &mut transform, Vec2::ZERO, window_size);

        *last_size = Some(proj_size);
    }
}

fn clamp_translation(
    cam: &PanCam,
    projection: &OrthographicProjection,
    transform: &mut Transform,
    delta_device_pixels: Vec2,
    window_size: Vec2,
) {
    let proj_size = projection.area.size();
    let world_units_per_device_pixel = proj_size / window_size;

    // The proposed new camera position
    let delta_world = delta_device_pixels * world_units_per_device_pixel;
    let mut proposed_cam_transform = transform.translation - delta_world.extend(0.);

    // Check whether the proposed camera movement would be within the provided boundaries, override it if we
    // need to do so to stay within bounds.
    if let Some(min_x_boundary) = cam.min_x {
        let min_safe_cam_x = min_x_boundary + proj_size.x / 2.;
        proposed_cam_transform.x = proposed_cam_transform.x.max(min_safe_cam_x);
    }
    if let Some(max_x_boundary) = cam.max_x {
        let max_safe_cam_x = max_x_boundary - proj_size.x / 2.;
        proposed_cam_transform.x = proposed_cam_transform.x.min(max_safe_cam_x);
    }
    if let Some(min_y_boundary) = cam.min_y {
        let min_safe_cam_y = min_y_boundary + proj_size.y / 2.;
        proposed_cam_transform.y = proposed_cam_transform.y.max(min_safe_cam_y);
    }
    if let Some(max_y_boundary) = cam.max_y {
        let max_safe_cam_y = max_y_boundary - proj_size.y / 2.;
        proposed_cam_transform.y = proposed_cam_transform.y.min(max_safe_cam_y);
    }

    transform.translation = proposed_cam_transform;
}

fn camera_mouse_zoom(
    mut query: Query<(
        &PanCam,
        &mut OrthographicProjection,
        &mut Transform,
        &mut Camera,
    )>,
    mut scroll_events: EventReader<MouseWheel>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let pixels_per_line = 100.; // Maybe make configurable?
    let scroll = scroll_events
        .iter()
        .map(|ev| match ev.unit {
            MouseScrollUnit::Pixel => ev.y,
            MouseScrollUnit::Line => ev.y * pixels_per_line,
        })
        .sum::<f32>();

    if scroll == 0. {
        return;
    }

    let window = primary_window.single();
    let window_size = Vec2::new(window.width(), window.height());
    let mouse_normalized_screen_pos = window
        .cursor_position()
        .map(|cursor_pos| (cursor_pos / window_size) * 2. - Vec2::ONE)
        .map(|p| Vec2::new(p.x, -p.y));

    for (cam, mut proj, mut pos, camera) in &mut query {
        if cam.enabled {
            let old_scale = proj.scale;
            proj.scale = (proj.scale * (1. + -scroll * 0.001)).max(cam.min_scale);

            // Apply max scale constraint
            if let Some(max_scale) = cam.max_scale {
                proj.scale = proj.scale.min(max_scale);
            }

            // If there is both a min and max boundary, that limits how far we can zoom. Make sure we don't exceed that
            let scale_constrained = BVec2::new(
                cam.min_x.is_some() && cam.max_x.is_some(),
                cam.min_y.is_some() && cam.max_y.is_some(),
            );

            if scale_constrained.x || scale_constrained.y {
                let bounds_width = if let (Some(min_x), Some(max_x)) = (cam.min_x, cam.max_x) {
                    max_x - min_x
                } else {
                    f32::INFINITY
                };

                let bounds_height = if let (Some(min_y), Some(max_y)) = (cam.min_y, cam.max_y) {
                    max_y - min_y
                } else {
                    f32::INFINITY
                };

                let bounds_size = vec2(bounds_width, bounds_height);
                let max_safe_scale = max_scale_within_bounds(bounds_size, &proj, window_size);

                if scale_constrained.x {
                    proj.scale = proj.scale.min(max_safe_scale.x);
                }

                if scale_constrained.y {
                    proj.scale = proj.scale.min(max_safe_scale.y);
                }
            }

            // Move the camera position to normalize the projection window
            if let (Some(mouse_normalized_screen_pos), true) =
                (mouse_normalized_screen_pos, cam.zoom_to_cursor)
            {
                let proj_size = proj.area.max / old_scale;
                let mouse_world_pos = pos.translation.truncate()
                    + mouse_normalized_screen_pos * proj_size * old_scale;
                pos.translation = (mouse_world_pos
                    - mouse_normalized_screen_pos * proj_size * proj.scale)
                    .extend(pos.translation.z);

                // As we zoom out, we don't want the viewport to move beyond the provided boundary. If the most recent
                // change to the camera zoom would move cause parts of the window beyond the boundary to be shown, we
                // need to change the camera position to keep the viewport within bounds. The four if statements below
                // provide this behavior for the min and max x and y boundaries.
                let proj_size = proj.area.size();

                let half_of_viewport = proj_size / 2.;

                if let Some(min_x_bound) = cam.min_x {
                    let min_safe_cam_x = min_x_bound + half_of_viewport.x;
                    pos.translation.x = pos.translation.x.max(min_safe_cam_x);
                }
                if let Some(max_x_bound) = cam.max_x {
                    let max_safe_cam_x = max_x_bound - half_of_viewport.x;
                    pos.translation.x = pos.translation.x.min(max_safe_cam_x);
                }
                if let Some(min_y_bound) = cam.min_y {
                    let min_safe_cam_y = min_y_bound + half_of_viewport.y;
                    pos.translation.y = pos.translation.y.max(min_safe_cam_y);
                }
                if let Some(max_y_bound) = cam.max_y {
                    let max_safe_cam_y = max_y_bound - half_of_viewport.y;
                    pos.translation.y = pos.translation.y.min(max_safe_cam_y);
                }
            }

            if let Some(size) = camera.logical_viewport_size() {
                proj.update(size.x, size.y);
            }

            clamp_translation(cam, &mut proj, &mut pos, Vec2::default(), window_size);
        }
    }
}

/// max_scale_within_bounds is used to find the maximum safe zoom out/projection
/// scale when we have been provided with minimum and maximum x boundaries for
/// the camera.
fn max_scale_within_bounds(
    bounds_size: Vec2,
    proj: &OrthographicProjection,
    window_size: Vec2, //viewport?
) -> Vec2 {
    let mut p = proj.clone();
    p.scale = 1.;
    p.update(window_size.x, window_size.y);
    let base_world_size = p.area.size();
    bounds_size / base_world_size
}

fn camera_mouse_pan(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut query: Query<(&PanCam, &mut Transform, &OrthographicProjection)>,
    mut last_pos: Local<Option<Vec2>>,
) {
    let window = primary_window.single();
    let window_size = Vec2::new(window.width(), window.height());

    // Use position instead of MouseMotion, otherwise we don't get acceleration movement
    let current_pos = match window.cursor_position() {
        Some(c) => Vec2::new(c.x, -c.y),
        None => return,
    };
    let delta_device_pixels = current_pos - last_pos.unwrap_or(current_pos);

    for (cam, mut transform, projection) in &mut query {
        if cam.enabled
            && (cam
                .grab_buttons
                .iter()
                .any(|btn| mouse_buttons.pressed(*btn)))
        {
            clamp_translation(
                cam,
                projection,
                &mut transform,
                delta_device_pixels,
                window_size,
            );
        }
    }
    *last_pos = Some(current_pos);
}

/// Contains the configuration parameters for the plugin.
/// A copy of this will be attached as a `Resource` to the `App`.
#[derive(Resource, Clone)]
pub struct TouchCameraConfig {
    /// How far the camera will move relative to the touch drag distance. Higher is faster
    pub drag_sensitivity: f32,
    /// How much the camera will zoom relative to the pinch distance using two fingers. Higher means faster.
    /// At the moment the default is very low at 0.005 but this might change in the future
    pub zoom_sensitivity: f32,
    /// Minimum time before starting to pan in seconds. Useful to avoid panning on short taps
    pub touch_time_min: f32,
    /// Tolerance for pinch fingers moving in opposite directions (-1. .. 1.).
    /// Higher values make it more tolerant.
    /// Very low values not recommended as it would be overly sensitive
    pub opposites_tolerance: f32,
}

impl Default for TouchCameraConfig {
    fn default() -> Self {
        Self {
            drag_sensitivity: 1.,
            zoom_sensitivity: 0.005,
            touch_time_min: 0.01,
            opposites_tolerance: 0.,
        }
    }
}

/// This is the tag that the plugin will scan for and update its `Camera` component.
/// You can either attach it manually to your camera, or the plugin will try to attach it
/// to the default camera in the `PostStartup` schedule
#[derive(Component)]
pub struct TouchCameraTag;

#[derive(PartialEq, Default)]
enum GestureType {
    #[default]
    None,
    Pan,
    Pinch,
    PinchCancelled,
}

#[derive(Resource, Default)]
struct TouchTracker {
    pub camera_start_pos: Vec3,
    pub time_start_touch: f32,
    pub gesture_type: GestureType,

    pub zoom_pointer_normalized_screen_pos: Option<Vec2>,

    // Keeps track of position on last frame.
    // This is different from Touch.last_position as that only updates when there has been a movement
    pub last_touch_a: Option<Vec2>,
    pub last_touch_b: Option<Vec2>,
}

fn camera_touch_pan_zoom(
    touches_res: Res<Touches>,
    mut tracker: ResMut<TouchTracker>,
    config: Res<TouchCameraConfig>,
    time: Res<Time>,

    mut camera_q: Query<(
        &PanCam,
        &mut Transform,
        &mut OrthographicProjection,
        &mut Camera,
    )>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok((cam, mut pos, mut proj, camera)) = camera_q.get_single_mut() else {
        return;
    };

    if !cam.enabled {
        return;
    }

    let touches: Vec<&Touch> = touches_res.iter().collect();

    if touches.is_empty() {
        tracker.gesture_type = GestureType::None;
        tracker.last_touch_a = None;
        tracker.last_touch_b = None;
        tracker.zoom_pointer_normalized_screen_pos = None;
        return;
    }

    let window = primary_window.single();
    let window_size = Vec2::new(window.width(), window.height());

    if touches_res.any_just_released() {
        tracker.gesture_type = GestureType::PinchCancelled;
        tracker.last_touch_a = None;
        tracker.last_touch_b = None;
        tracker.zoom_pointer_normalized_screen_pos = None;
    }

    if touches.len() == 2 {
        if tracker.gesture_type != GestureType::Pinch {
            // Calculate the center position between touches[0].position() and touches[1].position()
            let center = (touches[0].position() + touches[1].position()) / 2.;
            let normalized = (center / window_size) * 2. - Vec2::ONE;

            tracker.zoom_pointer_normalized_screen_pos =
                Some(Vec2::new(normalized.x, -normalized.y));
        }

        tracker.gesture_type = GestureType::Pinch;

        // complicated way to reset previous position to prevent some bugs. Should simplify
        let last_a = if tracker.last_touch_b.is_none() {
            touches[0].position()
        } else {
            tracker.last_touch_a.unwrap_or(touches[0].position())
        };
        let last_b = if tracker.last_touch_b.is_none() {
            touches[1].position()
        } else {
            tracker.last_touch_b.unwrap_or(touches[1].position())
        };

        let delta_a = touches[0].position() - last_a;
        let delta_b = touches[1].position() - last_b;
        let delta_total = (delta_a + delta_b).length();
        let dot_delta = delta_a.dot(delta_b);
        if dot_delta > config.opposites_tolerance {
            return;
        }

        let distance_current = touches[0].position() - touches[1].position();
        let distance_prev = touches[0].previous_position() - touches[1].previous_position();
        let pinch_direction = distance_prev.length() - distance_current.length();

        let old_scale = proj.scale;

        proj.scale += proj.scale * pinch_direction.signum() * delta_total * config.zoom_sensitivity;

        proj.scale = proj.scale.max(cam.min_scale);

        // Apply max scale constraint
        if let Some(max_scale) = cam.max_scale {
            proj.scale = proj.scale.min(max_scale);
        }

        // If there is both a min and max boundary, that limits how far we can zoom. Make sure we don't exceed that
        let scale_constrained = BVec2::new(
            cam.min_x.is_some() && cam.max_x.is_some(),
            cam.min_y.is_some() && cam.max_y.is_some(),
        );

        if scale_constrained.x || scale_constrained.y {
            let bounds_width = if let (Some(min_x), Some(max_x)) = (cam.min_x, cam.max_x) {
                max_x - min_x
            } else {
                f32::INFINITY
            };

            let bounds_height = if let (Some(min_y), Some(max_y)) = (cam.min_y, cam.max_y) {
                max_y - min_y
            } else {
                f32::INFINITY
            };

            let bounds_size = vec2(bounds_width, bounds_height);
            let max_safe_scale = max_scale_within_bounds(bounds_size, &proj, window_size);

            if scale_constrained.x {
                proj.scale = proj.scale.min(max_safe_scale.x);
            }

            if scale_constrained.y {
                proj.scale = proj.scale.min(max_safe_scale.y);
            }
        }

        // Move the camera position to normalize the projection window
        if let (Some(zoom_pointer_normalized_screen_pos), true) = (
            tracker.zoom_pointer_normalized_screen_pos,
            cam.zoom_to_cursor,
        ) {
            let proj_size = proj.area.max / old_scale;
            let mouse_world_pos = pos.translation.truncate()
                + zoom_pointer_normalized_screen_pos * proj_size * old_scale;
            pos.translation = (mouse_world_pos
                - zoom_pointer_normalized_screen_pos * proj_size * proj.scale)
                .extend(pos.translation.z);

            // As we zoom out, we don't want the viewport to move beyond the provided boundary. If the most recent
            // change to the camera zoom would move cause parts of the window beyond the boundary to be shown, we
            // need to change the camera position to keep the viewport within bounds. The four if statements below
            // provide this behavior for the min and max x and y boundaries.
            let proj_size = proj.area.size();

            let half_of_viewport = proj_size / 2.;

            if let Some(min_x_bound) = cam.min_x {
                let min_safe_cam_x = min_x_bound + half_of_viewport.x;
                pos.translation.x = pos.translation.x.max(min_safe_cam_x);
            }
            if let Some(max_x_bound) = cam.max_x {
                let max_safe_cam_x = max_x_bound - half_of_viewport.x;
                pos.translation.x = pos.translation.x.min(max_safe_cam_x);
            }
            if let Some(min_y_bound) = cam.min_y {
                let min_safe_cam_y = min_y_bound + half_of_viewport.y;
                pos.translation.y = pos.translation.y.max(min_safe_cam_y);
            }
            if let Some(max_y_bound) = cam.max_y {
                let max_safe_cam_y = max_y_bound - half_of_viewport.y;
                pos.translation.y = pos.translation.y.min(max_safe_cam_y);
            }
        }

        if let Some(size) = camera.logical_viewport_size() {
            proj.update(size.x, size.y);
        }

        clamp_translation(cam, &mut proj, &mut pos, Vec2::default(), window_size);

        tracker.last_touch_a = Some(touches[0].position());
        tracker.last_touch_b = Some(touches[1].position());
    } else if touches.len() == 1
        && matches!(tracker.gesture_type, GestureType::None | GestureType::Pan)
    {
        // TODO: Add support for inertia
        if tracker.gesture_type == GestureType::None {
            tracker.camera_start_pos = pos.translation;
            tracker.time_start_touch = time.elapsed_seconds();
        }

        tracker.gesture_type = GestureType::Pan;
        let time_since_start = time.elapsed_seconds() - tracker.time_start_touch;
        if time_since_start < config.touch_time_min {
            return;
        }

        let last_a = tracker.last_touch_a.unwrap_or(touches[0].position());
        let delta = touches[0].position() - last_a;
        let delta_device_pixels = config.drag_sensitivity * Vec2::new(delta.x, -delta.y);

        clamp_translation(cam, &mut proj, &mut pos, delta_device_pixels, window_size);

        tracker.last_touch_a = Some(touches[0].position());
        tracker.last_touch_b = None;
    }
}

/// A component that adds panning camera controls to an orthographic camera
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PanCam {
    /// The mouse buttons that will be used to drag and pan the camera
    pub grab_buttons: Vec<MouseButton>,
    /// Whether camera currently responds to user input
    pub enabled: bool,
    /// When true, zooming the camera will center on the mouse cursor
    ///
    /// When false, the camera will stay in place, zooming towards the
    /// middle of the screen
    pub zoom_to_cursor: bool,
    /// The minimum scale for the camera
    ///
    /// The orthographic projection's scale will be clamped at this value when zooming in
    pub min_scale: f32,
    /// The maximum scale for the camera
    ///
    /// If present, the orthographic projection's scale will be clamped at
    /// this value when zooming out.
    pub max_scale: Option<f32>,
    /// The minimum x position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub min_x: Option<f32>,
    /// The maximum x position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub max_x: Option<f32>,
    /// The minimum y position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub min_y: Option<f32>,
    /// The maximum y position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub max_y: Option<f32>,
}

impl Default for PanCam {
    fn default() -> Self {
        Self {
            grab_buttons: vec![MouseButton::Left, MouseButton::Right, MouseButton::Middle],
            enabled: true,
            zoom_to_cursor: true,
            min_scale: 0.00001,
            max_scale: None,
            min_x: None,
            max_x: None,
            min_y: None,
            max_y: None,
        }
    }
}
