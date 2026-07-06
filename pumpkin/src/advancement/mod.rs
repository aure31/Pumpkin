use crate::entity::player::Player;
use crate::world::loot::{LootConditionExt, LootContextParameters};
use pumpkin_data::entity::EntityType;
use pumpkin_data::Block;
use pumpkin_util::loot_table::LootCondition;
use std::sync::Arc;
use pumpkin_data::item_stack::ItemStack;

pub struct Criterion {
    trigger: &'static CriterionTrigger,
    instance: &'static CriterionTriggerInstance,
}

pub struct CriterionTrigger {
    id: &'static str,
}

impl CriterionTrigger {
    pub async fn trigger(&self, player: Arc<Player>, matcher: &dyn Fn(CriterionTriggerInstance) -> bool) {
        let advancement = player.advancements.lock().await;
        let all_listeners = self.advancement.get(&player.entity_id());
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

pub enum CriterionTriggerInstanceTypes {
    AnyBlockInteraction{
        location: Option<ContextAwarePredicate>
    },
    BeeNestDestroy{
        block: Option<Block>,
        item: Option<ItemStack>

    }


}

impl CriterionTriggerInstanceTypes {
    pub fn matches(&self, context: &LootContextParameters) -> bool {
        match self {
            Self::AnyBlockInteraction{ location } => {
                location.is_empty() || location.unwrap().matches(context)
            }
        }
    }
}

pub struct CriterionTriggerInstance {
    pub player_context: Option<ContextAwarePredicate>,
    pub criterion_type: CriterionTriggerInstanceTypes,
}

impl CriterionTriggerInstance {

    pub fn matches(&self, loot_context_parameters: &LootContextParameters) -> bool {

    }

}

pub struct ContextAwarePredicate {
    conditions: Vec<LootCondition>,
}

impl ContextAwarePredicate {
    pub fn new(conditions: Vec<LootCondition>) -> Self {
        Self {
            conditions,
        }
    }

    pub fn matches(&self, context:&LootContextParameters) ->bool{
        for condition in &self.conditions {
            if !condition.is_fulfilled(context)  {
                return false;
            }
        }
        true
    }
}