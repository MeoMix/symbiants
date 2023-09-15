// Create a floating menu which contains a set of action icons. Very similar to Photoshop/Paint action menu.
// Used in Sandbox Mode to allow the user to play around with the environment - manually spawning/despawning anything that could exist.
use bevy::prelude::*;



#[derive(Component)]
pub struct ActionMenu;

#[derive(Component)]
pub struct ActionMenuButton(PointerAction);

#[derive(Component)]
pub struct SelectButton;

#[derive(Component)]
pub struct SpawnFoodButton;

#[derive(Component)]
pub struct SpawnDirtButton;

#[derive(Component)]
pub struct SpawnSandButton;

#[derive(Component)]
pub struct DespawnButton;

#[derive(Resource, Default, PartialEq, Copy, Clone, Debug)]
pub enum PointerAction {
    #[default]
    Select,
    Despawn,
    Food,
    Dirt,
    Sand,
}

pub fn create_action_menu(mut commands: Commands) {
    commands.init_resource::<PointerAction>();

    let menu_bundle = (
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                width: Val::Px(100.0),
                display: Display::Flex,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::SpaceBetween,
                align_content: AlignContent::FlexStart,
                ..default()
            },
            background_color: BackgroundColor(Color::BLACK),
            ..default()
        },
        ActionMenu,
    );

    // TODO: Separate the concept of icon from button and nest.

    let select_icon_bundle = (
        ButtonBundle {
            style: Style {
                width: Val::Px(48.0),
                height: Val::Px(48.0),
                min_width: Val::Px(48.0),
                min_height: Val::Px(48.0),
                ..default()
            },
            background_color: BackgroundColor(Color::WHITE),
            ..default()
        },
        ActionMenuButton(PointerAction::Select),
        SelectButton,
    );

    let food_icon_bundle = (
        ButtonBundle {
            style: Style {
                width: Val::Px(48.0),
                height: Val::Px(48.0),
                min_width: Val::Px(48.0),
                min_height: Val::Px(48.0),
                ..default()
            },
            background_color: BackgroundColor(Color::rgb(0.388, 0.584, 0.294)),
            ..default()
        },
        ActionMenuButton(PointerAction::Food),
        SpawnFoodButton,
    );

    let dirt_icon_bundle = (
        ButtonBundle {
            style: Style {
                width: Val::Px(48.0),
                height: Val::Px(48.0),
                min_width: Val::Px(48.0),
                min_height: Val::Px(48.0),
                ..default()
            },
            background_color: BackgroundColor(Color::rgb(0.514, 0.396, 0.224)),
            ..default()
        },
        ActionMenuButton(PointerAction::Dirt),
        SpawnDirtButton,
    );

    let sand_icon_bundle = (
        ButtonBundle {
            style: Style {
                width: Val::Px(48.0),
                height: Val::Px(48.0),
                min_width: Val::Px(48.0),
                min_height: Val::Px(48.0),
                ..default()
            },
            background_color: BackgroundColor(Color::rgb(0.761, 0.698, 0.502)),
            ..default()
        },
        ActionMenuButton(PointerAction::Sand),
        SpawnSandButton,
    );

    let despawn_icon_bundle = (
        ButtonBundle {
            style: Style {
                width: Val::Px(48.0),
                height: Val::Px(48.0),
                min_width: Val::Px(48.0),
                min_height: Val::Px(48.0),
                ..default()
            },
            background_color: BackgroundColor(Color::RED),
            ..default()
        },
        ActionMenuButton(PointerAction::Despawn),
        DespawnButton,
    );

    commands.spawn(menu_bundle).with_children(|menu| {
        // Add a bunch of child icons.
        menu.spawn(select_icon_bundle);
        menu.spawn(despawn_icon_bundle);
        menu.spawn(food_icon_bundle);
        menu.spawn(dirt_icon_bundle);
        menu.spawn(sand_icon_bundle);
    });
}

pub fn on_interact_action_menu_button(
    interaction_query: Query<
        (&Interaction, &ActionMenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut pointer_action: ResMut<PointerAction>
) {
    for (interaction, action_menu_button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("ayy lmao");
            *pointer_action = action_menu_button.0;
        }
    }
}
