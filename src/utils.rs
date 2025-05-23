pub fn rotate_left<T>(slice: &[T], n: usize) -> impl Iterator<Item = &T> {
    let (left, right) = slice.split_at(n);
    right.iter().chain(left)
}
