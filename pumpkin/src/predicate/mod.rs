#![allow(unused)]
//! Small predicate helpers used to describe item and collection checks.
//!
//! The module keeps the building blocks generic so the same logic can be
//! reused across different predicate types without a lot of boilerplate.
use pumpkin_data::data_component_impl::DataComponentImpl;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::bounds::IntBounds;
use std::marker::PhantomData;
use wasmtime::component::__internal::wasmtime_environ::object::ReadCacheOps;

pub mod custom_predicate;
pub mod data_components;
pub mod item_predicate;

/// Checks whether an [`ItemStack`] satisfies a component-based predicate.
/// > Would be latter replace [`ItemStack`] to a more generic type
/// > like the `ComponentGetter` used in the Minecraft source code
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

/// Generic predicate interface used by the collection helpers below.
pub trait Predicate {
    type Item: 'static + ?Sized;
    #[must_use]
    fn test(&self, item: &Self::Item) -> bool;
}

/// Wraps a plain closure or function so it can be used where a [`Predicate`] is expected.
pub struct FnPredicate<F: Fn(&T) -> bool, T: ?Sized> {
    f: F,
    _marker: PhantomData<T>,
}

impl<F: Fn(&T) -> bool, T: 'static + ?Sized> Predicate for FnPredicate<F, T> {
    type Item = T;
    fn test(&self, item: &Self::Item) -> bool {
        (self.f)(item)
    }
}

/// Builds a [`FnPredicate`] from a closure.
pub const fn function<F: Fn(&T) -> bool, T>(f: F) -> FnPredicate<F, T> {
    FnPredicate {
        f,
        _marker: PhantomData,
    }
}

struct CollectionCountsEntry<G: Predicate> {
    predicate: G,
    counts: IntBounds,
}

impl<G: Predicate> CollectionCountsEntry<G> {
    /// Tests how many values match the inner predicate.
    pub fn test<'a>(&self, values: impl Iterator<Item = &'a G::Item> + Sized) -> bool {
        self.counts.matches(
            values
                .into_iter()
                .filter(|&value| self.predicate.test(value))
                .count() as i32,
        )
    }
}

enum CollectionContentsPredicate<G: Predicate> {
    Multiple(Vec<G>),
    Single(G),
    Zero,
}

impl<P: Predicate> CollectionContentsPredicate<P> {
    /// Packs an iterator of predicates into the smallest useful variant.
    pub fn of<T: IntoIterator<Item = P>>(predicates: T) -> Self {
        let mut iter = predicates.into_iter();
        match (iter.next(), iter.next()) {
            (None, _) => Self::Zero,
            (Some(p), None) => Self::Single(p),
            (Some(p0), Some(p1)) => {
                let mut multiple = vec![p0, p1];
                multiple.extend(iter);
                Self::Multiple(multiple)
            }
        }
    }
}

impl<G: Predicate> CollectionContentsPredicate<G> {
    /// Checks that each predicate finds at least one matching value.
    pub fn test<'a>(
        &self,
        values: impl IntoIterator<Item = &'a G::Item, IntoIter: Iterator<Item = &'a G::Item> + Clone>,
    ) -> bool {
        let mut iter = values.into_iter();
        match self {
            Self::Multiple(predicates) => predicates
                .iter()
                .all(|predicate| iter.clone().any(|val| predicate.test(val))),
            Self::Single(predicate) => iter.any(|val| predicate.test(val)),
            Self::Zero => true,
        }
    }
}

macro_rules! collection_contents_predicate {
    () => {
        CollectionContentsPredicate::Zero
    };
    ($elem:expr) => {
        CollectionContentsPredicate::Single($elem)
    };
    ($($elem:expr),* $(,)?) => {
        CollectionContentsPredicate::Multiple(vec![$($elem),*])
    };
    ($t:ty;$($elem:expr),* $(,)?) => {
        CollectionContentsPredicate::<$t>::Multiple(vec![$($elem),*])
    };
}

enum CollectionCountsPredicate<P: Predicate> {
    Multiple(Vec<CollectionCountsEntry<P>>),
    Single(CollectionCountsEntry<P>),
    Zero,
}

impl<P: Predicate> CollectionCountsPredicate<P> {
    /// Packs count constraints into the smallest useful variant.
    pub fn of<T: IntoIterator<Item = CollectionCountsEntry<P>>>(predicates: T) -> Self {
        let mut iter = predicates.into_iter();
        match (iter.next(), iter.next()) {
            (None, _) => Self::Zero,
            (Some(p), None) => Self::Single(p),
            (Some(p0), Some(p1)) => {
                let mut multiple = vec![p0, p1];
                multiple.extend(iter);
                Self::Multiple(multiple)
            }
        }
    }
}

macro_rules! collection_counts_predicate {
    () => {
        CollectionCountsPredicate::Zero
    };
    ($elem:expr=>$counts:expr) => {
        CollectionCountsPredicate::Single(CollectionCountsEntry {predicate:$elem,counts:$counts.into()})
    };
    ($($elem:expr=>$counts:expr),* $(,)?) => {
        CollectionCountsPredicate::Multiple(vec![$(CollectionCountsEntry {predicate:$elem,counts:$counts.into()}),*])
    };
    ($t:ty;$($elem:expr=>$counts:expr),* $(,)?) => {
        CollectionCountsPredicate::<$t>::Multiple(vec![$(CollectionCountsEntry {predicate:$elem,counts:$counts.into()}),*])
    };
}

