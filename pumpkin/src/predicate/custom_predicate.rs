use crate::entity::NBTStorage;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::data_component_impl::{
    CustomDataImpl, EnchantmentsImpl, FireworkExplosionImpl, FireworkExplosionShape, Modifier,
    Operation,
};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{AttributeModifierSlot, Enchantment};
use pumpkin_nbt::NbtCompound;
use pumpkin_util::math::bounds::{DoubleBounds, IntBounds};

pub struct ModifierPredicate {
    attribute: Option<Vec<&'static Attributes>>,
    id: Option<&'static str>,
    amount: DoubleBounds,
    operation: Option<Operation>,
    slot: Option<AttributeModifierSlot>,
}

impl ModifierPredicate {
    fn test(&self, value: &Modifier) -> bool {
        self.attribute
            .as_ref()
            .is_none_or(|attribute| attribute.contains(&value.r#type))
            && self.id.as_ref().is_none_or(|id| id == &value.id)
            && self.amount.matches(value.amount)
            && self
                .operation
                .as_ref()
                .is_none_or(|operation| operation == &value.operation)
            && self.slot.as_ref().is_none_or(|slot| slot == &value.slot)
    }
}

pub struct EnchantmentPredicate {
    enchantments: Option<Vec<&'static Enchantment>>,
    level: IntBounds,
}

impl EnchantmentPredicate {
    pub fn contained_in(&self, item_enchantments: &EnchantmentsImpl) -> bool {
        if let Some(enchantments) = &self.enchantments {
            for enchantment in enchantments {
                if self.matches_enchantment(item_enchantments, enchantment) {
                    return true;
                }
            }
            false
        } else if self.level != IntBounds::ANY {
            for &(_, level) in item_enchantments.enchantment.iter() {
                if self.level.matches(level) {
                    return true;
                }
            }
            false
        } else {
            !item_enchantments.enchantment.is_empty()
        }
    }

    fn matches_enchantment(
        &self,
        item_enchantments: &EnchantmentsImpl,
        enchantment: &Enchantment,
    ) -> bool {
        let level = item_enchantments.get_level(enchantment);
        level != 0 && (self.level == IntBounds::ANY || self.level.matches(level))
    }
}

pub struct FireworkPredicate {
    shape: Option<FireworkExplosionShape>,
    twinkle: Option<bool>,
    trail: Option<bool>,
}

impl FireworkPredicate {
    pub fn test(&self, firework_explosion: &FireworkExplosionImpl) -> bool {
        self.shape
            .as_ref()
            .is_none_or(|shape| shape == &firework_explosion.shape)
            && self
                .twinkle
                .as_ref()
                .is_none_or(|twinkle| twinkle == &firework_explosion.has_twinkle)
            && self
                .trail
                .as_ref()
                .is_none_or(|trail| trail == &firework_explosion.has_trail)
    }
}

pub struct NbtPredicate(NbtCompound);

impl NbtPredicate {
    pub async fn matches_storage(&self, storage: &dyn NBTStorage) -> bool {
        let mut output = NbtCompound::new();
        storage.write_nbt(&mut output).await;
        self.0 == output
    }

    pub fn matches_item(&self, item: &ItemStack) -> bool {
        let data: Option<&CustomDataImpl> = item.get_data_component();
        self.0.is_empty() || data.is_some_and(|data| data.data == self.0)
    }
}
