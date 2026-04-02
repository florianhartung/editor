//! Grapheme utilities on rope slices

use ropey::{
    RopeSlice,
    str_utils::{byte_to_char_idx, char_to_byte_idx},
};
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};

pub fn len_graphemes(rope: RopeSlice) -> usize {
    let (mut cursor, mut chunk) = new_cursor_and_chunk(rope, 0);

    let mut num_graphemes = 0;
    loop {
        match cursor.next_boundary(chunk.0, chunk.1) {
            Ok(Some(_grapheme_boundary)) => {
                num_graphemes += 1;
            }
            Ok(None) => {
                break num_graphemes;
            }
            Err(grapheme_incomplete) => {
                complete_grapheme(grapheme_incomplete, rope, &mut chunk);
            }
        }
    }
}

/// Returns the char index of the start of the next grapheme.
pub fn next_grapheme_boundary(rope: RopeSlice, char_idx: usize) -> usize {
    nth_next_grapheme_boundary(rope, char_idx, 1)
}

/// Returns the char index of the start of the n-th next grapheme.
pub fn nth_next_grapheme_boundary(rope: RopeSlice, char_idx: usize, n: usize) -> usize {
    let n = isize::try_from(n).expect("that n is usually small enough");
    nth_grapheme_boundary(rope, char_idx, n)
}

/// Returns the char index of the start of the previous grapheme (or the current
/// if the char index is not at the start already).
pub fn prev_grapheme_boundary(rope: RopeSlice, char_idx: usize) -> usize {
    nth_prev_grapheme_boundary(rope, char_idx, 1)
}

/// Returns the char index of the start of the n-th previous grapheme (or the
/// current if the char index is not at the start already).
pub fn nth_prev_grapheme_boundary(rope: RopeSlice, char_idx: usize, n: usize) -> usize {
    let n = isize::try_from(n).expect("that n is usually small enough");
    nth_grapheme_boundary(rope, char_idx, -n)
}

/// Returns the char index of the start of the n-th next or previous grapheme
/// (depending on whether n is positive or negative).
///
/// - The char index can be one-past-the-end.
/// - The returned char index saturates at one-past-the-end.
///
/// For convenience, this returns the char_idx when n is 0.
pub fn nth_grapheme_boundary(rope: RopeSlice, char_idx: usize, mut n: isize) -> usize {
    let (mut cursor, mut chunk) = new_cursor_and_chunk(rope, char_idx);

    if n == 0 {
        return char_idx;
    }
    let signum = n.signum();
    n -= signum;
    let boundary_fn = if signum > 0 {
        GraphemeCursor::next_boundary
    } else {
        GraphemeCursor::prev_boundary
    };

    let grapheme_boundary_byte_idx = loop {
        match boundary_fn(&mut cursor, chunk.0, chunk.1) {
            Ok(Some(grapheme_boundary_byte_idx)) => {
                if n == 0 {
                    break grapheme_boundary_byte_idx;
                }
                n -= signum;
            }
            Ok(None) => return if signum > 0 { rope.len_chars() } else { 0 },
            Err(grapheme_incomplete) => complete_grapheme(grapheme_incomplete, rope, &mut chunk),
        }
    };

    // Convert byte index to char index efficiently because we have the chunk already
    chunk.2 + byte_to_char_idx(chunk.0, grapheme_boundary_byte_idx - chunk.1)
}