impl<G: Predicate> CollectionCountsPredicate<G> {
    /// Checks that each count constraint is satisfied by the same values.
    pub fn test<'a>(&self, values: impl IntoIterator<Item = &'a G::Item, IntoIter: Clone>) -> bool {
        let iterator = &values.into_iter();
        match self {
            Self::Zero => true,
            Self::Single(entry) => entry.test(iterator.clone()),
            Self::Multiple(entries) => entries.iter().all(|entry| entry.test(iterator.clone())),
        }
    }
}

struct CollectionPredicate<G: Predicate> {
    contains: Option<CollectionContentsPredicate<G>>,
    counts: Option<CollectionCountsPredicate<G>>,
    size: Option<IntBounds>,
}

impl<G: Predicate> CollectionPredicate<G> {
    /// Creates a collection predicate from optional content, count, and size checks.
    pub const fn new(
        contains: Option<CollectionContentsPredicate<G>>,
        counts: Option<CollectionCountsPredicate<G>>,
        size: Option<IntBounds>,
    ) -> Self {
        Self {
            contains,
            counts,
            size,
        }
    }
}

impl<G: Predicate> CollectionPredicate<G> {
    /// Runs every enabled check against the provided values.
    pub fn test<'a>(
        &self,
        values: impl IntoIterator<Item = &'a G::Item, IntoIter: Clone> + Clone,
    ) -> bool {
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
                .is_none_or(|size| size.matches(values.into_iter().count() as i32))
    }
}

#[cfg(test)]
mod tests {
    use crate::predicate::{
        CollectionContentsPredicate, CollectionCountsEntry, CollectionCountsPredicate,
        CollectionPredicate, FnPredicate, function,
    };
    use pumpkin_util::math::bounds::IntBounds;
    type Fni32 = FnPredicate<fn(&i32) -> bool, i32>;

    #[test]
    fn collection_content_predicate() {
        let zero: CollectionContentsPredicate<Fni32> = collection_contents_predicate!();
        assert!(zero.test([&0]));
        let single = collection_contents_predicate!(function(|val| val > &0));
        assert!(single.test([&0, &-1, &1]));
        assert!(!single.test([&0]));
        let multiple = collection_contents_predicate!(
            Fni32;
            function(|val| val < &0),
            function(|val| val > &0)
        );
        assert!(multiple.test([&-1, &1]));
        assert!(multiple.test([&-2, &0, &1]));
        assert!(!multiple.test([&-3, &-1]));
    }

    #[test]
    fn collection_counts_predicate() {
        let zero: CollectionCountsPredicate<Fni32> = collection_counts_predicate!();
        assert!(zero.test([&0]));
        let single = collection_counts_predicate!(function(|val: &i32| val > &0)=>1..=2);
        assert!(!single.test([&0, &1, &2, &3]));
        assert!(single.test([&0, &1, &2]));
        assert!(single.test([&0, &1]));
        assert!(!single.test([&0]));
        let multiple = collection_counts_predicate!(
            Fni32;
            function(|val: &i32| val > &0)=>1..=2,
            function(|val: &i32| val < &0)=>2..=3,
        );
        assert!(!multiple.test([&-1, &0, &1, &2, &3]));
        assert!(multiple.test([&-2, &-1, &0, &1, &2]));
        assert!(!multiple.test([&0, &1]));
        assert!(!multiple.test([&0]));
    }

    #[test]
    fn collection_predicate() {
        let empty: CollectionPredicate<Fni32> = CollectionPredicate::new(None, None, None);
        assert!(empty.test([&0]));
        let contains = CollectionPredicate::new(
            Some(collection_contents_predicate!(function(
                |val: &i32| val > &0
            ))),
            None,
            None,
        );
        assert!(contains.test([&0, &-1, &1]));
        assert!(!contains.test([&0]));
        let counts = CollectionPredicate::new(
            None,
            Some(collection_counts_predicate!(function(|val: &i32| val > &0)=>1..=2)),
            None,
        );
        assert!(counts.test([&0, &-1, &1]));
        assert!(!counts.test([&0]));
        let size: CollectionPredicate<Fni32> =
            CollectionPredicate::new(None, None, Some((2..).into()));
        assert!(size.test([&0, &-1, &1]));
        assert!(!size.test([&0]));
        let zero_combine = CollectionPredicate::<Fni32>::new(
            Some(collection_contents_predicate!()),
            Some(collection_counts_predicate!()),
            Some((2..).into()),
        );
        assert!(zero_combine.test([&0, &-1, &1]));
        assert!(!zero_combine.test([&0]));
        let single_combine = CollectionPredicate::<Fni32>::new(
            Some(collection_contents_predicate!(function(
                |val: &i32| val > &0
            ))),
            Some(collection_counts_predicate!(function(|val:&i32| val < &0)=>1..=2)),
            Some((3..).into()),
        );
        assert!(single_combine.test([&0, &-1, &1]));
        assert!(!single_combine.test([&-1, &-2, &-3, &4]));
        assert!(!single_combine.test([&-1, &1]));
        assert!(!single_combine.test([&0]));
        let multiple_combine = CollectionPredicate::<Fni32>::new(
            Some(collection_contents_predicate!(
                function(|&val: &i32| val < 0),
                function(|&val: &i32| val == 0)
            )),
            Some(collection_counts_predicate!(Fni32;
                function(|&val:&i32| val <= 2 && val > 0)=>1..=2,
                function(|&val:&i32| val == 4)=>1..2
            )),
            Some((..=5).into()),
        );
        assert!(multiple_combine.test([&0, &-1, &1, &4]));
        assert!(!multiple_combine.test([&0, &-1, &1, &4, &4]));
        assert!(!multiple_combine.test([&0, &1]));
        assert!(multiple_combine.test([&0, &-1, &-2, &2, &4]));
        assert!(!multiple_combine.test([&0, &-1, &1, &1, &4, &0]));
    }
}
