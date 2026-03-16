//! Grapheme utilities on rope slices

use ropey::RopeSlice;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};

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

/// Returns the char index of the start of the current or previous grapheme.
///
/// - The char index can be one-past-the-end.
/// - The returned char index saturates at 0.
pub fn prev_grapheme_char_index(rope: RopeSlice, char_idx: usize) -> usize {
    debug_assert!(char_idx <= rope.len_chars(), "char index is out-of-bounds");

    if char_idx == 0 {
        return 0;
    }

    let byte_idx = rope.char_to_byte(char_idx);

    let mut cursor = GraphemeCursor::new(byte_idx, rope.len_bytes(), true);

    let (mut chunks, chunk_byte_idx, _, _) = rope.chunks_at_char(char_idx);

    let first_chunk = chunks
        .next()
        .map(|first_chunk| (first_chunk, chunk_byte_idx));

    let (mut chunk, mut chunk_byte_idx) = first_chunk.unwrap_or_else(|| {
        let (chunk, chunk_byte_idx, _, _) = rope
            .get_chunk_at_char(char_idx.checked_sub(1).expect("char index to be non zero"))
            .expect("at least one chunk to exist because rope slice is not empty");

        (chunk, chunk_byte_idx)
    });

    loop {
        match cursor.prev_boundary(chunk, chunk_byte_idx) {
            Ok(Some(grapheme_boundary)) => return rope.byte_to_char(grapheme_boundary),
            Ok(None) => return 0,
            Err(GraphemeIncomplete::PrevChunk) => {
                let (prev_chunk, prev_chunk_byte_idx, _, _) = rope.chunk_at_byte(chunk_byte_idx.checked_sub(1).expect("there to be at least one more chunk. therefore the byte index of the current chunk must be non zero"));
                chunk = prev_chunk;
                chunk_byte_idx = prev_chunk_byte_idx;
            }
            Err(GraphemeIncomplete::PreContext(_)) => todo!("provide more context"),
            Err(GraphemeIncomplete::NextChunk) => unreachable!("only possible for next_boundary"),
            Err(GraphemeIncomplete::InvalidOffset) => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use ropey::Rope;

    use crate::graphemes::{len_graphemes, prev_grapheme_char_index};

    #[test]
    fn simple() {
        let rope = Rope::from_str("Hello world!");

        assert_eq!(len_graphemes(rope.slice(..)), 12);
    }

    #[test]
    fn prev_grapheme() {
        let rope = Rope::from_str("A\u{1100}\u{1161}\u{11A8}BC");

        assert_eq!(prev_grapheme_char_index(rope.slice(..), 0), 0);
        assert_eq!(prev_grapheme_char_index(rope.slice(..), 1), 0);
        assert_eq!(prev_grapheme_char_index(rope.slice(..), 2), 1);
        assert_eq!(prev_grapheme_char_index(rope.slice(..), 3), 1);
        assert_eq!(prev_grapheme_char_index(rope.slice(..), 4), 1);
        assert_eq!(prev_grapheme_char_index(rope.slice(..), 5), 4);
        assert_eq!(prev_grapheme_char_index(rope.slice(..), 6), 5);

        let rope = Rope::from_str("\u{1100}\u{1161}AB");
        assert_eq!(prev_grapheme_char_index(rope.slice(..), 0), 0);
        assert_eq!(prev_grapheme_char_index(rope.slice(..), 1), 0);
        assert_eq!(prev_grapheme_char_index(rope.slice(..), 2), 0);
        assert_eq!(prev_grapheme_char_index(rope.slice(..), 3), 2);
        assert_eq!(prev_grapheme_char_index(rope.slice(..), 4), 3);
    }
}
