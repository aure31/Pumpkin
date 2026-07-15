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

#[cfg(test)]
mod tests {
    use crate::predicate::CollectionContentsPredicate;

    fn collection_content_predicate() {
        let zero = CollectionContentsPredicate::<i32>::Zero;
        assert!(zero.test(vec![&0].into_iter()));
        let single = CollectionContentsPredicate::Single(Box::new(|val: i32| val > 0));
        assert!(single.test(vec![&0, &-1, &1].into_iter()));
        assert!(!single.test(vec![&0].into_iter()));
        let multiple = CollectionContentsPredicate::Multiple(vec![
            Box::new(|val: i32| val < 0),
            Box::new(|val: i32| val > 0),
        ]);
        assert!(multiple.test(vec![&-1, &1].into_iter()));
        assert!(!multiple.test(vec![&-2, &0, &1].into_iter()));
        assert!(!multiple.test(vec![&-3].into_iter()));
    }
}
