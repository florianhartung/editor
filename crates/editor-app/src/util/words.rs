//! Word utilities on rope slices

use ropey::RopeSlice;
use unicode_segmentation::UnicodeSegmentation;

/// Returns char_idx one after the end if past the last word boundary already
pub fn next_word_boundary(rope: RopeSlice, char_idx: usize) -> usize {
    let s = rope.slice(char_idx..).to_string();

    let mut word_indices = s.split_word_bound_indices().skip(1);
    let Some((word_boundary, _word)) = word_indices.next() else {
        return rope.len_chars();
    };

    word_boundary + char_idx
}

pub fn next_whitespace_word_boundary(rope: RopeSlice, char_idx: usize) -> usize {
    let chars = rope.chars_at(char_idx).enumerate();

    // Skip current word
    let chars = chars.skip_while(|(_idx, c)| !c.is_whitespace());

    // Skip whitespace
    let mut chars = chars.skip_while(|(_idx, c)| c.is_whitespace());
    let start_of_word_char_idx = chars.next().map(|(char_idx, _)| char_idx);

    // TODO return original char idex as default
    start_of_word_char_idx.unwrap_or(0) + char_idx
}

/// Returns the char index of the start of the previous word or the current word if the given char
/// index does not point to its start already.
pub fn previous_word_boundary(_rope: RopeSlice, _char_idx: usize) -> usize {
    todo!("seek previous word boundary")
}
