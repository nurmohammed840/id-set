# id-set

`id-set` is a library for creating and managing sets of unique identifiers.

## Features

- Reuses identifiers when they are removed from the set.
- Atomic and thread-safe operations.
- Constant-time (`O(1)`) insertion, removal and lookup of identifiers.
- Compact memory usage, Each identifier is represented by a bit in the memory.
- Identifiers are unique and as small as possible.

## Why use `id-set`?

In our use case, We needed to track the online/offline status of millions of users with minimal memory usage and lightning-fast lookup performance.

We use `id_set::IdAllocator` to generate unique identifiers for each users and `id_set::IdSet` to track user's online status.
Resulting in just 2 bits of memory used per user.

## How it works

`id-set` pre-allocates a fixed amount of memory. Where each bit represents one unique identifier.
For example, `1` megabyte of memory can store `8` millions (`8,388,608`) unique identifiers.

When an identifier is removed, it is recycled for future use, ensuring smallest possible identifiers.

## Example

```rust
use id_set::{IdAllocator, slot_count};

// Create a new IdAllocator with a size of 1 kilobyte,
static IDS: IdAllocator<{ slot_count::from_kb(1) }> = IdAllocator::new();

fn main() {
    assert_eq!(IDS.next_id(), Some(0));

    IDS.set(2);
    assert_eq!(IDS.next_id(), Some(1));
    assert_eq!(IDS.next_id(), Some(3));

    IDS.remove(1);
    assert_eq!(IDS.has(1), false);
    assert_eq!(IDS.next_id(), Some(1));

    assert_eq!(IDS.total_ids(), 4);

    // it can hold up to 8192 unique identifiers.
    assert_eq!(IDS.mem_size() * 8, 8192);
}
```