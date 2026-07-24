use std::ops::{Bound, RangeBounds};

/// Represents a single range bound of some type `T`, whose bounds may be optional.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds<T: Clone + PartialOrd> {
    min: Option<T>,
    max: Option<T>,
}

impl<T: Clone + PartialOrd> Bounds<T> {
    /// Constructs a new [`Bounds`].
    pub const fn new<U: Clone + PartialOrd>(min: Option<U>, max: Option<U>) -> Bounds<U> {
        Bounds { min, max }
    }

    /// Returns whether the minimum and maximum numbers in this [`Bounds`] are swapped.
    pub fn are_swapped(&self) -> bool {
        if let Some(min) = self.min.clone()
            && let Some(max) = self.max.clone()
        {
            min > max
        } else {
            false
        }
    }
}

impl<T: Clone + PartialOrd> RangeBounds<T> for Bounds<T> {
    fn start_bound(&self) -> Bound<&T> {
        self.min.as_ref().map_or(Bound::Unbounded, Bound::Included)
    }

    fn end_bound(&self) -> Bound<&T> {
        self.max.as_ref().map_or(Bound::Unbounded, Bound::Included)
    }
}

/// Represents a range of integers.
/// This range stores both the bounds of the range and the squares of the bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct IntBounds {
    bounds: Bounds<i32>,
    squared_bounds: Bounds<i64>,
}

macro_rules! impl_square_cached_bounds {
    ($ty:ty, $normal_ty:ty, $squared_ty:ty) => {
        /// Returns a pair of bounds with the provided minimum and maximum values.
        pub fn new(min: $normal_ty, max: $normal_ty) -> Self {
            Self::from_bounds(Bounds::<$normal_ty>::new(Some(min), Some(max)))
        }

        /// Returns a pair of bounds with the provided minimum value.
        pub fn new_at_least(min: $normal_ty) -> Self {
            Self::from_bounds(Bounds::<$normal_ty>::new(Some(min), None))
        }

        /// Returns a pair of bounds with the provided maximum value.
        pub fn new_at_most(max: $normal_ty) -> Self {
            Self::from_bounds(Bounds::<$normal_ty>::new(None, Some(max)))
        }

        /// Returns whether a number satisfies these bounds.
        #[must_use]
        pub fn matches(&self, number: $normal_ty) -> bool {
            self.bounds.min.is_none_or(|min| min <= number)
                && self.bounds.max.is_none_or(|max| max >= number)
        }

        /// Returns whether a number satisfies these bounds' squared form.
        #[must_use]
        pub fn matches_square(&self, number: $squared_ty) -> bool {
            self.squared_bounds.min.is_none_or(|min| min <= number)
                && self.squared_bounds.max.is_none_or(|max| max >= number)
        }

        #[doc = concat!("Returns the maximum bound of this [`", stringify!($ty), "`].")]
        #[must_use]
        pub const fn min(&self) -> Option<$normal_ty> {
            self.bounds.min
        }

        /// Returns the maximum bound of this [`IntBounds`].
        #[must_use]
        pub const fn max(&self) -> Option<$normal_ty> {
            self.bounds.max
        }
    };
}

impl IntBounds {
    pub const ANY: Self = Self {
        bounds: Bounds::<i32>::new(None, None),
        squared_bounds: Bounds::<i64>::new(None, None),
    };
    /// Creates a new [`IntBounds`] with the provided [`Bounds`].
    pub fn from_bounds(bounds: Bounds<i32>) -> Self {
        Self {
            bounds,
            squared_bounds: Bounds {
                min: bounds.min.map(|m| (m as i64) * (m as i64)),
                max: bounds.max.map(|m| (m as i64) * (m as i64)),
            },
        }
    }

    impl_square_cached_bounds!(IntBounds, i32, i64);
}

#[inline]
const fn bound_int_to_option(bound: Bound<&i32>, start: bool) -> Option<i32> {
    match bound {
        Bound::Included(&n) => Some(n),
        Bound::Excluded(&n) => {
            if n == i32::MAX && start || n == i32::MIN && !start {
                None
            } else {
                Some(n + (start as i32) * 2 - 1)
            }
        }
        Bound::Unbounded => None,
    }
}

impl<T: RangeBounds<i32>> From<T> for IntBounds {
    fn from(value: T) -> Self {
        Self::from_bounds(Bounds::<i32>::new(
            bound_int_to_option(value.start_bound(), true),
            bound_int_to_option(value.end_bound(), false),
        ))
    }
}

/// Represents a range of `f64`s.
/// This range stores both the bounds of the range and the squares of the bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use]
pub struct DoubleBounds {
    bounds: Bounds<f64>,
    squared_bounds: Bounds<f64>,
}

impl DoubleBounds {
    /// Creates a new [`DoubleBounds`] with the provided [`Bounds`].
    pub fn from_bounds(bounds: Bounds<f64>) -> Self {
        Self {
            bounds,
            squared_bounds: Bounds {
                min: bounds.min.map(|m| m * m),
                max: bounds.max.map(|m| m * m),
            },
        }
    }

    impl_square_cached_bounds!(DoubleBounds, f64, f64);
}

#[inline]
const fn bound_double_to_option(bound: Bound<&f64>) -> Option<f64> {
    match bound {
        Bound::Included(n) | Bound::Excluded(n) => Some(*n),
        Bound::Unbounded => None,
    }
}

impl<T: RangeBounds<f64>> From<T> for DoubleBounds {
    fn from(value: T) -> Self {
        Self::from_bounds(Bounds::<f64>::new(
            bound_double_to_option(value.start_bound()),
            bound_double_to_option(value.end_bound()),
        ))
    }
}

/// Represents a range of degrees, stored as `f32`s.
/// This range only stores the minimum and maximum degree values.
#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use]
pub struct FloatDegreeBounds {
    bounds: Bounds<f32>,
}

impl FloatDegreeBounds {
    /// Creates a new [`FloatDegreeBounds`] with the provided [`Bounds`].
    pub const fn from_bounds(bounds: Bounds<f32>) -> Self {
        Self { bounds }
    }

    /// Returns the minimum degree amount of this [`FloatDegreeBounds`].
    #[must_use]
    pub const fn min(&self) -> Option<f32> {
        self.bounds.min
    }

    /// Returns the maximum degree amount of this [`FloatDegreeBounds`].
    #[must_use]
    pub const fn max(&self) -> Option<f32> {
        self.bounds.max
    }
}

#[inline]
const fn bound_float_to_option(bound: Bound<&f32>) -> Option<f32> {
    match bound {
        Bound::Included(n) | Bound::Excluded(n) => Some(*n),
        Bound::Unbounded => None,
    }
}

impl<T: RangeBounds<f32>> From<T> for FloatDegreeBounds {
    fn from(value: T) -> Self {
        Self::from_bounds(Bounds::<f32>::new(
            bound_float_to_option(value.start_bound()),
            bound_float_to_option(value.end_bound()),
        ))
    }
}
