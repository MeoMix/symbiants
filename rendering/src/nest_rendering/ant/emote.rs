use super::AntSpriteContainer;
use crate::common::{ModelViewEntityMap, VisibleGrid};
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};
use simulation::{
    common::grid::Grid,
    nest_simulation::{
        ant::{
            emote::{Emote, EmoteType},
            sleep::Asleep,
            AntAteFoodEvent,
        },
        nest::{AtNest, Nest},
    },
    settings::Settings,
    story_time::DEFAULT_TICKS_PER_SECOND,
};

pub fn on_removed_emote(
    mut removed: RemovedComponents<Emote>,
    mut ant_view_query: Query<&mut AntSpriteContainer>,
    mut commands: Commands,
    nest_query: Query<&Grid, With<Nest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    for emote_view_entity in removed.read() {
        if let Ok(mut ant_sprite_container) = ant_view_query.get_mut(emote_view_entity) {
            // Surprisingly, Bevy doesn't fix parent/child relationship when despawning children, so do it manually.
            commands
                .entity(ant_sprite_container.emote_entity.unwrap())
                .remove_parent()
                .despawn();

            ant_sprite_container.emote_entity = None;
        }
    }
}

pub fn on_tick_emote(
    mut ant_view_query: Query<(Entity, &mut Emote), With<AtNest>>,
    mut commands: Commands,
    settings: Res<Settings>,
) {
    for (ant_view_entity, mut emote) in ant_view_query.iter_mut() {
        let rate_of_emote_expire =
            emote.max() / (settings.emote_duration * DEFAULT_TICKS_PER_SECOND) as f32;
        emote.tick(rate_of_emote_expire);

        if emote.is_expired() {
            commands.entity(ant_view_entity).remove::<Emote>();
        }
    }
}

pub fn on_added_ant_emote(
    mut ant_view_query: Query<(&Emote, &mut AntSpriteContainer), Added<Emote>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    nest_query: Query<&Grid, With<Nest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (emote, mut ant_sprite_container) in ant_view_query.iter_mut() {
        let texture = match emote.emote_type() {
            EmoteType::Asleep => asset_server.load("images/zzz.png"),
            EmoteType::FoodLove => asset_server.load("images/foodlove.png"),
        };

        let ant_emote_entity = commands
            .spawn(SpriteBundle {
                transform: Transform::from_xyz(0.75, 1.0, 1.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(1.0)),
                    ..default()
                },
                texture,
                ..default()
            })
            .id();

        commands
            .entity(ant_sprite_container.sprite_entity)
            .push_children(&[ant_emote_entity]);
        ant_sprite_container.emote_entity = Some(ant_emote_entity);
    }
}

pub fn on_ant_ate_food(
    mut ant_action_events: EventReader<AntAteFoodEvent>,
    mut commands: Commands,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    for AntAteFoodEvent(ant_model_entity) in ant_action_events.read() {
        let ant_view_entity = match model_view_entity_map.get(ant_model_entity) {
            Some(ant_view_entity) => *ant_view_entity,
            None => continue,
        };

        commands
            .entity(ant_view_entity)
            .insert(Emote::new(EmoteType::FoodLove));
    }
}

pub fn ants_sleep_emote(
    ants_query: Query<Entity, (With<Asleep>, With<AtNest>)>,
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
    settings: Res<Settings>,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    for ant_model_entity in ants_query.iter() {
        if rng.f32() < settings.probabilities.sleep_emote {
            let ant_view_entity = match model_view_entity_map.get(&ant_model_entity) {
                Some(ant_view_entity) => *ant_view_entity,
                None => continue,
            };

            commands
                .entity(ant_view_entity)
                .insert(Emote::new(EmoteType::Asleep));
        }
    }
}

pub fn on_ant_wake_up(
    // TODO: Maybe this should be event-driven and have an AntWakeUpEvent instead? That way this won't run when ants are getting destroyed.
    mut removed: RemovedComponents<Asleep>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    mut commands: Commands,
) {
    for ant_model_entity in removed.read() {
        if let Some(view_entity) = model_view_entity_map.get(&ant_model_entity) {
            // TODO: This code isn't great because it's presumptuous. There's not a guarantee that sleeping ant was showing an emote
            // and, if it is showing an emote, maybe that emote isn't the Asleep emote. Still, this assumption holds for now.
            commands.entity(*view_entity).remove::<Emote>();
        }
    }
}
