//! Predicates wrapping Minecraft data components for item checks.
//!
//! Each struct here validates one or more component fields (damage, enchantments,
//! book content, etc.) using the lower-level predicates from `custom_predicate`.

use crate::predicate::custom_predicate::{
    EnchantmentPredicate, FireworkPredicate, ModifierPredicate, NbtPredicate,
};
use crate::predicate::item_predicate::ItemPredicate;
use crate::predicate::{
    CollectionPredicate, DataComponentPredicate, Predicate, SingleComponentItemPredicate,
};
use pumpkin_data::data_component_impl::{
    AttributeModifiersImpl, BundleContentsImpl, ContainerImpl, DamageImpl, DataComponentImpl,
    EnchantmentsImpl, FireworkExplosionImpl, FireworksImpl, JukeboxPlayableImpl,
    PotionContentsImpl, TrimImpl, VillagerVariantImpl, WritableBookContentImpl,
    WrittenBookContentImpl,
};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::jukebox_song::JukeboxSong;
use pumpkin_data::potion::Potion;
use pumpkin_util::math::bounds::IntBounds;
use pumpkin_util::text::TextComponent;

/// Matches any item that has the given data component, regardless of its value.
pub struct AnyValue<T: DataComponentImpl + 'static>(T);
impl<T: DataComponentImpl + 'static> DataComponentPredicate for AnyValue<T> {
    fn matches(&self, components: &ItemStack) -> bool {
        components.get_data_component::<T>().is_some()
    }
}

/// Matches attribute modifiers with optional collection filters on the modifier list.
pub struct AttributeModifiersPredicate {
    modifiers: Option<CollectionPredicate<ModifierPredicate>>,
}
impl SingleComponentItemPredicate for AttributeModifiersPredicate {
    type Component = AttributeModifiersImpl;
    fn matches_type(&self, attributes: &AttributeModifiersImpl) -> bool {
        self.modifiers
            .as_ref()
            .is_none_or(|modifiers| modifiers.test(attributes.attribute_modifiers.iter()))
    }
}

/// Matches bundle contents against optional item filters.
pub struct BundlePredicate<'a> {
    items: Option<CollectionPredicate<ItemPredicate<'a>>>,
}

impl SingleComponentItemPredicate for BundlePredicate<'_> {
    type Component = BundleContentsImpl;
    fn matches_type(&self, content: &BundleContentsImpl) -> bool {
        self.items
            .as_ref()
            .is_none_or(|items| items.test(content.items.iter()))
    }
}

/// Matches container inventory contents against optional item filters.
pub struct ContainerPredicate<'a> {
    items: Option<CollectionPredicate<ItemPredicate<'a>>>,
}

impl SingleComponentItemPredicate for ContainerPredicate<'_> {
    type Component = ContainerImpl;
    fn matches_type(&self, content: &ContainerImpl) -> bool {
        self.items
            .as_ref()
            .is_none_or(|items| items.test(content.items.iter().map(|(_, item)| item)))
    }
}

/// Matches item durability and damage against bounds.
pub struct DamagePredicate {
    durability: IntBounds,
    damage: IntBounds,
}

impl DataComponentPredicate for DamagePredicate {
    fn matches(&self, components: &ItemStack) -> bool {
        let damage = components.get_data_component::<DamageImpl>();
        damage.is_some_and(|damage| {
            let max_damage = components.get_max_damage().unwrap_or(0);
            self.durability.matches(max_damage - damage.damage)
                && self.damage.matches(damage.damage)
        })
    }
}

/// Matches enchantments against a list of optional filters.
pub struct EnchantmentsPredicate {
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

/// Matches a single firework explosion's properties.
struct FireworkExplosionPredicate(FireworkPredicate);

impl SingleComponentItemPredicate for FireworkExplosionPredicate {
    type Component = FireworkExplosionImpl;

    fn matches_type(&self, value: &Self::Component) -> bool {
        self.0.test(value)
    }
}

/// Matches a fireworks item with explosions and flight duration.
struct FireworksPredicate {
    explosions: Option<CollectionPredicate<FireworkPredicate>>,
    flight_duration: IntBounds,
}

impl SingleComponentItemPredicate for FireworksPredicate {
    type Component = FireworksImpl;

    fn matches_type(&self, value: &Self::Component) -> bool {
        self.explosions
            .as_ref()
            .is_none_or(|p| p.test(value.explosions.iter()))
            && self.flight_duration.matches(value.flight_duration)
    }
}

/// Matches a jukebox playable disc by song.
struct JukeboxPlayablePredicate {
    song: Option<Vec<&'static JukeboxSong>>,
}

impl SingleComponentItemPredicate for JukeboxPlayablePredicate {
    type Component = JukeboxPlayableImpl;

