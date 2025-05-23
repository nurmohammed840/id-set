use crate::*;

/// A trait for updating values in a shared bit-set.
pub trait SharedBitSet<T> {
    /// Clears the set
    ///
    /// # Example
    ///
    /// ```rust
    /// use index_set::{SharedBitSet, BitSet};
    /// use std::sync::atomic::AtomicU32;
    ///
    /// let mut bitset: [AtomicU32; 4] = Default::default();
    /// bitset.insert(0);
    /// assert!(!BitSet::is_empty(&bitset[..]));
    ///
    /// bitset.clear();
    /// assert!(BitSet::is_empty(&bitset[..]));
    /// ```
    fn clear(&self);

    /// Inserts the index into the set
    /// Returns `true` if the index was not already set
    ///
    /// # Example
    ///
    /// ```rust
    /// use index_set::{SharedBitSet, BitSet};
    /// use std::sync::atomic::AtomicU32;
    ///
    /// let mut bitset: [AtomicU32; 4] = Default::default();
    /// bitset.insert(0);
    /// assert_eq!(bitset.has(0), true);
    /// ```
    fn insert(&self, index: T) -> Option<bool>;

    /// Removes the index from the set
    /// Returns `true` if the index was set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use index_set::{SharedBitSet, BitSet};
    /// use std::sync::atomic::AtomicU32;
    ///
    /// let mut bitset: [AtomicU32; 4] = Default::default();
    /// bitset.insert(42);
    /// assert_eq!(bitset.has(42), true);
    ///
    /// bitset.remove(42);
    /// assert_eq!(bitset.has(42), false);
    /// ```
    fn remove(&self, index: T) -> Option<bool>;
}

impl<Set, T> SharedBitSet<T> for &Set
where
    Set: SharedBitSet<T> + ?Sized,
{
    #[inline]
    fn clear(&self) {
        SharedBitSet::clear(*self);
    }

    #[inline]
    fn insert(&self, index: T) -> Option<bool> {
        SharedBitSet::insert(*self, index)
    }

    #[inline]
    fn remove(&self, index: T) -> Option<bool> {
        SharedBitSet::remove(*self, index)
    }
}

macro_rules! impl_shared_bit_set {
    [$($ty:tt for $target: ty)*] => {$(
        impl SharedBitSet<$ty> for [$target] {
            fn clear(&self) {
                for slot in self.iter() {
                    slot.store(0, Ordering::Release);
                }
            }

            #[inline]
            fn insert(&self, index: $ty) -> Option<bool> {
                let slot_idx = usize::try_from(index / $ty::BITS as $ty).ok()?;
                let mask = 1 << (index % $ty::BITS as $ty);

                let slot = self
                    .get(slot_idx)?
                    .fetch_or(mask, Ordering::Release);

                Some(slot & mask != 0)
            }

            #[inline]
            fn remove(&self, index: $ty) -> Option<bool> {
                let slot_idx = usize::try_from(index / $ty::BITS as $ty).ok()?;
                let mask = 1 << (index % $ty::BITS as $ty);

                let slot = self
                    .get(slot_idx)?
                    .fetch_and(!mask, Ordering::Release);

                Some(slot & mask != 0)
            }
        }
    )*};
}

impl_shared_bit_set! {
    u32 for AtomicU32
    u64 for AtomicU64
    usize for AtomicUsize
}
