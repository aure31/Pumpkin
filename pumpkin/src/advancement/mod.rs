use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::player::advancement::PlayerAdvancement;
use crate::entity::predicate::EntityPredicate;
use crate::world::loot::LootContextParameters;
use dashmap::{DashMap, DashSet};
use pumpkin_data::Advancement;
use pumpkin_data::entity::EntityType;
use pumpkin_util::identifier::Identifier;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

pub fn get_criterion(identifier: Identifier) {}

pub struct Criterion<T>
where
    T: 'static + Clone + Debug + CriterionTriggerInstance,
{
    trigger: &'static CriterionTrigger<T>,
    instance: &'static T,
}

pub struct CriterionTrigger<T>
where
    T: 'static + Clone + Debug + CriterionTriggerInstance,
{
    players: DashMap<i32, DashSet<Listener<T>>>,
}

impl<T> CriterionTrigger<T> {
    fn add_player_listener(&mut self, player: i32, listener: Listener<T>) {
        self.players
            .entry(player)
            .or_insert_with(HashSet::new)
            .insert(listener);
    }

    fn remove_player_listener(&mut self, player: &i32, listener: Listener<T>) {
        let listeners = self.players.get_mut(player);
        if let Some(listeners) = listeners {
            listeners.remove(&listener);
            if listeners.is_empty() {
                self.players.remove(player);
            }
        }
    }

    fn remove_player_listeners(&mut self, player: &i32) {
        self.players.remove(player);
    }

    pub async fn trigger(&self, player: Arc<Player>, matcher: &dyn Fn(T) -> bool) {
        let advancement = player.advancements.lock().await;
        let all_listeners = self.players.get(&player.entity_id());
        if let Some(all_listeners) = all_listeners
            && !all_listeners.is_empty()
        {
            let context = LootContextParameters {
                position: Some(player.position()),
                this_entity: Some(&EntityType::PLAYER),
                ..Default::default()
            };
            let listeners: Vec<Listener<T>> = Vec::new();
            for listener in all_listeners.value().iter() {
                let trigger = listener.trigger;
                if matcher(trigger) {
                    let predicate = trigger.player
                }
            }
        }
    }

    fn create_criterion(&'static self, instance: &'static T) -> Criterion<T> {
        Criterion {
            trigger: self,
            instance,
        }
    }
}

pub struct Listener<T>
where
    T: 'static + Clone + Debug + CriterionTriggerInstance,
{
    trigger: &'static T,
    advancement: &'static Advancement,
    criterion: &'static str,
}

impl<T> Listener<T> {
    pub fn run(&self, player: &mut PlayerAdvancement) {
        player.award(self.advancement, self.criterion);
    }
}

pub trait CriterionTriggerInstance {
    fn validate(ctx: LootContextParameters);

    fn player() -> Option<LootItemCondition> ;
}