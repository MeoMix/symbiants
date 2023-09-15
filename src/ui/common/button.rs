use bevy::prelude::*;

pub struct HoveredButtonConfig {
    pub border_color: BorderColor,
    pub background_color: BackgroundColor,
}

pub struct PressedButtonConfig {
    pub border_color: BorderColor,
    pub background_color: BackgroundColor,
}

pub struct ButtonStates {
    pressed: PressedButtonConfig,
    hovered: HoveredButtonConfig,
}

pub struct ButtonConfig {
    pub style: Style,
    pub text_style: TextStyle,
    pub border_color: BorderColor,
    pub background_color: BackgroundColor,
    pub states: ButtonStates,
}

lazy_static! {
    pub static ref BUTTON: ButtonConfig = {
        ButtonConfig {
            style: Style {
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                padding: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            text_style: TextStyle {
                font_size: 40.0,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            background_color: BackgroundColor(Color::rgb(0.15, 0.15, 0.15)),

            states: ButtonStates {
                hovered: HoveredButtonConfig {
                    border_color: BorderColor(Color::BLACK),
                    background_color: BackgroundColor(Color::rgb(0.25, 0.25, 0.25)),
                },

                pressed: PressedButtonConfig {
                    border_color: BorderColor(Color::BLACK),
                    background_color: BackgroundColor(Color::rgb(0.75, 0.75, 0.35)),
                }
            },
        }
    };
}

// TODO: This is way too general
pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        // *color = match *interaction {
        //     Interaction::Pressed => BUTTON.states.pressed.background_color,
        //     Interaction::Hovered => BUTTON.states.hovered.background_color,
        //     Interaction::None => BUTTON.background_color,
        // }
    }
}
