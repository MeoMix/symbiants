use bevy::prelude::*;

pub struct OverlayConfig {
    pub style: Style,
    pub background_color: BackgroundColor,
}

lazy_static! {
    pub static ref OVERLAY: OverlayConfig = OverlayConfig {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.8)),
    };
}
