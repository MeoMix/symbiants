use bevy::prelude::*;

use crate::{time::{IsFastForwarding, PendingTicks}, ui::common::{overlay::OVERLAY, dialog::DIALOG}};

#[derive(Component)]
pub struct LoadingDialog;

#[derive(Component)]
pub struct LoadingDialogText;

// Don't flicker the dialogs visibility when processing a small number of ticks
const MIN_PENDING_TICKS: isize = 1000;

pub fn update_loading_dialog(
    mut text_query: Query<&mut Text, With<LoadingDialogText>>,
    dialog_query: Query<Entity, With<LoadingDialog>>,
    pending_ticks: Res<PendingTicks>,
    is_fast_forwarding: Res<IsFastForwarding>,
    mut commands: Commands,
) {
    if is_fast_forwarding.is_changed() {
        if is_fast_forwarding.0 && pending_ticks.0 > MIN_PENDING_TICKS {
            commands
            .spawn((
                NodeBundle {
                    style: OVERLAY.style.clone(),
                    background_color: OVERLAY.background_color.clone(),
                    ..default()
                },
                LoadingDialog,
            ))
            .with_children(|dialog_container| {
                dialog_container
                    .spawn(NodeBundle {
                        style: DIALOG.style.clone(),
                        background_color: DIALOG.background_color,
                        border_color: DIALOG.border_color,
                        ..default()
                    })
                    .with_children(|dialog| {
                        let minutes = pending_ticks.as_minutes();

                        dialog.spawn((
                            TextBundle::from_sections([
                                TextSection::new(&format!("You were gone for {:.0} minutes.\nPlease wait while this time is simulated.\nRemaining ticks:", minutes), DIALOG.content.text_style.clone()),
                                TextSection::new(pending_ticks.0.to_string(), DIALOG.content.text_style.clone())
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
