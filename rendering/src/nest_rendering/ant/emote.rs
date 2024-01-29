use crate::common::{ModelViewEntityMap, VisibleGrid};

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

use bevy::{prelude::*, utils::HashSet};
use bevy_turborand::{DelegatedRng, GlobalRng};

use super::AntSprite;

#[derive(Component)]
pub struct EmoteSprite {
    parent_entity: Entity,
}

// TODO: Invert ownership would make this O(1) instead of O(n).
pub fn on_removed_emote(
    mut removed: RemovedComponents<Emote>,
    emote_view_query: Query<(Entity, &EmoteSprite)>,
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

    let emoting_view_entities = &mut removed.read().collect::<HashSet<_>>();

    for (emote_view_entity, emote_sprite) in emote_view_query.iter() {
        if emoting_view_entities.contains(&emote_sprite.parent_entity) {
            // Surprisingly, Bevy doesn't fix parent/child relationship when despawning children, so do it manually.
            commands.entity(emote_view_entity).remove_parent().despawn();
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
    ant_view_query: Query<(Entity, &Emote, &Children), Added<Emote>>,
    ant_sprite_view_query: Query<&AntSprite>,
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

    for (ant_view_entity, emote, children) in ant_view_query.iter() {
        if let Some(ant_sprite_view_entity) = children
            .iter()
            .find(|&&child| ant_sprite_view_query.get(child).is_ok())
        {
            commands
                .entity(*ant_sprite_view_entity)
                .with_children(|parent| {
                    let texture = match emote.emote_type() {
                        EmoteType::Asleep => asset_server.load("images/zzz.png"),
                        EmoteType::FoodLove => asset_server.load("images/foodlove.png"),
                    };

                    parent.spawn((
                        EmoteSprite {
                            parent_entity: ant_view_entity,
                        },
                        SpriteBundle {
                            transform: Transform::from_xyz(0.75, 1.0, 1.0),
                            sprite: Sprite {
                                custom_size: Some(Vec2::splat(1.0)),
                                ..default()
                            },
                            texture,
                            ..default()
                        },
                    ));
                });
        }
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
