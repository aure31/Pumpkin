//! Predicates for matching items by type, count, and data components.
//!
//! Combines item identity checks with component matchers to validate
//! whether an item stack satisfies all predicates.

use crate::predicate::{DataComponentPredicate, Predicate};
use pumpkin_data::data_component_impl::DataComponentImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::bounds::IntBounds;
use std::collections::HashMap;

/// Checks that exact data component values match (no partial matching).
struct DataComponentExactPredicate {
    expected_components: Vec<Box<dyn DataComponentImpl>>,
}

impl DataComponentExactPredicate {
    fn test(&self, actual_components: &ItemStack) -> bool {
        for expected in &self.expected_components {
            let actual = actual_components.get_data_component_dyn(&expected.get_self_enum());
            if actual.is_none()
                || actual.unwrap().get_self_enum() != expected.get_self_enum()
                || !expected.equal(actual.unwrap())
            {
                return false;
            }
        }
        true
    }
}

/// Combines exact component checks with partial predicates for flexible matching.
struct DataComponentMatcher<'a> {
    exact: DataComponentExactPredicate,
    partial: HashMap<&'a dyn DataComponentImpl, &'a dyn DataComponentPredicate>,
}

impl Predicate for DataComponentMatcher<'_> {
    type Item = ItemStack;
    fn test(&self, item: &ItemStack) -> bool {
        if self.exact.test(item) {
            for &predicate in self.partial.values() {
                if !predicate.matches(item) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

/// Matches an item stack by type, count, and data components.
pub struct ItemPredicate<'a> {
    items: Option<Vec<&'static Item>>,
    count: IntBounds,
    components: DataComponentMatcher<'a>,
}

impl Predicate for ItemPredicate<'_> {
    type Item = ItemStack;
    fn test(&self, item: &ItemStack) -> bool {
        self.items
            .as_ref()
            .is_none_or(|items| items.contains(&item.item))
            && self.count.matches(item.item_count as i32)
            && self.components.test(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_predicate_matches_item_type() {
        let stone_stack = ItemStack::new(1, &pumpkin_data::item::Item::STONE);
        let dirt_stack = ItemStack::new(1, &pumpkin_data::item::Item::DIRT);

        let predicate = ItemPredicate {
            items: Some(vec![&pumpkin_data::item::Item::STONE]),
            count: IntBounds::new(1, 64),
            components: DataComponentMatcher {
                exact: DataComponentExactPredicate {
                    expected_components: vec![],
                },
                partial: HashMap::new(),
            },
        };

        assert!(predicate.test(&stone_stack));
        assert!(!predicate.test(&dirt_stack));
    }

    #[test]
    fn item_predicate_matches_count_bounds() {
        let item_stack = ItemStack::new(32, &pumpkin_data::item::Item::STONE);

        let predicate = ItemPredicate {
            items: None,
            count: IntBounds::new(10, 50),
            components: DataComponentMatcher {
                exact: DataComponentExactPredicate {
                    expected_components: vec![],
                },
                partial: HashMap::new(),
            },
        };

        assert!(predicate.test(&item_stack));
    }

    #[test]
    fn item_predicate_rejects_out_of_bounds_count() {
        let item_stack = ItemStack::new(1, &pumpkin_data::item::Item::STONE);

        let predicate = ItemPredicate {
            items: None,
            count: IntBounds::new(10, 50),
            components: DataComponentMatcher {
                exact: DataComponentExactPredicate {
                    expected_components: vec![],
                },
                partial: HashMap::new(),
            },
        };

        assert!(!predicate.test(&item_stack));
    }

    #[test]
    fn data_component_exact_predicate_requires_exact_match() {
        let predicate = DataComponentExactPredicate {
            expected_components: vec![],
        };
        let item_stack = ItemStack::new(1, &pumpkin_data::item::Item::STONE);

        // Empty expected components should match any item
        assert!(predicate.test(&item_stack));
    }
}
