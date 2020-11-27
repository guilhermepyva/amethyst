pub fn array_copy(src: &[u8], start_pos: usize, end_pos: usize) -> Vec<u8> {
    let mut dest = vec![0; end_pos - start_pos];

    for x in start_pos..end_pos {
        dest[x - start_pos] = src[x];
    }

    dest
}