use bevy::prelude::*;

pub struct ContentConfig {
    pub text_style: TextStyle,
}

pub struct PanelConfig {
    // pub border_color: BorderColorConfig,
    // pub background_color: BackgroundColorConfig,

    pub content: ContentConfig,
}

lazy_static! {
    pub static ref PANEL: PanelConfig = PanelConfig {
        // border_color: BorderColorConfig {
        //     normal: BorderColor(Color::BLACK),
        //     hovered: BorderColor(Color::BLACK),
        //     pressed: BorderColor(Color::BLACK),
        // },
    
        // background_color: BackgroundColorConfig {
        //     normal: BackgroundColor(Color::BLACK),
        //     hovered: BackgroundColor(Color::BLACK),
        //     pressed: BackgroundColor(Color::BLACK),
        // },
    
        content: ContentConfig {
            text_style: TextStyle {
                font_size: 32.0,
                color: Color::RED,
                ..default()
            },
        }
    };
}
