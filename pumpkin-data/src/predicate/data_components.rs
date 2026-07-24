use crate::data_component::DataComponent;
use crate::data_component_impl::{AttributeModifiersImpl, BundleContentsImpl, ContainerImpl};
use crate::item_stack::ItemStack;
use crate::jukebox_song::JukeboxSong;
use crate::potion::Potion;
use crate::predicate::custom_predicate::{
    EnchantmentPredicate, FireworkPredicate, ModifierPredicate, NbtPredicate,
};
use crate::predicate::item_predicate::ItemPredicate;
use crate::predicate::{CollectionPredicate, FnStoredPredicate, Predicate};
use pumpkin_util::math::bounds::IntBounds;
use pumpkin_util::text::TextComponent;

/// Checks whether an [`ItemStack`] satisfies a component-based predicate.
/// > Would be latter replace [`ItemStack`] to a more generic type
/// > like the `ComponentGetter` used in the Minecraft source code
pub enum DataComponentPredicate {
    AnyValue(DataComponent),
    AttributeModifiers(Option<CollectionPredicate<ModifierPredicate>>),
    Bundle(Option<CollectionPredicate<ItemPredicate>>),
    Container(Option<CollectionPredicate<ItemPredicate>>),
    Damage {
        durability: IntBounds,
        damage: IntBounds,
    },
    Enchantments(Vec<EnchantmentPredicate>),
    FireworkExplosion(FireworkPredicate),
    Fireworks {
        explosions: Option<CollectionPredicate<FireworkPredicate>>,
        flight_duration: IntBounds,
    },
    JukeboxPlayable(Option<Vec<&'static JukeboxSong>>),
    Potions(Vec<&'static Potion>),
    Trim {
        material: Option<Vec<&'static str>>,
        pattern: Option<Vec<&'static str>>,
    },
    VillagerType(Vec<&'static str>),
    WritableBook(Option<CollectionPredicate<FnStoredPredicate<String>>>),
    WrittenBook {
        pages: Option<CollectionPredicate<FnStoredPredicate<TextComponent>>>,
        author: Option<String>,
        title: Option<String>,
        generation: IntBounds,
        resolved: Option<bool>,
    },
    CustomData(NbtPredicate),
}

impl Predicate for DataComponentPredicate {
    type Item = ItemStack;

    fn test(&self, item: &Self::Item) -> bool {
        match self {
            Self::AnyValue(data_component) => item.get_data_component_dyn(data_component).is_some(),
            Self::AttributeModifiers(modifiers) => {
                let attributes = item.get_data_component::<AttributeModifiersImpl>();
                attributes.is_some_and(|attributes| {
                    modifiers.as_ref().is_none_or(|modifiers| {
                        modifiers.test(attributes.attribute_modifiers.iter())
                    })
                })
            }
            Self::Bundle(items) => {
                let content = item.get_data_component::<BundleContentsImpl>();
                content.is_some_and(|content| {
                    items
                        .as_ref()
                        .is_none_or(|items| items.test(content.items.iter()))
                })
            }
            Self::Container(items) => {
                let content = item.get_data_component::<ContainerImpl>();
                content.is_some_and(|content| {
                    items
                        .as_ref()
                        .is_none_or(|items| items.test(content.items.iter().map(|(_, item)| item)))
                })
            }
            Self::Damage { durability, damage } => {
                let dmg = item.get_data_component::<crate::data_component_impl::DamageImpl>();
                dmg.is_some_and(|d| {
                    let max_damage = item.get_max_damage().unwrap_or(0);
                    durability.matches(max_damage - d.damage) && damage.matches(d.damage)
                })
            }
            Self::Enchantments(enchantments) => {
                let ench =
                    item.get_data_component::<crate::data_component_impl::EnchantmentsImpl>();
                ench.is_some_and(|value| {
                    for enchantment in enchantments {
                        if !enchantment.contained_in(value) {
                            return false;
                        }
                    }
                    true
                })
            }
            Self::FireworkExplosion(pred) => {
                let expl =
                    item.get_data_component::<crate::data_component_impl::FireworkExplosionImpl>();
                expl.is_some_and(|v| pred.test(v))
            }
            Self::Fireworks {
                explosions,
                flight_duration,
            } => {
                let fw = item.get_data_component::<crate::data_component_impl::FireworksImpl>();
                fw.is_some_and(|v| {
                    explosions
                        .as_ref()
                        .is_none_or(|p| p.test(v.explosions.iter()))
                        && flight_duration.matches(v.flight_duration)
                })
            }
            Self::JukeboxPlayable(song) => {
                let jb =
                    item.get_data_component::<crate::data_component_impl::JukeboxPlayableImpl>();
                jb.is_some_and(|v| {
                    song.as_ref()
                        .is_none_or(|song| song.iter().any(|j| j.to_name() == v.song))
                })
            }
            Self::Potions(potions) => {
                let pc =
                    item.get_data_component::<crate::data_component_impl::PotionContentsImpl>();
                pc.is_some_and(|v| {
                    v.potion_id
                        .as_ref()
                        .is_none_or(|potion| potions.iter().any(|p| p.id == *potion as u8))
                })
            }
            Self::Trim {
                material: _,
                pattern: _,
            } => {
                // TODO: provide meaningful trim checks once TrimImpl types are available
                let _ = item.get_data_component::<crate::data_component_impl::TrimImpl>();
                false
            }
            Self::VillagerType(names) => {
                let vv =
                    item.get_data_component::<crate::data_component_impl::VillagerVariantImpl>();
                vv.is_some_and(|v| names.contains(&v.value.as_ref()))
            }
            Self::WritableBook(pages) => {
                let wb = item
                    .get_data_component::<crate::data_component_impl::WritableBookContentImpl>();
                wb.is_some_and(|v| pages.as_ref().is_none_or(|p| p.test(v.pages.iter())))
            }
            Self::WrittenBook {
                pages,
                author,
                title,
                generation,
                resolved,
            } => {
                let wb =
                    item.get_data_component::<crate::data_component_impl::WrittenBookContentImpl>();
                wb.is_some_and(|v| {
                    author.as_deref().is_none_or(|a| a == v.author)
                        && title.as_deref().is_none_or(|t| t == v.title)
                        && generation.matches(v.generation)
                        && resolved.is_none_or(|r| r == v.resolved)
                        && pages.as_ref().is_none_or(|p| p.test(v.pages.iter()))
                })
            }
            Self::CustomData(nbt) => nbt.matches_item(item),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Item;
    use crate::predicate::stored_predicate;

    #[test]
    fn any_value_matches_component_presence() {
        let item = ItemStack::new(1, &Item::STONE);
        let sword = ItemStack::new(2, &Item::DIAMOND_SWORD);
        let predicate = DataComponentPredicate::AnyValue(DataComponent::Damage);

        // Stone doesn't have damage by default
        assert!(!predicate.test(&item));
        assert!(predicate.test(&sword));
    }

    #[test]
    fn damage_predicate_checks_durability_and_damage() {
        let mut item = ItemStack::new(1, &Item::DIAMOND_PICKAXE);
        let predicate = DataComponentPredicate::Damage {
            durability: IntBounds::new(0, 100),
            damage: IntBounds::new(0, 50),
        };

        // Item with no damage component shouldn't match
        assert!(!predicate.test(&item));
    }
}
