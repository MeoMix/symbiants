use bevy::prelude::*;

use crate::story_state::StoryState;

use crate::ui::common::button::BUTTON;

#[derive(Component)]
pub struct CommandButtons;

#[derive(Component)]
pub struct ResetButton;

pub fn setup_command_buttons(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: Val::Auto,
                    ..default()
                },
                ..default()
            },
            CommandButtons,
        ))
        .with_children(|button_container| {
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
