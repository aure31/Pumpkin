//! Predicates for concrete item, enchantment, firework, and NBT checks.
//!
//! The helpers stay fairly small so they can be combined from higher-level
//! predicate builders without duplicating the matching logic.
use crate::attributes::Attributes;
use crate::data_component_impl::{
    CustomDataImpl, EnchantmentsImpl, FireworkExplosionImpl, FireworkExplosionShape, Modifier,
    Operation,
};
use crate::item_stack::ItemStack;
use crate::predicate::Predicate;
use crate::{AttributeModifierSlot, Enchantment};
use pumpkin_nbt::NbtCompound;
use pumpkin_util::math::bounds::{DoubleBounds, IntBounds};

/// Matches a single attribute modifier against a few optional filters.
pub struct ModifierPredicate {
    attribute: Option<Vec<&'static Attributes>>,
    id: Option<&'static str>,
    amount: DoubleBounds,
    operation: Option<Operation>,
    slot: Option<AttributeModifierSlot>,
}

impl Predicate for ModifierPredicate {
    type Item = Modifier;
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

/// Matches an enchantment list with either an id filter, a level filter, or both.
pub struct EnchantmentPredicate {
    enchantments: Option<Vec<&'static Enchantment>>,
    level: IntBounds,
}

impl EnchantmentPredicate {
    #[must_use]
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

/// Matches a firework explosion by shape and flags.
pub struct FireworkPredicate {
    shape: Option<FireworkExplosionShape>,
    twinkle: Option<bool>,
    trail: Option<bool>,
}

impl Predicate for FireworkPredicate {
    type Item = FireworkExplosionImpl;
    fn test(&self, firework_explosion: &FireworkExplosionImpl) -> bool {
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

/// Compares NBT data against an item or an arbitrary storage backend.
pub struct NbtPredicate(NbtCompound);

impl NbtPredicate {
    #[must_use]
    /// Checks the custom data component stored on an item.
    pub fn matches_item(&self, item: &ItemStack) -> bool {
        let data: Option<&CustomDataImpl> = item.get_data_component();
        self.0.is_empty() || data.is_some_and(|data| data.data == self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_component::DataComponent;
    use crate::data_component_impl::CustomDataImpl;
    use crate::item::Item;
    use crate::item_stack::ItemStack;
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_util::math::bounds::{DoubleBounds, IntBounds};
    use std::borrow::Cow;

    #[test]
    fn modifier_predicate_filters_values() {
        let predicate = ModifierPredicate {
            attribute: Some(vec![&Attributes::ARMOR]),
            id: Some("armor_bonus"),
            amount: DoubleBounds::new(1.0, 3.0),
            operation: Some(Operation::AddValue),
            slot: Some(AttributeModifierSlot::Chest),
        };

        let matching = Modifier {
            r#type: &Attributes::ARMOR,
            id: "armor_bonus",
            amount: 2.0,
            operation: Operation::AddValue,
            slot: AttributeModifierSlot::Chest,
        };
        let wrong_amount = Modifier {
            amount: 4.0,
            ..matching.clone()
        };

        assert!(predicate.test(&matching));
        assert!(!predicate.test(&wrong_amount));
    }

    static ENCHANT: &[(&Enchantment, i32); 1] = &[(&Enchantment::SHARPNESS, 4)];

    #[test]
    fn enchantment_predicate_matches_expected_level() {
        let predicate = EnchantmentPredicate {
            enchantments: Some(vec![&Enchantment::SHARPNESS]),
            level: (3..=5).into(),
        };

        let enchantments = EnchantmentsImpl {
            enchantment: Cow::Owned(vec![(&Enchantment::SHARPNESS, 4)]),
        };

        assert!(predicate.contained_in(&enchantments));
        assert!(!predicate.matches_enchantment(&enchantments, &Enchantment::MENDING));
    }

    #[test]
    fn firework_predicate_checks_shape_and_flags() {
        let predicate = FireworkPredicate {
            shape: Some(FireworkExplosionShape::Star),
            twinkle: Some(true),
            trail: Some(false),
        };
        let firework = FireworkExplosionImpl::new(
            FireworkExplosionShape::Star,
            vec![0xff00ff],
            vec![],
            false,
            true,
        );

        assert!(predicate.test(&firework));
    }

    #[test]
    fn nbt_predicate_matches_item_custom_data() {
        let mut data = NbtCompound::new();
        data.put_string("owner", "pumpkin".to_owned());

        let predicate = NbtPredicate(data.clone());
        let item = ItemStack::new_with_component(
            1,
            &Item::STONE,
            vec![(
                DataComponent::CustomData,
                Some(Box::new(CustomDataImpl::new(data))),
            )],
        );

        assert!(predicate.matches_item(&item));
    }
}
