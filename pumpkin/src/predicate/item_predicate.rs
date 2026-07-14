use crate::entity::NBTStorage;
use pumpkin_data::AttributeModifierSlot;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::data_component_impl::{
    AttributeModifiersImpl, CustomDataImpl, DataComponentImpl, Modifier, Operation,
};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::NbtCompound;
use pumpkin_util::math::bounds::{DoubleBounds, IntBounds};
use std::collections::HashMap;

pub trait DataComponentPredicate {
    fn matches(&self, components: &ItemStack) -> bool;
}

pub trait DataComponentItemPredicate<T: DataComponentImpl + 'static> {
    fn matches_type(&self, components: &T) -> bool;
}

impl<T: DataComponentImpl + 'static, G: DataComponentItemPredicate<T>> DataComponentPredicate
    for G
{
    fn matches(&self, components: &ItemStack) -> bool {
        let value: Option<&T> = components.get_data_component();
        value.is_some() && self.matches_type(value.unwrap())
    }
}

struct AnyValue<T: DataComponentImpl + 'static>(T);
impl<T: DataComponentImpl + 'static> DataComponentPredicate for AnyValue<T> {
    fn matches(&self, components: &ItemStack) -> bool {
        components.get_data_component::<T>().is_some()
    }
}

struct ModifierPredicate {
    attribute: Option<Vec<&'static Attributes>>,
    id: Option<&'static str>,
    amount: DoubleBounds,
    operation: Option<Operation>,
    slot: Option<AttributeModifierSlot>,
}

impl DataComponentItemPredicate<Modifier> for ModifierPredicate {
    fn matches_type(&self, value: &Modifier) -> bool {
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

type Predicate<T> = Box<dyn Fn(&T) -> bool + Send + Sync>;

struct CollectionCountsEntry<T> {
    predicate: Predicate<T>,
    counts: IntBounds,
}

impl<T: 'static> CollectionCountsEntry<T> {
    pub fn test<'a>(&self, values: impl Iterator<Item = &'a T> + Sized) -> bool {
        values.into_iter().any(|value| (self.predicate)(value))
    }
}

enum CollectionContentsPredicate<T> {
    Multiple(Vec<Predicate<T>>),
    Single(Predicate<T>),
    Zero,
}

impl<T: 'static> CollectionContentsPredicate<T> {
    pub fn test<'a>(&self, values: impl Iterator<Item = &'a T> + Clone) -> bool {
        match self {
            Self::Multiple(predicates) => predicates
                .iter()
                .all(|predicate| values.clone().any(predicate)),
            Self::Single(predicate) => values.into_iter().any(predicate),
            Self::Zero => true,
        }
    }
}

enum CollectionCountsPredicate<T> {
    Multiple(Vec<CollectionCountsEntry<T>>),
    Single(CollectionCountsEntry<T>),
    Zero,
}

impl<T: 'static> CollectionCountsPredicate<T> {
    pub fn test<'a>(&self, values: impl Iterator<Item = &'a T> + Clone) -> bool {
        match self {
            Self::Zero => true,
            Self::Single(entry) => entry.test(values),
            Self::Multiple(entries) => entries.iter().all(|entry| entry.test(values.clone())),
        }
    }
}

struct CollectionPredicate<T> {
    contains: Option<CollectionContentsPredicate<T>>,
    counts: Option<CollectionCountsPredicate<T>>,
    size: Option<IntBounds>,
}

impl<T: 'static> CollectionPredicate<T> {
    pub fn test<'a>(&self, values: impl Iterator<Item = &'a T> + Clone) -> bool {
        self.contains
            .as_ref()
            .is_none_or(|contains| contains.test(values.clone()))
            && self
                .counts
                .as_ref()
                .is_none_or(|counts| counts.test(values.clone()))
            && self
                .size
                .as_ref()
                .is_none_or(|size| size.matches(values.count() as i32))
    }
}

struct AttributeModifiersPredicate {
    modifiers: Option<CollectionPredicate<Modifier>>,
}
impl DataComponentPredicate for AttributeModifiersPredicate {
    fn matches(&self, components: &ItemStack) -> bool {
        let attributes = components
            .get_data_component::<AttributeModifiersImpl>()
            .unwrap();
        self.modifiers
            .as_ref()
            .is_none_or(|modifiers| modifiers.test(attributes.attribute_modifiers.iter()))
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
