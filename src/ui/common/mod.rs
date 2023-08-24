use bevy::prelude::*;

pub struct BackgroundColorConfig {
    pub normal: BackgroundColor,
    pub hovered: BackgroundColor,
    pub pressed: BackgroundColor,
}

pub struct BorderColorConfig {
    pub normal: BorderColor,
    pub hovered: BorderColor,
    pub pressed: BorderColor,
}

pub struct ButtonConfig {
    pub border_color: BorderColorConfig,
    pub background_color: BackgroundColorConfig,
}

pub const BUTTON: ButtonConfig = ButtonConfig {
    border_color: BorderColorConfig {
        normal: BorderColor(Color::BLACK),
        hovered: BorderColor(Color::BLACK),
        pressed: BorderColor(Color::BLACK),
    },

    background_color: BackgroundColorConfig {
        normal: BackgroundColor(Color::rgb(0.15, 0.15, 0.15)),
        hovered: BackgroundColor(Color::rgb(0.25, 0.25, 0.25)),
        pressed: BackgroundColor(Color::rgb(0.35, 0.75, 0.35)),
    },
};

pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => BUTTON.background_color.pressed,
            Interaction::Hovered => BUTTON.background_color.hovered,
            Interaction::None => BUTTON.background_color.normal,
        }
    }
}
