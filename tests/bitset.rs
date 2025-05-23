use index_set::{AtomicBitSet, BitSet, SharedBitSet, slot_count};
use std::sync::atomic::AtomicU32;

#[test]
fn test_id_set() {
    let bitset: AtomicBitSet<{ slot_count::from_bits(128) }> = AtomicBitSet::new();

    bitset.insert(0);
    assert_eq!(bitset.has(0), true);

    assert_eq!(bitset.set_next_free_bit(), Some(1));
    assert_eq!(bitset.has(1), true);

    bitset.insert(2);
    assert_eq!(bitset.set_next_free_bit(), Some(3));
    assert_eq!(bitset.set_next_free_bit(), Some(4));
    assert_eq!(bitset.set_next_free_bit(), Some(5));

    bitset.remove(4);
    assert_eq!(bitset.has(4), false);
    assert_eq!(bitset.set_next_free_bit(), Some(4));

    while let Some(_) = bitset.set_next_free_bit() {}
    assert_eq!(bitset.set_next_free_bit(), None);

    assert_eq!(bitset.size(), 128);
    bitset.clear();
    assert_eq!(bitset.size(), 0);
}

#[test]
fn test_prev_value() {
    let bitset = [AtomicU32::new(0); slot_count::from_bits(64)];

    assert_eq!(bitset.remove(0), Some(false));
    assert_eq!(bitset.insert(0), Some(false));
    assert_eq!(bitset.insert(0), Some(true));
    assert_eq!(bitset.remove(0), Some(true));
    assert_eq!(bitset.remove(0), Some(false));

    assert!(bitset.insert(65).is_none());
}
