pub fn extract_vector(src: &[u8], start_pos: usize, end_pos: usize) -> Vec<u8> {
    let mut dest = vec![0; end_pos - start_pos];

    for x in start_pos..end_pos {
        dest[x - start_pos] = src[x];
    }

    dest
}

pub fn array_copy(src: Vec<u8>, dest: &mut [u8]) {
    for x in 0..src.len() {
        dest[x] = src[x];
    }
}