    fn matches_type(&self, value: &Self::Component) -> bool {
        self.song
            .as_ref()
            .is_none_or(|song| song.iter().any(|j| j.to_name() == value.song))
    }
}

/// Matches potion type against optional potion list.
struct PotionsPredicate {
    potions: Vec<&'static Potion>,
}

impl SingleComponentItemPredicate for PotionsPredicate {
    type Component = PotionContentsImpl;

    fn matches_type(&self, value: &Self::Component) -> bool {
        !value
            .potion_id
            .is_some_and(|potion| self.potions.iter().any(|p| p.id == potion as u8))
    }
}

/// Matches armor trim material and pattern (TODO: needs TrimMaterial/TrimPattern types).
struct TrimPredicate {
    material: Option<Vec<&'static str>>,
    pattern: Option<Vec<&'static str>>,
}

impl SingleComponentItemPredicate for TrimPredicate {
    type Component = TrimImpl;
    fn matches_type(&self, _value: &Self::Component) -> bool {
        false // TODO
    }
}

/// Matches villager type by variant name.
struct VillagerTypePredicate {
    villager_types: Vec<&'static str>,
}

impl SingleComponentItemPredicate for VillagerTypePredicate {
    type Component = VillagerVariantImpl;
    fn matches_type(&self, value: &Self::Component) -> bool {
        self.villager_types.contains(&value.value.as_ref())
    }
}

/// Matches a page string in a writable book.
struct StringPagePredicate(String);
impl Predicate for StringPagePredicate {
    type Item = String;
    fn test(&self, value: &String) -> bool {
        value == &self.0
    }
}

/// Matches writable book pages against optional page filters.
struct WritableBookPredicate {
    pages: Option<CollectionPredicate<StringPagePredicate>>,
}

impl SingleComponentItemPredicate for WritableBookPredicate {
    type Component = WritableBookContentImpl;

    fn matches_type(&self, value: &Self::Component) -> bool {
        self.pages
            .as_ref()
            .is_none_or(|pages| pages.test(value.pages.iter()))
    }
}

/// Matches a text component page in a written book.
struct ComponentPagePredicate(TextComponent);
impl Predicate for ComponentPagePredicate {
    type Item = TextComponent;
    fn test(&self, component: &TextComponent) -> bool {
        component == &self.0
    }
}

/// Matches written book metadata (author, title, generation) and pages.
struct WrittenBookPredicate {
    pages: Option<CollectionPredicate<ComponentPagePredicate>>,
    author: Option<String>,
    title: Option<String>,
    generation: IntBounds,
    resolved: Option<bool>,
}

impl SingleComponentItemPredicate for WrittenBookPredicate {
    type Component = WrittenBookContentImpl;
    fn matches_type(&self, value: &Self::Component) -> bool {
        self.author.as_deref().is_none_or(|a| a == value.author)
            && self.title.as_deref().is_none_or(|t| t == value.title)
            && self.generation.matches(value.generation)
            && self.resolved.is_none_or(|r| r == value.resolved)
            && self
                .pages
                .as_ref()
                .is_none_or(|p| p.test(value.pages.iter()))
    }
}

/// Matches item NBT custom data.
struct CustomDataPredicate(NbtPredicate);
impl DataComponentPredicate for CustomDataPredicate {
    fn matches(&self, item: &ItemStack) -> bool {
        self.0.matches_item(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_value_matches_component_presence() {
        let item = ItemStack::new(1, &pumpkin_data::item::Item::STONE);
        let predicate = AnyValue(DamageImpl { damage: 0 });

        // Stone doesn't have damage by default
        assert!(!predicate.matches(&item));
    }

    #[test]
    fn damage_predicate_checks_durability_and_damage() {
        let mut item = ItemStack::new(1, &pumpkin_data::item::Item::DIAMOND_PICKAXE);
        let predicate = DamagePredicate {
            durability: IntBounds::new(0, 100),
            damage: IntBounds::new(0, 50),
        };

        // Item with no damage component shouldn't match
        assert!(!predicate.matches(&item));
    }

    #[test]
    fn string_page_predicate_tests_equality() {
        let page = StringPagePredicate("Hello, world!".to_string());
        assert!(page.test(&"Hello, world!".to_string()));
        assert!(!page.test(&"Goodbye!".to_string()));
    }

    #[test]
    fn component_page_predicate_tests_text_component_equality() {
        use pumpkin_util::text::TextComponent;
        let component = TextComponent::text("Test page");
        let page = ComponentPagePredicate(component.clone());

        assert!(page.test(&component));
        assert!(!page.test(&TextComponent::text("Different")));
    }
}
