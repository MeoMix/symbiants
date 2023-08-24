use bevy::prelude::*;

use crate::{grid::save::delete_save, story_state::StoryState};

use super::common::BUTTON;

#[derive(Component)]
pub struct StoryOverDialogModalOverlay;

#[derive(Component)]
pub struct StoryOverDialog;

#[derive(Component)]
pub struct StoryOverDialogText;

#[derive(Component)]
pub enum DialogButtonAction {
    BeginNewStory,
}

const BORDER_WIDTH: Val = Val::Px(5.0);
const FONT_SIZE: f32 = 32.0;
const FONT_COLOR: Color = Color::RED;

pub fn setup_story_over_dialog(mut commands: Commands) {
    let default_text_style = TextStyle {
        font_size: FONT_SIZE,
        color: FONT_COLOR,
        ..default()
    };

    let modal_overlay_bundle = (
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
        StoryOverDialogModalOverlay,
    );

    let dialog_bundle = (
        NodeBundle {
            style: Style {
                width: Val::Percent(25.0),
                height: Val::Percent(50.0),
                max_width: Val::Percent(25.0),
                max_height: Val::Percent(50.0),
                min_height: Val::Px(400.0),
                border: UiRect::all(BORDER_WIDTH),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::BLACK.into(),
            border_color: Color::BLACK.into(),
            ..default()
        },
        StoryOverDialog,
    );

    let dialog_header_bundle = NodeBundle { ..default() };

    let dialog_content_bundle = NodeBundle {
        style: Style {
            flex_grow: 1.0,
            flex_shrink: 1.0,
            flex_basis: Val::Auto,
            ..default()
        },
        ..default()
    };

    let story_over_text_bundle = (
        TextBundle::from_sections([TextSection::new(
            &format!("Your queen died. Sadge. Story over."),
            default_text_style.clone(),
        )]),
        StoryOverDialogText,
    );

    let dialog_footer_bundle = NodeBundle {
        style: Style {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    };

    let begin_new_story_button_bundle = (
        ButtonBundle {
            style: Style {
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                padding: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            border_color: BUTTON.border_color.normal,
            background_color: BUTTON.background_color.normal,
            ..default()
        },
        DialogButtonAction::BeginNewStory,
    );

    let begin_new_story_button_text_bundle = TextBundle::from_section(
        "Begin New Story",
        TextStyle {
            font_size: 40.0,
            color: Color::rgb(0.9, 0.9, 0.9),
            ..default()
        },
    );

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

pub fn handle_story_over_dialog_button_interactions(
    interaction_query: Query<
        (&Interaction, &DialogButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut story_state: ResMut<NextState<StoryState>>,
) {
    for (interaction, dialog_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match dialog_button_action {
                DialogButtonAction::BeginNewStory => {
                    delete_save();

                    story_state.set(StoryState::NotStarted);

                    // TODO: destroy and recreate WorldMap from settings
                }
            }
        }
    }
}
