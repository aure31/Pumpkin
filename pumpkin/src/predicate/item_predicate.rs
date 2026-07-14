use crate::entity::NBTStorage;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::data_component_impl::{
    AttributeModifiersImpl, BundleContentsImpl, ContainerImpl, CustomDataImpl, DamageImpl,
    DataComponentImpl, EnchantmentsImpl, FireworkExplosionImpl, FireworkExplosionShape,
    FireworksImpl, JukeboxPlayableImpl, Modifier, Operation, PotionContentsImpl,
};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::jukebox_song::JukeboxSong;
use pumpkin_data::potion::Potion;
use pumpkin_data::{AttributeModifierSlot, Enchantment};
use pumpkin_nbt::NbtCompound;
use pumpkin_util::math::bounds::{Bounds, DoubleBounds, IntBounds};
use std::collections::HashMap;

pub trait DataComponentPredicate {
    fn matches(&self, components: &ItemStack) -> bool;
}

trait SingleComponentItemPredicate {
    type Component: DataComponentImpl + 'static;
    fn matches_type(&self, value: &Self::Component) -> bool;
}

impl<G: SingleComponentItemPredicate> DataComponentPredicate for G {
    fn matches(&self, components: &ItemStack) -> bool {
        let value: Option<&G::Component> = components.get_data_component();
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
impl SingleComponentItemPredicate for AttributeModifiersPredicate {
    type Component = AttributeModifiersImpl;
    fn matches_type(&self, attributes: &AttributeModifiersImpl) -> bool {
        self.modifiers
            .as_ref()
            .is_none_or(|modifiers| modifiers.test(attributes.attribute_modifiers.iter()))
    }
}

struct BundlePredicate {
    items: Option<CollectionPredicate<ItemStack>>,
}

impl SingleComponentItemPredicate for BundlePredicate {
    type Component = BundleContentsImpl;
    fn matches_type(&self, content: &BundleContentsImpl) -> bool {
        self.items
            .as_ref()
            .is_none_or(|items| items.test(content.items.iter()))
    }
}

struct ContainerPredicate {
    items: Option<CollectionPredicate<(u8, ItemStack)>>,
}

impl SingleComponentItemPredicate for ContainerPredicate {
    type Component = ContainerImpl;
    fn matches_type(&self, content: &ContainerImpl) -> bool {
        self.items
            .as_ref()
            .is_none_or(|items| items.test(content.items.iter()))
    }
}

struct DamagePredicate {
    durability: IntBounds,
    damage: IntBounds,
}

impl DataComponentPredicate for DamagePredicate {
    fn matches(&self, components: &ItemStack) -> bool {
        let damage = components.get_data_component::<DamageImpl>();
        damage.map_or(false, |damage| {
            let max_damage = components.get_max_damage().unwrap_or(0);
            self.durability.matches(max_damage - damage.damage)
                && self.damage.matches(damage.damage)
        })
    }
}

struct EnchantmentPredicate {
    enchantments: Option<Vec<&'static Enchantment>>,
    level: IntBounds,
}

impl EnchantmentPredicate {
    pub fn contained_in(&self, item_enchantments: &EnchantmentsImpl) -> bool {
        if let Some(enchantments) = self.enchantments {
            for enchantment in enchantments {
                if self.matchesEnchantment(item_enchantments, enchantment) {
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
            !item_enchantments.isEmpty()
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

struct EnchantmentsPredicate {
    enchantments: Vec<EnchantmentPredicate>,
}
impl SingleComponentItemPredicate for EnchantmentsPredicate {
    type Component = EnchantmentsImpl;

    fn matches_type(&self, value: &Self::Component) -> bool {
        for enchantment in &self.enchantments {
            if !enchantment.contained_in(value) {
                return false;
            }
        }
        true
    }
}

struct FireworkPredicate {
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

struct FireworkExplosionPredicate(FireworkPredicate);

impl SingleComponentItemPredicate for FireworkExplosionPredicate {
    type Component = FireworkExplosionImpl;

    fn matches_type(&self, value: &Self::Component) -> bool {
        self.0.test(value)
    }
}

struct FireworksPredicate {
    explosions: Option<CollectionPredicate<FireworkExplosionImpl>>,
    flight_duration: IntBounds,
}

impl SingleComponentItemPredicate for FireworksPredicate {
    type Component = FireworksImpl;

    fn matches_type(&self, value: &Self::Component) -> bool {
        self.explosions
            .as_ref()
            .is_none_or(|p| p.test(&value.explosions))
            && self.flight_duration.matches(value.flight_duration)
    }
}

struct JukeboxPlayablePredicate {
    song: Option<Vec<&'static JukeboxSong>>,
}

impl SingleComponentItemPredicate for JukeboxPlayablePredicate {
    type Component = JukeboxPlayableImpl;

    fn matches_type(&self, value: &Self::Component) -> bool {
        self.song
            .is_none_or(|song| song.iter().any(|j| j.to_name() == value.song))
    }
}

struct PotionsPredicate {
    potions: Vec<&'static Potion>,
}

impl SingleComponentItemPredicate for PotionsPredicate {
    type Component = PotionContentsImpl;

    fn matches_type(&self, value: &Self::Component) -> bool {
        !value
            .potion
            .is_some_and(|potion| self.potions.contains(potion))
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
impl DataComponentPredicate for CustomDataPredicate {
    fn matches(&self, item: &ItemStack) -> bool {
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
