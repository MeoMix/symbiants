use bevy::prelude::*;

use crate::{
    ant::{hunger::Hunger, AntRole},
    element::Food,
    ui::common::panel::PANEL,
};

#[derive(Component)]
pub struct InfoPanel;

#[derive(Component)]
pub struct InfoPanelAntCountText;

#[derive(Component)]
pub struct InfoPanelAntQueenHungerText;

#[derive(Component)]
pub struct InfoPanelFoodText;

const BORDER_WIDTH: Val = Val::Px(5.0);

pub fn setup_info_panel(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(400.0),
                    height: Val::Px(200.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(BORDER_WIDTH),
                    ..default()
                },
                border_color: Color::BLACK.into(),
                ..default()
            },
            InfoPanel,
        ))
        .with_children(|info_panel| {
            info_panel.spawn((
                TextBundle::from_sections([
                    TextSection::new("Ants:", PANEL.content.text_style.clone()),
                    TextSection::from_style(PANEL.content.text_style.clone()),
                ]),
                InfoPanelAntCountText,
            ));

            info_panel.spawn((
                TextBundle::from_sections([
                    TextSection::new("Queen Hunger:", PANEL.content.text_style.clone()),
                    TextSection::from_style(PANEL.content.text_style.clone()),
                    TextSection::new("%", PANEL.content.text_style.clone()),
                ]),
                InfoPanelAntQueenHungerText,
            ));

            info_panel.spawn((
                TextBundle::from_sections([
                    TextSection::new("Food:", PANEL.content.text_style.clone()),
                    TextSection::from_style(PANEL.content.text_style.clone()),
                ]),
                InfoPanelFoodText,
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
    mut text_query: Query<&mut Text, With<InfoPanelAntQueenHungerText>>,
    ant_query: Query<(&AntRole, &Hunger)>,
) {
    let mut text = text_query.single_mut();
    let queen_ant = ant_query.iter().find(|(&role, _)| role == AntRole::Queen);

    if let Some((_, hunger)) = queen_ant {
        text.sections[1].value = format!("{:.0}", hunger.value());
    }
}

pub fn update_info_panel_food(
    mut text_query: Query<&mut Text, With<InfoPanelFoodText>>,
    food_query: Query<&Food>,
) {
    for mut text in &mut text_query {
        text.sections[1].value = food_query.iter().len().to_string();
    }
}
