use bevy::prelude::*;


pub struct DialogHeaderConfig {
    pub style: Style,
}

pub struct DialogContentConfig {
    pub style: Style,
    pub text_style: TextStyle,
}

pub struct DialogFooterConfig {
    pub style: Style,
}

pub struct DialogConfig {
    pub style: Style,
    pub border_color: BorderColor,
    pub background_color: BackgroundColor,
    pub header: DialogHeaderConfig,
    pub content: DialogContentConfig,
    pub footer: DialogFooterConfig,
}

lazy_static! {
    pub static ref DIALOG: DialogConfig = DialogConfig {
        style: Style {
            width: Val::Percent(25.0),
            height: Val::Percent(50.0),
            max_width: Val::Percent(25.0),
            max_height: Val::Percent(50.0),
            min_height: Val::Px(400.0),
            border: UiRect::all(Val::Px(5.0)),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        border_color: BorderColor(Color::BLACK),
        background_color: BackgroundColor(Color::BLACK),

        // TODO: Add a header to the dialog with a close icon in the upper-right corner and a title in the upper-left corner.
        header: DialogHeaderConfig {
            style: Style {
                ..default()
            }
        },

        content: DialogContentConfig {
            style: Style {
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Auto,
                ..default()
            },

            text_style: TextStyle {
                font_size: 32.0,
                color: Color::RED,
                ..default()
            },
        },

        footer: DialogFooterConfig {
            style: Style {
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            }
        }
    };
}
