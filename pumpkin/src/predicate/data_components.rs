use crate::predicate::custom_predicate::{
    EnchantmentPredicate, FireworkPredicate, ModifierPredicate, NbtPredicate,
};
use crate::predicate::item_predicate::ItemPredicate;
use crate::predicate::{
    CollectionPredicate, DataComponentPredicate, Predicate, SingleComponentItemPredicate,
};
use pumpkin_data::data_component_impl::{
    AttributeModifiersImpl, BundleContentsImpl, ContainerImpl, DamageImpl, DataComponentImpl,
    EnchantmentsImpl, FireworkExplosionImpl, FireworksImpl, JukeboxPlayableImpl, Modifier,
    PotionContentsImpl, TrimImpl, VillagerVariantImpl, WritableBookContentImpl,
    WrittenBookContentImpl,
};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::jukebox_song::JukeboxSong;
use pumpkin_data::potion::Potion;
use pumpkin_util::math::bounds::IntBounds;
use pumpkin_util::text::TextComponent;

pub struct AnyValue<T: DataComponentImpl + 'static>(T);
impl<T: DataComponentImpl + 'static> DataComponentPredicate for AnyValue<T> {
    fn matches(&self, components: &ItemStack) -> bool {
        components.get_data_component::<T>().is_some()
    }
}

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

struct FireworkExplosionPredicate(FireworkPredicate);

impl SingleComponentItemPredicate for FireworkExplosionPredicate {
    type Component = FireworkExplosionImpl;

    fn matches_type(&self, value: &Self::Component) -> bool {
        self.0.test(value)
    }
}

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

struct TrimPredicate {
    material: Option<Vec<&'static str>>, //TODO use TrimMaterial when implemented
    pattern: Option<Vec<&'static str>>,  //TODO use TrimPattern when implemented
}

impl SingleComponentItemPredicate for TrimPredicate {
    type Component = TrimImpl;
    fn matches_type(&self, _value: &Self::Component) -> bool {
        false // TODO
    }
}

struct VillagerTypePredicate {
    villager_types: Vec<&'static str>,
}

impl SingleComponentItemPredicate for VillagerTypePredicate {
    type Component = VillagerVariantImpl;
    fn matches_type(&self, value: &Self::Component) -> bool {
        self.villager_types.contains(&value.value.as_ref())
    }
}

struct StringPagePredicate(String);
impl Predicate for StringPagePredicate {
    type Item = String;
    fn test(&self, value: &String) -> bool {
        value == &self.0
    }
}
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
struct ComponentPagePredicate(TextComponent);
impl Predicate for ComponentPagePredicate {
    type Item = TextComponent;
    fn test(&self, component: &TextComponent) -> bool {
        component == &self.0
    }
}
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

struct CustomDataPredicate(NbtPredicate);
impl DataComponentPredicate for CustomDataPredicate {
    fn matches(&self, item: &ItemStack) -> bool {
        self.0.matches_item(item)
    }
}
