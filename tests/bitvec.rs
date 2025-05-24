use index_set::{BitSet, BitSetMut};


#[test]
fn test_bitvec() {
    let mut bitset: Vec<u32> = Vec::new();

    assert!(BitSetMut::insert(&mut bitset, 42).is_ok());
    assert!(BitSet::has(&bitset[..], 42));

    assert_eq!(BitSetMut::remove(&mut bitset, 42), Some(true));
    assert_eq!(BitSetMut::remove(&mut bitset, 0), Some(false));
}