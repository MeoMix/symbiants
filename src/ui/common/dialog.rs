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

pub struct DialogConfig {
    pub border_color: BorderColorConfig,
    pub background_color: BackgroundColorConfig,
}

pub const DIALOG: DialogConfig = DialogConfig {
    border_color: BorderColorConfig {
        normal: BorderColor(Color::BLACK),
        hovered: BorderColor(Color::BLACK),
        pressed: BorderColor(Color::BLACK),
    },

    background_color: BackgroundColorConfig {
        normal: BackgroundColor(Color::BLACK),
        hovered: BackgroundColor(Color::BLACK),
        pressed: BackgroundColor(Color::BLACK),
    },
};