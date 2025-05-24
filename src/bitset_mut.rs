/// A trait for a mutate values in a bit set.
pub trait BitSetMut<T> {
    /// Clears the set
    ///
    /// # Example
    ///
    /// ```rust
    /// use index_set::{BitSet, BitSetMut};
    ///
    /// let mut bitset: [u32; 4] = [0; 4];
    /// bitset.insert(0);
    /// assert!(!BitSet::is_empty(&bitset[..]));
    ///
    /// bitset.clear();
    /// assert!(BitSet::is_empty(&bitset[..]));
    /// ```
    fn clear(&mut self);

    /// Inserts the value into the set.
    ///
    /// Returns `Ok(true)` if the value was already set.
    /// Returns `Err(usize)` if the set cannot hold the value, where `usize` is the index of the slot.
    ///
    /// # Example
    ///
    /// ```rust
    /// use index_set::{BitSet, BitSetMut};
    ///
    /// let mut bitset: [u32; 4] = [0; 4];
    /// bitset.insert(0);
    /// assert_eq!(bitset.has(0), true);
    /// ```
    fn insert(&mut self, _: T) -> Result<bool, usize>;

    /// Removes the value from the set
    ///
    /// Returns `Some(true)` if the value was already set.
    /// Returns `None` if the set cannot hold the value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use index_set::{BitSet, BitSetMut};
    ///
    /// let mut bitset: [u32; 4] = [0; 4];
    /// bitset.insert(42);
    /// assert_eq!(bitset.has(42), true);
    ///
    /// bitset.remove(42);
    /// assert_eq!(bitset.has(42), false);
    /// ```
    fn remove(&mut self, _: T) -> Option<bool>;
}

impl<T> BitSetMut<T> for Vec<T>
where
    T: Default + Clone,
    [T]: BitSetMut<T>,
{
    #[inline]
    fn clear(&mut self) {
        Vec::clear(self);
    }

    fn insert(&mut self, value: T) -> Result<bool, usize> {
        match self.as_mut_slice().insert(value.clone()) {
            Ok(has) => Ok(has),
            Err(slot_index) => {
                self.resize(slot_index + 1, T::default());
                self.as_mut_slice().insert(value)
            }
        }
    }

    #[inline]
    fn remove(&mut self, value: T) -> Option<bool> {
        self.as_mut_slice().remove(value)
    }
}

macro_rules! impl_deref_mut {
    ($($target: ty),*) => {$(
        impl<Set, T> BitSetMut<T> for $target
        where
            Set: BitSetMut<T> + ?Sized,
        {
            #[inline]
            fn clear(&mut self) {
                BitSetMut::clear(&mut **self)
            }

            #[inline]
            fn insert(&mut self, index: T) -> Result<bool, usize> {
                BitSetMut::insert(&mut **self, index)
            }

            #[inline]
            fn remove(&mut self, index: T) -> Option<bool> {
                BitSetMut::remove(&mut **self, index)
            }
        }
    )*}
}

impl_deref_mut! {
    &mut Set, Box<Set>
}

macro_rules! impl_bit_set_mut {
    [$($ty:tt),*] => {$(
        impl BitSetMut<$ty> for [$ty] {
            fn clear(&mut self) {
                for slot in self {
                    *slot = 0;
                }
            }

            #[inline]
            fn insert(&mut self, index: $ty) -> Result<bool, usize> {
                let slot_idx = usize::try_from(index / $ty::BITS as $ty).unwrap();
                let mask = 1 << (index % $ty::BITS as $ty);
                let slot = self.get_mut(slot_idx).ok_or(slot_idx)?;

                let old_value = *slot & mask != 0;
                *slot |= mask;
                Ok(old_value)
            }

            #[inline]
            fn remove(&mut self, index: $ty) -> Option<bool> {
                let slot_idx = usize::try_from(index / $ty::BITS as $ty).ok()?;
                let mask = 1 << (index % $ty::BITS as $ty);
                let slot = self.get_mut(slot_idx)?;

                let old_value = *slot & mask != 0;
                *slot &= !mask;
                Some(old_value)
            }
        }
    )*};
}

impl_bit_set_mut! {
    u16, u32, u64, usize, u128
}
