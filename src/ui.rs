use bevy::prelude::*;

use crate::{
    ant::Hunger,
    time::{IsFastForwarding, PendingTicks},
};

#[derive(Component)]
pub struct LoadingDialog;

#[derive(Component)]
pub struct LoadingDialogText;

// TODO: Does this flicker if I always render it?
pub fn setup_loading_text_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
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
                        ..default()
                    },
                    background_color: Color::BLACK.into(),
                    ..default()
                })
                .with_children(|dialog| {
                    dialog.spawn((
                        TextBundle {
                            text: Text::from_sections([
                                TextSection::new(
                                    "Fast-Forwarding Time While You Were Away. Ticks Remaining:",
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                        font_size: 64.0,
                                        color: Color::RED,
                                    },
                                ),
                                TextSection::from_style(TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 64.0,
                                    color: Color::RED,
                                }),
                            ]),
                            ..default()
                        }
                        .with_style(Style {
                            max_width: Val::Px(400.),
                            ..default()
                        }),
                        LoadingDialogText,
                    ));
                });
        });
}

pub fn loading_text_update_system(
    mut text_query: Query<&mut Text, With<LoadingDialogText>>,
    dialog_query: Query<Entity, With<LoadingDialog>>,
    pending_ticks: Res<PendingTicks>,
    is_fast_forwarding: Res<IsFastForwarding>,
    mut commands: Commands,
) {
    if is_fast_forwarding.0 {
        for mut text in &mut text_query {
            text.sections[1].value = pending_ticks.0.to_string();
        }
    } else if is_fast_forwarding.is_changed() {
        commands.entity(dialog_query.single()).despawn_recursive();
    }
}

#[derive(Component)]
pub struct InfoPanel;

#[derive(Component)]
pub struct InfoPanelText;

pub fn setup_info_panel(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(200.0),
                    height: Val::Px(200.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 1.0, 0.0).into(),
                ..default()
            },
            InfoPanel,
        ))
        .with_children(|info_panel| {
            info_panel.spawn((
                TextBundle {
                    text: Text::from_sections([
                        TextSection::new(
                            "Ants:",
                            TextStyle {
                                font_size: 64.0,
                                color: Color::RED,
                                ..default()
                            },
                        ),
                        TextSection::from_style(TextStyle {
                            font_size: 64.0,
                            color: Color::RED,
                            ..default()
                        }),
                    ]),
                    ..default()
                }
                .with_style(Style {
                    max_width: Val::Px(400.),
                    ..default()
                }),
                InfoPanelText,
            ));
        });
}

pub fn update_info_panel_system(
    mut text_query: Query<&mut Text, With<InfoPanelText>>,
    ant_query: Query<Entity, With<Hunger>>,
) {
    for mut text in &mut text_query {
        text.sections[1].value = ant_query.iter().count().to_string();
    }
}
