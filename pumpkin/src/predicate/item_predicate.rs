use crate::entity::{Entity, NBTStorage};
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component::DataComponent::CustomData;
use pumpkin_data::data_component_impl::{CustomDataImpl, DataComponentImpl};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::NbtCompound;
use pumpkin_util::math::bounds::IntBounds;
use std::collections::HashMap;

pub trait DataComponentPredicate {
    fn matches(&self, components: ItemStack) -> bool;
}

struct AnyValue<T: DataComponentImpl + 'static>(T);
impl<T: DataComponentImpl + 'static> DataComponentPredicate for AnyValue<T> {
    fn matches(&self, components: ItemStack) -> bool {
        components.get_data_component::<T>().is_some()
    }
}

struct DataComponentExactPredicate {
    expectedComponents: Vec<Box<dyn DataComponentImpl>>,
}

struct NbtPredicate(NbtCompound);

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
struct CustomDataPredicate(NbtPredicate);
impl CustomDataPredicate {
    pub fn matches(&self, item: &ItemStack) -> bool {
        self.0.matches_item(item)
    }
}

struct DataComponentMatcher<'a> {
    exact: DataComponentExactPredicate,
    partial: HashMap<&'a dyn DataComponentImpl, &'a dyn DataComponentPredicate>,
}

impl<'a> DataComponentMatcher<'a> {
    pub fn test(&self, item: ItemStack) -> bool {
        if !self.exact.test(item) {
            return false;
        }
        for predicate in &self.partial.values() {}
    }
}

struct ItemPredicate<'a> {
    items: Vec<&'static Item>,
    counts: IntBounds,
    components: DataComponentMatcher<'a>,
}
