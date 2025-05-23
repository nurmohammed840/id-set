#![doc = include_str!("../README.md")]
// #![no_std]

mod atomic_bitset;
mod bitset;
mod bitset_mut;
mod shared_bitset;
mod utils;

/// A module that provides functions to calculate the number of slots.
pub mod slot_count;

pub use atomic_bitset::AtomicBitSet;
pub use bitset::BitSet;
pub use bitset_mut::BitSetMut;
pub use shared_bitset::SharedBitSet;

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
