use bevy::prelude::*;

pub struct ContentConfig {
    pub text_style: TextStyle,
}

pub struct PanelConfig {
    pub content: ContentConfig,
}

lazy_static! {
    pub static ref PANEL: PanelConfig = PanelConfig {
        content: ContentConfig {
            text_style: TextStyle {
                font_size: 32.0,
                color: Color::RED,
                ..default()
            },
        }
    };
}
