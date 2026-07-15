use pumpkin_data::data_component_impl::DataComponentImpl;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::bounds::IntBounds;

pub mod custom_predicate;
pub mod data_components;
pub mod item_predicate;

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

pub trait Predicate {
    type Target;
    #[must_use]
    fn test(&self, item: &Self::Target) -> bool;
}

impl<T, G: Fn(&T) -> bool> Predicate for G {
    type Target = T;
    fn test(&self, item: &Self::Target) -> bool {
        self(item)
    }
}

struct CollectionCountsEntry<G: Predicate> {
    predicate: G,
    counts: IntBounds,
}

impl<T: 'static, G: Predicate> CollectionCountsEntry<G> {
    pub fn test<'a>(&self, values: impl Iterator<Item = &'a T> + Sized) -> bool {
        self.counts.matches(
            values
                .into_iter()
                .filter(|value| self.predicate.test(value))
                .count() as i32,
        )
    }
}

enum CollectionContentsPredicate<G: Predicate> {
    Multiple(Vec<G>),
    Single(G),
    Zero,
}

impl<G: Predicate> CollectionContentsPredicate<G> {
    pub fn test<'a>(&self, values: impl Iterator<Item = &'a G::Target> + Clone) -> bool {
        match self {
            Self::Multiple(predicates) => predicates
                .iter()
                .all(|predicate| values.clone().any(|val| predicate.test(val))),
            Self::Single(predicate) => values.into_iter().any(|val| predicate.test(val)),
            Self::Zero => true,
        }
    }
}

enum CollectionCountsPredicate<G: Predicate> {
    Multiple(Vec<CollectionCountsEntry<G>>),
    Single(CollectionCountsEntry<G>),
    Zero,
}

impl<T: 'static, G: Predicate> CollectionCountsPredicate<G> {
    pub fn test<'a>(&self, values: impl Iterator<Item = &'a T> + Clone) -> bool {
        match self {
            Self::Zero => true,
            Self::Single(entry) => entry.test(values),
            Self::Multiple(entries) => entries.iter().all(|entry| entry.test(values.clone())),
        }
    }
}

struct CollectionPredicate<G: Predicate> {
    contains: Option<CollectionContentsPredicate<G>>,
    counts: Option<CollectionCountsPredicate<G>>,
    size: Option<IntBounds>,
}

impl<G: Predicate> CollectionPredicate<G> {
    pub fn test<'a>(&self, values: impl Iterator<Item = &'a G::Target> + Clone) -> bool {
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

#[cfg(test)]
mod tests {
    use crate::predicate::{
        CollectionContentsPredicate, CollectionCountsEntry, CollectionCountsPredicate,
    };
    use pumpkin_util::math::bounds::IntBounds;

    #[test]
    fn collection_content_predicate() {
        let zero = CollectionContentsPredicate::<i32, dyn Fn(&i32) -> bool>::Zero;
        assert!(zero.test(vec![&0].into_iter()));
        let single = CollectionContentsPredicate::Single(Box::new(|val| val > &0));
        assert!(single.test(vec![&0, &-1, &1].into_iter()));
        assert!(!single.test(vec![&0].into_iter()));
        let multiple = CollectionContentsPredicate::Multiple(vec![
            Box::new(|val| val < &0),
            Box::new(|val| val > &0),
        ]);
        assert!(multiple.test(vec![&-1, &1].into_iter()));
        assert!(multiple.test(vec![&-2, &0, &1].into_iter()));
        assert!(!multiple.test(vec![&-3, &-1].into_iter()));
    }

    #[test]
    fn collection_count_predicate() {
        let zero = CollectionCountsPredicate::<i32>::Zero;
        assert!(zero.test(vec![&0].into_iter()));
        let single = CollectionCountsPredicate::Single(CollectionCountsEntry {
            predicate: Box::new(|val: &i32| val > &0),
            counts: IntBounds::new(1, 2),
        });
        assert!(!single.test(vec![&0, &1, &2, &3].into_iter()));
        assert!(single.test(vec![&0, &1, &2].into_iter()));
        assert!(single.test(vec![&0, &1].into_iter()));
        assert!(!single.test(vec![&0].into_iter()));
        let multiple = CollectionCountsPredicate::<i32>::Multiple(vec![
            CollectionCountsEntry {
                predicate: Box::new(|val: &i32| val > &0),
                counts: IntBounds::new(1, 2),
            },
            CollectionCountsEntry {
                predicate: Box::new(|val: &i32| val < &0),
                counts: IntBounds::new(2, 3),
            },
        ]);
        assert!(!multiple.test(vec![&-1, &0, &1, &2, &3].into_iter()));
        assert!(multiple.test(vec![&-2, &-1, &0, &1, &2].into_iter()));
        assert!(multiple.test(vec![&0, &1].into_iter()));
        assert!(multiple.test(vec![&0].into_iter()));
    }
}
