//! Grapheme utilities on rope slices

use ropey::RopeSlice;
use unicode_segmentation::GraphemeCursor;

pub fn len_graphemes(rope: RopeSlice) -> usize {
    let mut cursor = GraphemeCursor::new(0, rope.len_bytes(), true);

    let chunks = rope.chunks();

    let mut start_idx = 0;
    let mut chunks_with_start_indices = chunks.map(|chunk| {
        let current_start_idx = start_idx;
        start_idx += chunk.len();
        (chunk, current_start_idx)
    });

    let mut num_graphemes = 0;

    let Some(mut current_chunk) = chunks_with_start_indices.next() else {
        return 0;
    };

    loop {
        let boundary = cursor.next_boundary(current_chunk.0, current_chunk.1);

        match boundary {
            Ok(Some(_grapheme_boundary)) => {
                num_graphemes += 1;
            }
            Ok(None) => break,
            Err(_grapheme_incomplete) => {
                current_chunk = chunks_with_start_indices
                    .next()
                    .expect("next chunk to be present, because current grapheme was incomplete");
            }
        }
    }

    num_graphemes
}

#[cfg(test)]
mod tests {
    use ropey::Rope;

    use crate::graphemes::len_graphemes;

    #[test]
    fn simple() {
        let s = Rope::from_str("Hello world!");

        assert_eq!(len_graphemes(s.slice(..)), 12);
    }
}
