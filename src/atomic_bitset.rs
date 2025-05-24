use crate::*;

/// Same as `[AtomicUsize; N]`, but with an additional functionality.
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
    /// use index_set::{AtomicBitSet, slot_count, BitSet, SharedBitSet};
    /// // Create a new AtomicBitSet with memory size of 1 kilobyte
    /// static BIT_SET: AtomicBitSet<{ slot_count::from_kilobytes(1) }> = AtomicBitSet::new();
    /// assert_eq!(BIT_SET.set_next_free_bit(), Some(0));
    /// 
    /// BIT_SET.insert(2);
    /// assert_eq!(BIT_SET.set_next_free_bit(), Some(1));
    /// assert_eq!(BIT_SET.set_next_free_bit(), Some(3));
    ///
    /// BIT_SET.remove(1);
    /// assert_eq!(BIT_SET.has(1), false);
    /// assert_eq!(BIT_SET.set_next_free_bit(), Some(1));
    ///
    /// assert_eq!(BIT_SET.size(), 4);
    /// // it can hold up to 8192 unique identifiers.
    /// assert_eq!(BIT_SET.capacity(), 8192);
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
