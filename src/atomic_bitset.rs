use crate::*;

/// A thread-safe bitset that uses atomic operations to manage bits.
pub struct AtomicBitSet<const N: usize> {
    bitset: [AtomicUsize; N],
    // used for optimizing the search to find the next free bit
    rotation: AtomicUsize,
}

impl<const N: usize> AtomicBitSet<N> {
    /// Creates a new `AtomicBitSet` with the specified number of slots.
    /// Each slot can hold 32/64 bits depending on the architecture.
    /// 
    /// ## Examples
    /// 
    /// ```rust
    /// use index_set::{AtomicBitSet, slot_count};
    /// 
    /// let bitset: AtomicBitSet<{ slot_count::from_bits(128) }> = AtomicBitSet::new();
    /// ```
    #[inline]
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            bitset: [const { AtomicUsize::new(0) }; N],
            rotation: AtomicUsize::new(0),
        }
    }

    /// Atomically finds the next free bit (unset bit with value `0`) in the bitset, sets it to `1`,
    /// and returns its index.
    ///
    /// This method is thread-safe and can be used in concurrent environments.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use index_set::{AtomicBitSet, slot_count};
    /// let ids: AtomicBitSet<{ slot_count::from_bits(128) }> = AtomicBitSet::new();
    /// assert_eq!(ids.set_next_free_bit(), Some(0));
    /// assert_eq!(ids.set_next_free_bit(), Some(1));
    /// ```
    pub fn set_next_free_bit(&self) -> Option<usize> {
        // rotate the slots to find the next free id
        let skip = self.rotation.load(Ordering::Relaxed);
        let mut slot_idx = skip;

        let slots = utils::rotate_left(&self.bitset, skip);

        for slot in slots {
            let available_slot = slot.fetch_update(Ordering::AcqRel, Ordering::Acquire, |curr| {
                // slot is full
                if curr == usize::MAX {
                    return None;
                }
                let next_available_bit = (!curr).trailing_zeros() as usize;
                Some(curr | (1 << next_available_bit))
            });

            if let Ok(curr) = available_slot {
                if skip != slot_idx {
                    self.rotation.store(slot_idx, Ordering::Relaxed);
                }
                let next_available_bit = (!curr).trailing_zeros() as usize;
                return Some(slot_idx * usize::BITS as usize + next_available_bit);
            }

            slot_idx += 1;
            if slot_idx >= N {
                slot_idx = 0;
            }
        }
        None
    }
}

impl<const N: usize> std::ops::Deref for AtomicBitSet<N> {
    type Target = [AtomicUsize];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.bitset
    }
}
