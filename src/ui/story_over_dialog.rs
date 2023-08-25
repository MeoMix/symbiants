use bevy::prelude::*;

use crate::story_state::StoryState;

use super::common::{button::BUTTON, dialog::DIALOG, overlay::OVERLAY};

#[derive(Component)]
pub struct StoryOverDialogModalOverlay;

#[derive(Component)]
pub struct StoryOverDialog;

#[derive(Component)]
pub struct StoryOverDialogText;

#[derive(Component)]
pub enum StoryOverDialogAction {
    BeginNewStory,
}

pub fn setup_story_over_dialog(mut commands: Commands) {
    let modal_overlay_bundle = (
        NodeBundle {
            style: OVERLAY.style.clone(),
            background_color: OVERLAY.background_color.clone(),
            ..default()
        },
        StoryOverDialogModalOverlay,
    );

    let dialog_bundle = (
        NodeBundle {
            style: DIALOG.style.clone(),
            background_color: DIALOG.background_color,
            border_color: DIALOG.border_color,
            ..default()
        },
        StoryOverDialog,
    );

    let dialog_header_bundle = NodeBundle {
        style: DIALOG.header.style.clone(),
        ..default()
    };

    let dialog_content_bundle = NodeBundle {
        style: DIALOG.content.style.clone(),
        ..default()
    };

    let story_over_text_bundle = (
        TextBundle::from_sections([TextSection::new(
            "Queen has died. Sadge :(. Story over. Begin again?",
            DIALOG.content.text_style.clone(),
        )]),
        StoryOverDialogText,
    );

    let dialog_footer_bundle = NodeBundle {
        style: DIALOG.footer.style.clone(),
        ..default()
    };

    let begin_new_story_button_bundle = (
        ButtonBundle {
            style: BUTTON.style.clone(),
            border_color: BUTTON.border_color,
            background_color: BUTTON.background_color,
            ..default()
        },
        StoryOverDialogAction::BeginNewStory,
    );

    let begin_new_story_button_text_bundle =
        TextBundle::from_section("Begin New Story", BUTTON.text_style.clone());

    commands
        .spawn(modal_overlay_bundle)
        .with_children(|modal_overlay| {
            modal_overlay.spawn(dialog_bundle).with_children(|dialog| {
                dialog.spawn(dialog_header_bundle);

                dialog
                    .spawn(dialog_content_bundle)
                    .with_children(|dialog_content| {
                        dialog_content.spawn(story_over_text_bundle);
                    });

                dialog
                    .spawn(dialog_footer_bundle)
                    .with_children(|dialog_footer| {
                        dialog_footer
                            .spawn(begin_new_story_button_bundle)
                            .with_children(|begin_new_story_button| {
                                begin_new_story_button.spawn(begin_new_story_button_text_bundle);
                            });
                    });
            });
        });
}

pub fn on_interact_button(
    interaction_query: Query<
        (&Interaction, &StoryOverDialogAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut story_state: ResMut<NextState<StoryState>>,
) {
    for (interaction, dialog_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match dialog_button_action {
                StoryOverDialogAction::BeginNewStory => {
                    story_state.set(StoryState::Cleanup);
                }
            }
        }
    }
}
