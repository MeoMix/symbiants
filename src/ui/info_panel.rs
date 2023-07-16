use bevy::prelude::*;

use crate::ant::{Hunger, AntRole};

#[derive(Component)]
pub struct InfoPanel;

#[derive(Component)]
pub struct InfoPanelAntCountText;

#[derive(Component)]
pub struct InfoPanelAntHungerText;

pub fn setup_info_panel(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(200.0),
                    height: Val::Px(200.0),
                    flex_direction: FlexDirection::Column,
                    // align_items: AlignItems::Center,
                    // justify_content: JustifyContent::Center,
                    border: UiRect::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 1.0, 0.0).into(),
                border_color: Color::BLACK.into(),
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
                                font_size: 32.0,
                                color: Color::RED,
                                ..default()
                            },
                        ),
                        TextSection::from_style(TextStyle {
                            font_size: 32.0,
                            color: Color::RED,
                            ..default()
                        }),
                    ]),
                    ..default()
                },
                InfoPanelAntCountText,
            ));

            info_panel.spawn((
                TextBundle {
                    text: Text::from_sections([
                        TextSection::new(
                            "Hunger:",
                            TextStyle {
                                font_size: 32.0,
                                color: Color::RED,
                                ..default()
                            },
                        ),
                        TextSection::from_style(TextStyle {
                            font_size: 32.0,
                            color: Color::RED,
                            ..default()
                        }),
                        TextSection::new(
                            "%",
                            TextStyle {
                                font_size: 32.0,
                                color: Color::RED,
                                ..default()
                            },
                        ),
                    ]),
                    ..default()
                },
                InfoPanelAntHungerText,
            ));
        });
}

pub fn update_info_panel_ant_count(
    mut text_query: Query<&mut Text, With<InfoPanelAntCountText>>,
    ant_query: Query<Entity, With<AntRole>>,
) {
    for mut text in &mut text_query {
        text.sections[1].value = ant_query.iter().count().to_string();
    }
}

pub fn update_info_panel_ant_hunger(
    mut text_query: Query<&mut Text, With<InfoPanelAntHungerText>>,
    ant_query: Query<&Hunger, With<AntRole>>,
) {
    for mut text in &mut text_query {
        let hunger_sum: f64 = ant_query.iter().map(|hunger: &Hunger| hunger.as_percent()).sum();
        let hunger_avg = hunger_sum / ant_query.iter().count() as f64;

        text.sections[1].value = format!("{:.0}", hunger_avg);
    }
}