use bevy::prelude::*;

use crate::{ant::{Hunger, AntRole}, food::FoodCount, map::WorldMap};

#[derive(Component)]
pub struct InfoPanel;

#[derive(Component)]
pub struct InfoPanelAntCountText;

#[derive(Component)]
pub struct InfoPanelAntHungerText;

#[derive(Component)]
pub struct InfoPanelFoodText;

#[derive(Component)]
pub struct InfoPanelDayText;

const BORDER_WIDTH: Val = Val::Px(5.0);
const FONT_SIZE: f32 = 32.0;
const FONT_COLOR: Color = Color::RED;

pub fn setup_info_panel(mut commands: Commands) {
    let default_text_style = TextStyle {
        font_size: FONT_SIZE,
        color: FONT_COLOR,
        ..default()
    };

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
                    TextSection::new("Ants:", default_text_style.clone()),
                    TextSection::from_style(default_text_style.clone()),
                ]),
                InfoPanelAntCountText,
            ));

            info_panel.spawn((
                TextBundle::from_sections([
                    TextSection::new("Hunger:", default_text_style.clone()),
                    TextSection::from_style(default_text_style.clone()),
                    TextSection::new("%", default_text_style.clone()),
                ]),
                InfoPanelAntHungerText,
            ));

            info_panel.spawn((
                TextBundle::from_sections([
                    TextSection::new("Food:", default_text_style.clone()),
                    TextSection::from_style(default_text_style.clone()),
                ]),
                InfoPanelFoodText,
            ));

            info_panel.spawn((
                TextBundle::from_sections([
                    TextSection::new("Day:", default_text_style.clone()),
                    TextSection::from_style(default_text_style.clone()),
                    TextSection::new(" of ", default_text_style.clone()),
                    TextSection::from_style(default_text_style.clone()),
                ]),
                InfoPanelDayText,
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

pub fn update_info_panel_food(
    mut text_query: Query<&mut Text, With<InfoPanelFoodText>>,
    food_count: Res<FoodCount>,
) {
    for mut text in &mut text_query {
        text.sections[1].value = food_count.0.to_string();
    }
}

pub fn update_info_panel_day(
    mut text_query: Query<&mut Text, With<InfoPanelDayText>>,
    world_map: Res<WorldMap>,
) {
    for mut text in &mut text_query {
        text.sections[1].value = world_map.days_old().to_string();
        text.sections[3].value = 3.to_string();
    }
}