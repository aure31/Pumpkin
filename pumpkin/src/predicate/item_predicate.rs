use crate::entity::NBTStorage;
use crate::entity::attributes::ModifierOperation;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::data_component_impl::{
    CustomDataImpl, DataComponentImpl, EquipmentSlot, EquipmentSlotData,
};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::NbtCompound;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::math::bounds::{DoubleBounds, IntBounds};
use std::collections::HashMap;

pub trait DataComponentPredicate {
    fn matches(&self, components: &ItemStack) -> bool;
}

struct AnyValue<T: DataComponentImpl + 'static>(T);
impl<T: DataComponentImpl + 'static> DataComponentPredicate for AnyValue<T> {
    fn matches(&self, components: &ItemStack) -> bool {
        components.get_data_component::<T>().is_some()
    }
}

struct ModifierEntryPredicate {
    attribute: Option<Vec<Attributes>>,
    id: Option<Identifier>,
    amount: DoubleBounds,
    operation: Option<ModifierOperation>,
    slot: Option<EquipmentSlotData>,
}

struct AttributeModifiersPredicate<T: DataComponentImpl + 'static> {
    modifiers: Option<Vec<Entry, ModifierEntryPredicate>>,
}
impl<T: DataComponentImpl + 'static> DataComponentPredicate for AttributeModifiersPredicate<T> {
    fn matches(&self, components: &ItemStack) -> bool {
        if let Some(predicate) = self.modifiers {}
    }
}

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

impl DataComponentMatcher<'_> {
    pub fn test(&self, item: &ItemStack) -> bool {
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

struct ItemPredicate<'a> {
    items: Option<Vec<&'static Item>>,
    count: IntBounds,
    components: DataComponentMatcher<'a>,
}

impl ItemPredicate<'_> {
    pub fn test(&self, item: &ItemStack) -> bool {
        if let Some(items) = &self.items
            && !items.contains(&item.item)
        {
            false
        } else {
            self.count.matches(item.item_count as i32) && self.components.test(item)
        }
    }
}
