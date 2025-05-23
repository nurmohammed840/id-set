use crate::*;

/// A trait for reading values from a bit set.
pub trait BitSet<T> {
    /// Returns the number of bits that can be stored in the set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use index_set::BitSet;
    ///
    /// let bitset: &[u32] = &[0; 4];
    /// assert_eq!(bitset.capacity(), 128);
    /// ```
    fn capacity(&self) -> T;

    /// Returns `true` if the set contains the given value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use index_set::{BitSet, BitSetMut};
    ///
    /// let mut bitset: [u32; 4] = [0; 4];
    /// bitset.insert(0);
    /// assert!(bitset.has(0));
    /// ```
    fn has(&self, _: T) -> bool;

    /// Returns `true` if the set is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use index_set::{BitSet, BitSetMut};
    ///
    /// let mut bitset: [u32; 4] = [0; 4];
    /// assert!(BitSet::is_empty(&bitset[..]));
    ///
    /// bitset.insert(0);
    /// assert!(!BitSet::is_empty(&bitset[..]));
    /// ```
    fn is_empty(&self) -> bool;

    /// Returns the number of values in the set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use index_set::{BitSet, BitSetMut};
    ///
    /// let mut bitset: [u32; 4] = [0; 4];
    ///
    /// bitset.insert(0);
    /// assert_eq!(bitset.size(), 1);
    /// ```
    fn size(&self) -> T;
}

macro_rules! impl_deref {
    ($($target: ty),*) => {$(
        impl<Set, T> BitSet<T> for $target
        where
            Set: BitSet<T> + ?Sized,
        {
            #[inline]
            fn capacity(&self) -> T {
                BitSet::capacity(&**self)
            }

            #[inline]
            fn has(&self, index: T) -> bool {
                BitSet::has(&**self, index)
            }

            #[inline]
            fn is_empty(&self) -> bool {
                BitSet::is_empty(&**self)
            }

            #[inline]
            fn size(&self) -> T {
                BitSet::size(&**self)
            }
        }
    )*}
}

impl_deref! {
    &Set, Box<Set>
}

macro_rules! impl_bit_set {
    [$($ty:tt),*] => {$(
        impl BitSet<$ty> for [$ty] {
            #[inline]
            fn capacity(&self) -> $ty {
                self.len() as $ty * $ty::BITS as $ty
            }

            #[inline]
            fn has(&self, index: $ty) -> bool {
                 let slot_idx = match usize::try_from(index / $ty::BITS as $ty) {
                    Ok(slot_idx) => slot_idx,
                    Err(_) => return false,
                };
                let mask = 1 << (index % $ty::BITS as $ty);
                self.get(slot_idx).is_some_and(|slot| slot & mask != 0)
            }

            #[inline]
            fn is_empty(&self) -> bool {
                self.iter().all(|&slot| slot == 0)
            }

            #[inline]
            fn size(&self) -> $ty {
                self.iter().map(|slot| slot.count_ones() as $ty).sum()
            }
        }
    )*};
}

macro_rules! impl_atomic_bit_set {
    [$($ty:tt for $target: ty)*] => {$(
        impl BitSet<$ty> for [$target] {
            fn capacity(&self) -> $ty {
                self.len() as $ty * $ty::BITS as $ty
            }

            #[inline]
            fn has(&self, index: $ty) -> bool {
                let slot_idx = match usize::try_from(index / $ty::BITS as $ty) {
                    Ok(slot_idx) => slot_idx,
                    Err(_) => return false,
                };
                let mask = 1 << (index % $ty::BITS as $ty);
                self.get(slot_idx)
                    .is_some_and(|slot| slot.load(Ordering::Acquire) & mask != 0)
            }

            fn is_empty(&self) -> bool {
                self.iter().all(|slot| slot.load(Ordering::Acquire) == 0)
            }

            fn size(&self) -> $ty {
                self.iter()
                    .map(|slot| slot.load(Ordering::Acquire).count_ones() as $ty)
                    .sum()
            }
        }
    )*};
}

impl_bit_set! {
    u32, u64, usize, u128
}

impl_atomic_bit_set! {
    u32 for AtomicU32
    u64 for AtomicU64
    usize for AtomicUsize
}
