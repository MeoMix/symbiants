use super::AntSpriteContainer;
use crate::common::{visible_grid::VisibleGrid, ModelViewEntityMap};
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};
use simulation::{
    common::{ant::AntAteFoodEvent, grid::Grid},
    nest_simulation::{
        ant::sleep::Asleep,
        nest::{AtNest, Nest},
    },
    settings::Settings,
    story_time::{StoryTime, DEFAULT_TICKS_PER_SECOND},
};

#[derive(Component, Debug, PartialEq, Copy, Clone)]
pub enum EmoteType {
    Asleep,
    FoodLove,
}

#[derive(Component, Debug, PartialEq, Copy, Clone)]
pub struct Emote {
    emote_type: EmoteType,
    expires_at: isize,
}

impl Emote {
    pub fn new(emote_type: EmoteType, expires_at: isize) -> Self {
        Self {
            emote_type,
            expires_at,
        }
    }

    pub fn emote_type(&self) -> EmoteType {
        self.emote_type
    }

    pub fn expires_at(&self) -> isize {
        self.expires_at
    }
}

pub fn on_removed_ant_emote(
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

pub fn despawn_expired_emotes(
    ant_view_query: Query<(Entity, &Emote), (With<AtNest>, With<AntSpriteContainer>)>,
    mut commands: Commands,
    story_time: Res<StoryTime>,
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

    for (ant_view_entity, emote) in ant_view_query.iter() {
        if story_time.elapsed_ticks() < emote.expires_at() {
            continue;
        }

        commands.entity(ant_view_entity).remove::<Emote>();
    }
}

pub fn on_ant_ate_food(
    mut ant_action_events: EventReader<AntAteFoodEvent>,
    mut commands: Commands,
    model_view_entity_map: Res<ModelViewEntityMap>,
    story_time: Res<StoryTime>,
    settings: Res<Settings>,
) {
    for AntAteFoodEvent(ant_model_entity) in ant_action_events.read() {
        let ant_view_entity = match model_view_entity_map.get(ant_model_entity) {
            Some(ant_view_entity) => *ant_view_entity,
            None => continue,
        };

        commands.entity(ant_view_entity).insert(Emote::new(
            EmoteType::FoodLove,
            story_time.elapsed_ticks() + (settings.emote_duration * DEFAULT_TICKS_PER_SECOND),
        ));
    }
}

/// Periodically show sleeping emotes above sleeping ants.
pub fn ants_sleep_emote(
    ants_query: Query<Entity, (With<Asleep>, With<AtNest>)>,
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
    settings: Res<Settings>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    story_time: Res<StoryTime>,
) {
    for ant_model_entity in ants_query.iter() {
        let ant_view_entity = match model_view_entity_map.get(&ant_model_entity) {
            Some(ant_view_entity) => *ant_view_entity,
            None => continue,
        };

        if rng.f32() >= settings.probabilities.sleep_emote {
            continue;
        }
        commands.entity(ant_view_entity).insert(Emote::new(
            EmoteType::Asleep,
            story_time.elapsed_ticks() + (settings.emote_duration * DEFAULT_TICKS_PER_SECOND),
        ));
    }
}

/// Despawn sleeping emote when ants wake up. It's possible they weren't showing an emote, so need to be a little careful.
pub fn on_ant_wake_up(
    // TODO: Maybe this should be event-driven and have an AntWakeUpEvent instead? That way this won't run when ants are getting destroyed.
    mut removed: RemovedComponents<Asleep>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    mut commands: Commands,
    ant_view_query: Query<Option<&Emote>, With<AntSpriteContainer>>,
) {
    for ant_model_entity in removed.read() {
        if let Some(&ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            let existing_emote = ant_view_query.get(ant_view_entity).unwrap();

            if let Some(emote) = existing_emote {
                if emote.emote_type() == EmoteType::Asleep {
                    commands.entity(ant_view_entity).remove::<Emote>();
                }
            }
        }
    }
}
