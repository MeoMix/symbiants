use bevy::prelude::*;

use crate::time::{IsFastForwarding, PendingTicks};

#[derive(Component)]
pub struct LoadingDialog;

#[derive(Component)]
pub struct LoadingDialogText;

// Don't flicker the dialogs visibility when processing a small number of ticks
const MIN_PENDING_TICKS: isize = 1000;

const BORDER_WIDTH: Val = Val::Px(5.0);
const FONT_SIZE: f32 = 32.0;
const FONT_COLOR: Color = Color::RED;

pub fn update_loading_dialog(
    mut text_query: Query<&mut Text, With<LoadingDialogText>>,
    dialog_query: Query<Entity, With<LoadingDialog>>,
    pending_ticks: Res<PendingTicks>,
    is_fast_forwarding: Res<IsFastForwarding>,
    mut commands: Commands,
) {
    let default_text_style = TextStyle {
        font_size: FONT_SIZE,
        color: FONT_COLOR,
        ..default()
    };

    if is_fast_forwarding.is_changed() {
        if is_fast_forwarding.0 && pending_ticks.0 > MIN_PENDING_TICKS {
            commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    background_color: Color::rgba(0.0, 0.0, 0.0, 0.8).into(),
                    ..default()
                },
                LoadingDialog,
            ))
            .with_children(|dialog_container| {
                dialog_container
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(25.0),
                            height: Val::Percent(50.0),
                            max_width: Val::Percent(25.0),
                            max_height: Val::Percent(50.0),
                            border: UiRect::all(BORDER_WIDTH),
                            ..default()
                        },
                        background_color: Color::BLACK.into(),
                        border_color: Color::BLACK.into(),
                        ..default()
                    })
                    .with_children(|dialog| {
                        let minutes = pending_ticks.as_minutes();

                        dialog.spawn((
                            TextBundle::from_sections([
                                TextSection::new(&format!("You were gone for {:.0} minutes.\nPlease wait while this time is simulated.\nRemaining ticks:", minutes), default_text_style.clone()),
                                TextSection::new(pending_ticks.0.to_string(), default_text_style.clone())
                            ]),
                            LoadingDialogText,
                        ));
                    });
            });
        } else {
            let dialog = match dialog_query.get_single() {
                Ok(dialog) => dialog,
                Err(_) => return,
            };

            commands.entity(dialog).despawn_recursive();
        }
    } else if is_fast_forwarding.0 {
        for mut text in &mut text_query {
            text.sections[1].value = pending_ticks.0.to_string();
        }
    }
}
