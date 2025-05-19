use std::mem;

/// Returns the number of slots needed to store the given number of bits.
pub const fn from_bits(n: usize) -> usize {
    n / usize::BITS as usize
}

/// Returns the number of slots available in the given number of bytes.
pub const fn from_bytes(n: usize) -> usize {
    n / mem::size_of::<usize>()
}

/// Returns the number of slots available in the given number of kilobytes.
pub const fn from_kb(n: usize) -> usize {
    n * 1024 / mem::size_of::<usize>()
}

/// Returns the number of slots available in the given number of megabytes.
pub const fn from_mb(n: usize) -> usize {
    n * 1024 * 1024 / mem::size_of::<usize>()
}
