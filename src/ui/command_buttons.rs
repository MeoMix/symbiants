use bevy::prelude::*;

use crate::{grid::save::delete_save, story_state::StoryState};


use crate::food::FoodCount;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);


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
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    FoodButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Food",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });

            // Reset Button
            button_container
                .spawn((
                    ButtonBundle {
                        style: Style {
                            // position_type: PositionType::Absolute,
                            // right: Val::Px(0.0),
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    ResetButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Reset",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
}

pub fn handle_reset_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ResetButton>)>,
    mut story_state: ResMut<NextState<StoryState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("gogo");
            delete_save();

            story_state.set(StoryState::NotStarted);
        }
    }
}

pub fn update_food_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut food_count: ResMut<FoodCount>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;
                food_count.0 += 1;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}
