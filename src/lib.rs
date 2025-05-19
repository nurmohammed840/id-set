#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![allow(clippy::new_without_default)]

/// A module that provides functions to calculate the number of slots.
pub mod slot_count;

use std::{
    fmt, mem,
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

/// A thread-safe id manager that uses atomic operations to manage ids.
/// It uses a bitmask to represent the ids, where each bit represents an id.
#[derive(Debug)]
pub struct IdSet<const SLOT_COUNT: usize> {
    slots: [AtomicUsize; SLOT_COUNT],
}

const SLOT_SIZE: usize = usize::BITS as usize;

impl<const N: usize> IdSet<N> {
    /// Creates a new `IdSet<N>` with the specified number of slots.
    /// Each slot can hold 32/64 ids depending on the architecture.
    #[inline]
    pub const fn new() -> Self {
        Self {
            slots: [const { AtomicUsize::new(0) }; N],
        }
    }

    // #[inline]
    // #[doc(hidden)]
    // pub fn slots(&self) -> &[AtomicUsize] {
    //     &self.slots
    // }

    /// Returns the number of memory in bytes.
    #[inline]
    pub const fn mem_size(&self) -> usize {
        mem::size_of::<Self>()
    }

    /// Clears all the ids.
    pub fn clear(&self) {
        for slot in &self.slots {
            slot.store(0, Ordering::Release);
        }
    }

    /// Returns `true` if the id is set
    ///
    /// ## Examples
    ///
    /// ```rust
    ///  let ids = id_set::IdSet::<2>::new();
    ///  assert_eq!(ids.has(0), false);
    /// ```
    #[inline]
    pub fn has(&self, id: usize) -> bool {
        let slot_idx = id / SLOT_SIZE;
        let mask = 1 << (id % SLOT_SIZE);

        self.slots
            .get(slot_idx)
            .is_some_and(|slot| slot.load(Ordering::Acquire) & mask != 0)
    }

    /// Sets the id to `true`
    ///
    /// Returns `Ok(true)` if the id was not set before, otherwise `Ok(false)`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// let ids = id_set::IdSet::<2>::new();
    /// assert_eq!(ids.has(0), false);
    /// ids.set(0);
    /// assert_eq!(ids.has(0), true);
    /// ```
    #[inline]
    pub fn set(&self, id: usize) -> Result<bool, OutOfBoundsError> {
        let slot_idx = id / SLOT_SIZE;
        let mask = 1 << (id % SLOT_SIZE);

        let slot = self
            .slots
            .get(slot_idx)
            .ok_or(OutOfBoundsError)?
            .fetch_or(mask, Ordering::Release);

        Ok(slot & mask != 0)
    }

    /// Sets the id to `false`
    ///
    /// Returns `Ok(true)` if the id was set before, otherwise `Ok(false)`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// let ids = id_set::IdSet::<2>::new();
    /// assert_eq!(ids.has(0), false);
    /// ids.set(0);
    /// assert_eq!(ids.has(0), true);
    /// ids.remove(0);
    /// assert_eq!(ids.has(0), false);
    /// ```
    #[inline]
    pub fn remove(&self, id: usize) -> Result<bool, OutOfBoundsError> {
        let slot_idx = id / SLOT_SIZE;
        let mask = 1 << (id % SLOT_SIZE);

        let slot = self
            .slots
            .get(slot_idx)
            .ok_or(OutOfBoundsError)?
            .fetch_and(!mask, Ordering::Release);

        Ok(slot & mask != 0)
    }

    /// Returns the number of ids that are set to `true`
    ///
    /// # Examples
    ///
    /// ```rust
    /// let ids = id_set::IdSet::<2>::new();
    /// assert_eq!(ids.total_ids(), 0);
    /// ids.set(0);
    /// ids.set(1);
    /// assert_eq!(ids.total_ids(), 2);
    /// ```
    #[inline]
    pub fn total_ids(&self) -> usize {
        self.slots
            .iter()
            .map(|slot| slot.load(Ordering::Acquire).count_ones() as usize)
            .sum()
    }
}

/// It uses a bitmask to represent the ids, where each bit represents an id.
#[derive(Debug)]
pub struct IdAllocator<const SLOT_COUNT: usize> {
    id_set: IdSet<SLOT_COUNT>,
    // used for optimizing the search for the next id.
    rotation: AtomicUsize,
}

impl<const N: usize> IdAllocator<N> {
    /// Creates a new `IdAllocator` with the specified number of slots.
    /// Each slot can hold 32/64 ids depending on the architecture.
    #[inline]
    pub const fn new() -> Self {
        Self {
            id_set: IdSet::<N>::new(),
            rotation: AtomicUsize::new(0),
        }
    }

    /// Allocates the next available id.
    /// Returns `None` if there is no space to allocate a new id
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use id_set::{IdAllocator, slot_count};
    /// let ids: IdAllocator<{ slot_count::from_bits(128) }> = IdAllocator::new();
    /// assert_eq!(ids.next_id(), Some(0));
    /// assert_eq!(ids.next_id(), Some(1));
    /// ```
    pub fn next_id(&self) -> Option<usize> {
        // rotate the slots to find the next free id
        let skip = self.rotation.load(Ordering::Acquire);
        let mut slot_idx = skip;

        let slots = rotate_left(&self.slots, skip);

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
                    self.rotation.store(slot_idx, Ordering::Release);
                }
                let next_available_bit = (!curr).trailing_zeros() as usize;
                return Some(slot_idx * SLOT_SIZE + next_available_bit);
            }

            slot_idx += 1;
            if slot_idx >= N {
                slot_idx = 0;
            }
        }
        None
    }
}

impl<const N: usize> Deref for IdAllocator<N> {
    type Target = IdSet<N>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.id_set
    }
}

/// This error is returned when trying to set or remove an id that is out of bounds.
#[derive(Debug, PartialEq, Eq)]

pub struct OutOfBoundsError;

impl std::error::Error for OutOfBoundsError {}
impl fmt::Display for OutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id out of bounds")
    }
}

fn rotate_left<T>(slice: &[T], n: usize) -> impl Iterator<Item = &T> {
    let (left, right) = slice.split_at(n);
    right.iter().chain(left)
}

#[cfg(test)]
mod tests {
    #![allow(unused_must_use)]
    use super::*;

    #[test]
    fn test_id_manager() {
        let ids: IdAllocator<{ slot_count::from_bits(128) }> = IdAllocator::new();

        assert_eq!(ids.has(0), false);
        ids.set(0);
        assert_eq!(ids.has(0), true);

        assert_eq!(ids.next_id(), Some(1));
        assert_eq!(ids.has(1), true);

        ids.set(2);
        assert_eq!(ids.next_id(), Some(3));
        assert_eq!(ids.next_id(), Some(4));
        assert_eq!(ids.next_id(), Some(5));

        ids.remove(4);
        assert_eq!(ids.has(4), false);
        assert_eq!(ids.next_id(), Some(4));

        while let Some(_) = ids.next_id() {}

        assert_eq!(ids.next_id(), None);

        assert_eq!(ids.total_ids(), 128);
        ids.clear();
        assert_eq!(ids.total_ids(), 0);
    }

    #[test]
    fn test_prev_value() {
        let ids: IdAllocator<{ slot_count::from_bits(64) }> = IdAllocator::new();

        assert_eq!(ids.remove(0), Ok(false));
        assert_eq!(ids.set(0), Ok(false));
        assert_eq!(ids.set(0), Ok(true));
        assert_eq!(ids.remove(0), Ok(true));
        assert_eq!(ids.remove(0), Ok(false));

        assert!(ids.set(65).is_err());
    }
}
