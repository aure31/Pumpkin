use crate::predicate::{DataComponentPredicate, Predicate};
use pumpkin_data::data_component_impl::DataComponentImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::bounds::IntBounds;
use std::collections::HashMap;

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