fn new_cursor_and_chunk<'a>(
    rope: RopeSlice<'a>,
    char_idx: usize,
) -> (GraphemeCursor, (&'a str, usize, usize, usize)) {
    let chunk = rope.chunk_at_char(char_idx);

    let byte_idx = chunk.1 + char_to_byte_idx(chunk.0, char_idx - chunk.2);
    let cursor = GraphemeCursor::new(byte_idx, rope.len_bytes(), true);

    (cursor, chunk)
}

/// Completes a grapheme as requested by the [`GraphemeIncomplete`] error.
fn complete_grapheme<'a>(
    grapheme_incomplete: GraphemeIncomplete,
    rope: RopeSlice<'a>,
    chunk: &mut (&'a str, usize, usize, usize),
) {
    match grapheme_incomplete {
        GraphemeIncomplete::PrevChunk => {
            let prev_chunk_last_byte_idx = chunk
                .1
                .checked_sub(1)
                .expect("that the current chunk is not the first one");
            *chunk = rope.chunk_at_byte(prev_chunk_last_byte_idx);
        }
        GraphemeIncomplete::NextChunk => {
            let next_chunk_first_byte_idx = chunk.1 + chunk.0.len();
            *chunk = rope.chunk_at_byte(next_chunk_first_byte_idx);
        }
        GraphemeIncomplete::PreContext(_) => todo!("provide more context"),
        GraphemeIncomplete::InvalidOffset => panic!(),
    }
}

#[cfg(test)]
mod tests {
    use ropey::Rope;

    use crate::graphemes::{
        len_graphemes, next_grapheme_boundary, nth_next_grapheme_boundary,
        nth_prev_grapheme_boundary, prev_grapheme_boundary,
    };

    #[test]
    fn simple() {
        let rope = Rope::from_str("Hello world!");

        assert_eq!(len_graphemes(rope.slice(..)), 12);
    }

    #[test]
    fn prev_grapheme() {
        let rope = Rope::from_str("A\u{1100}\u{1161}\u{11A8}BC");
        let rope = rope.slice(..);

        assert_eq!(prev_grapheme_boundary(rope, 0), 0);
        assert_eq!(prev_grapheme_boundary(rope, 1), 0);
        assert_eq!(prev_grapheme_boundary(rope, 2), 1);
        assert_eq!(prev_grapheme_boundary(rope, 3), 1);
        assert_eq!(prev_grapheme_boundary(rope, 4), 1);
        assert_eq!(prev_grapheme_boundary(rope, 5), 4);
        assert_eq!(prev_grapheme_boundary(rope, 6), 5);

        let rope = Rope::from_str("\u{1100}\u{1161}AB");
        let rope = rope.slice(..);
        assert_eq!(prev_grapheme_boundary(rope, 0), 0);
        assert_eq!(prev_grapheme_boundary(rope, 1), 0);
        assert_eq!(prev_grapheme_boundary(rope, 2), 0);
        assert_eq!(prev_grapheme_boundary(rope, 3), 2);
        assert_eq!(prev_grapheme_boundary(rope, 4), 3);
    }

    #[test]
    fn next_grapheme() {
        let rope = Rope::from_str("A\u{1100}\u{1161}\u{11A8}BC");
        let rope = rope.slice(..);

        assert_eq!(next_grapheme_boundary(rope, 0), 1);
        assert_eq!(next_grapheme_boundary(rope, 1), 4);
        assert_eq!(next_grapheme_boundary(rope, 2), 4);
        assert_eq!(next_grapheme_boundary(rope, 3), 4);
        assert_eq!(next_grapheme_boundary(rope, 4), 5);
        assert_eq!(next_grapheme_boundary(rope, 5), 6);
        assert_eq!(next_grapheme_boundary(rope, 5), 6);
        assert_eq!(next_grapheme_boundary(rope, 6), 6);

        let rope = Rope::from_str("AB\u{1100}\u{1161}");
        let rope = rope.slice(..);
        assert_eq!(next_grapheme_boundary(rope, 0), 1);
        assert_eq!(next_grapheme_boundary(rope, 1), 2);
        assert_eq!(next_grapheme_boundary(rope, 2), 4);
        assert_eq!(next_grapheme_boundary(rope, 3), 4);
        assert_eq!(next_grapheme_boundary(rope, 4), 4);
    }

    #[test]
    fn nth_operations() {
        let rope = Rope::from_str("A\u{1100}\u{1161}\u{11A8}BC");
        let rope = rope.slice(..);

        assert_eq!(nth_next_grapheme_boundary(rope, 0, 1), 1);
        assert_eq!(nth_next_grapheme_boundary(rope, 0, 2), 4);
        assert_eq!(nth_next_grapheme_boundary(rope, 0, 3), 5);
        assert_eq!(nth_next_grapheme_boundary(rope, 0, 4), 6);
        assert_eq!(nth_next_grapheme_boundary(rope, 0, 5), 6);

        assert_eq!(nth_prev_grapheme_boundary(rope, 6, 1), 5);
        assert_eq!(nth_prev_grapheme_boundary(rope, 6, 2), 4);
        assert_eq!(nth_prev_grapheme_boundary(rope, 6, 3), 1);
        assert_eq!(nth_prev_grapheme_boundary(rope, 6, 4), 0);
        assert_eq!(nth_prev_grapheme_boundary(rope, 6, 5), 0);
    }
}
