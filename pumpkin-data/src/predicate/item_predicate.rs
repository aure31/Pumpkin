//! Predicates for matching items by type, count, and data components.
//!
//! Combines item identity checks with component matchers to validate
//! whether an item stack satisfies all predicates.
#![allow(clippy::disallowed_types)]
use crate::data_component::DataComponent;
use crate::data_component_impl::DataComponentImpl;
use crate::item::Item;
use crate::item_stack::ItemStack;
use crate::predicate::Predicate;
use crate::predicate::data_components::DataComponentPredicate;
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
            if actual.is_none() || !expected.equal(actual.unwrap()) {
                return false;
            }
        }
        true
    }
}

/// Combines exact component checks with partial predicates for flexible matching.
struct DataComponentMatcher {
    exact: DataComponentExactPredicate,
    partial: HashMap<DataComponent, DataComponentPredicate>,
}

impl Predicate for DataComponentMatcher {
    type Item = ItemStack;
    fn test(&self, item: &ItemStack) -> bool {
        if self.exact.test(item) {
            for predicate in self.partial.values() {
                if !predicate.test(item) {
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
pub struct ItemPredicate {
    items: Option<Vec<&'static Item>>,
    count: IntBounds,
    components: DataComponentMatcher,
}

impl Predicate for ItemPredicate {
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
        let stone_stack = ItemStack::new(1, &Item::STONE);
        let dirt_stack = ItemStack::new(1, &Item::DIRT);

        let predicate = ItemPredicate {
            items: Some(vec![&Item::STONE]),
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
        let item_stack = ItemStack::new(32, &Item::STONE);

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
        let item_stack = ItemStack::new(1, &Item::STONE);

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
        let item_stack = ItemStack::new(1, &Item::STONE);

        // Empty expected components should match any item
        assert!(predicate.test(&item_stack));
    }
}
