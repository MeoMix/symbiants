use bevy::prelude::*;

use crate::story_state::StoryState;

use crate::food::FoodCount;

use super::common::button::BUTTON;

#[derive(Component)]
pub struct ResetButton;

#[derive(Component)]
pub struct FoodButton;

pub fn setup_command_buttons(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Auto,
                ..default()
            },
            ..default()
        })
        .with_children(|button_container| {
            // Food Button
            button_container
                .spawn((
                    ButtonBundle {
                        style: BUTTON.style.clone(),
                        border_color: BUTTON.border_color,
                        background_color: BUTTON.background_color,
                        ..default()
                    },
                    FoodButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Food", BUTTON.text_style.clone()));
                });

            // Reset Button
            button_container
                .spawn((
                    ButtonBundle {
                        style: BUTTON.style.clone(),
                        border_color: BUTTON.border_color,
                        background_color: BUTTON.background_color,
                        ..default()
                    },
                    ResetButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Reset", BUTTON.text_style.clone()));
                });
        });
}

pub fn handle_reset_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ResetButton>)>,
    mut story_state: ResMut<NextState<StoryState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            story_state.set(StoryState::Cleanup);
        }
    }
}

pub fn update_food_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<FoodButton>)>,
    mut food_count: ResMut<FoodCount>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            food_count.0 += 1;
        }
    }
}
