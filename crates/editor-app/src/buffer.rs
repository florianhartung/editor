use ropey::Rope;

use editor_core::communication::AppEvent;

use crate::graphemes;

/// A buffer is the in-memory representation of a document providing operations
/// on a per-grapheme or per-char granularity.
///
/// It is not possible to view the same buffer more than once at a time, for
/// example in different tabs/windows.
#[derive(Debug, Default)]
pub struct Buffer {
    rope: Rope,
    /// The cursor index in chars.
    ///
    /// It must always be in range `0..=C` with `C` being the current rope
    /// length in chars.
    cursor_char_idx: usize,

    /// In graphmes
    cursor_preferred_column_idx: usize,
}

impl Buffer {
    pub fn new(initial_contents: Rope) -> Self {
        Self {
            rope: initial_contents,
            cursor_char_idx: 0,
            cursor_preferred_column_idx: 0,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.rope.insert_char(self.cursor_char_idx, c);
        self.cursor_char_idx += 1;
    }

    pub fn insert(&mut self, s: &str) {
        self.rope.insert(self.cursor_char_idx, s);
        self.cursor_char_idx += s.len();
    }

    pub fn remove_char(&mut self) {
        if self.rope.len_chars() > 0 && self.cursor_char_idx > 0 {
            self.rope
                .remove((self.cursor_char_idx - 1)..(self.cursor_char_idx));
            self.cursor_char_idx -= 1;
        }
    }

    /// Absolute cursor movement.
    pub fn set_cursor_position_saturating(&mut self, line_index: usize, column_index: usize) {
        // Saturate line index
        let line_index = line_index.min(self.rope.len_lines().saturating_sub(1));
        let start_of_line = self.rope.line_to_char(line_index);
        let line = self.rope.line(line_index);

        // Saturate column index
        let column_index = column_index.min(line.len_chars().saturating_sub(1));

        self.cursor_char_idx = start_of_line + column_index;
    }

    /// Simple relative cursor movement
    pub fn cursor_forward(&mut self) {
        self.cursor_char_idx =
            graphemes::next_grapheme_boundary(self.rope.slice(..), self.cursor_char_idx);
        // TODO might be a bit wasteful, maybe store column in a field
        self.cursor_preferred_column_idx = self.current_cursor_position_column();
    }

    /// Simple relative cursor movement
    pub fn cursor_backward(&mut self) {
        self.cursor_char_idx =
            graphemes::prev_grapheme_boundary(self.rope.slice(..), self.cursor_char_idx);
        // TODO might be a bit wasteful, maybe store column in a field
        self.cursor_preferred_column_idx = self.current_cursor_position_column();
    }

    pub fn current_cursor_position_column(&self) -> usize {
        let line_idx = self.rope.char_to_line(self.cursor_char_idx);
        let line_start_char_idx = self.rope.line_to_char(line_idx);
        let line_until_cursor = self.rope.slice(line_start_char_idx..self.cursor_char_idx);
        graphemes::len_graphemes(line_until_cursor)
    }

    pub fn current_cursor_position_line(&self) -> usize {
        self.rope.char_to_line(self.cursor_char_idx)
    }

    pub fn cursor_seek_next_word_start(&mut self, include_punctuation_in_words: bool) {
        let chars_from_cursor = self.rope.chars_at(self.cursor_char_idx);
        let num_chars_to_skip = chars_from_cursor
            .skip(1)
            .scan(false, |found_whitespace, next| {
                *found_whitespace |= next.is_whitespace()
                    | (!include_punctuation_in_words && next.is_ascii_punctuation());

                match (found_whitespace, next.is_whitespace()) {
                    (true, true) => Some(next),
                    (true, false) => None,
                    (false, _) => Some(next),
                }
            })
            .count();

        let new_cursor_position = self.cursor_char_idx + num_chars_to_skip;
        self.cursor_char_idx = new_cursor_position.min(self.rope.len_chars());
    }

    pub fn insert_new_line(&mut self) {
        // TODO add support for different line endings
        self.insert_char('\n');
    }

    pub fn full_info_to_app_event(&self) -> AppEvent {
        AppEvent::Batched(vec![
            AppEvent::SetBufferViewContents(self.rope.clone()),
            AppEvent::SetBufferViewCursorPosition {
                line: self.current_cursor_position_line(),
                column: self.current_cursor_position_column(),
            },
        ])
    }

    // TODO maybe use unicode width instead of number of graphemes?
    pub fn cursor_up(&mut self) {
        let line_idx = self.rope.char_to_line(self.cursor_char_idx);
        if line_idx == 0 {
            // TODO maybe add flag for this saturating behavior?
            self.cursor_char_idx = 0;
            return;
        }
        let prev_line_idx = line_idx - 1;
        let prev_line_char_idx = self.rope.line_to_char(prev_line_idx);
        self.move_to_preferred_column_in_line(prev_line_char_idx);
    }

    // TODO maybe use unicode width instead of number of graphemes?
    pub fn cursor_down(&mut self) {
        let line_idx = self.rope.char_to_line(self.cursor_char_idx);
        if line_idx == self.rope.len_lines() {
            self.cursor_char_idx = self.rope.len_chars();
            return;
        }
        let next_line_idx = line_idx + 1;
        let next_line_char_idx = self.rope.line_to_char(next_line_idx);
        self.move_to_preferred_column_in_line(next_line_char_idx);
    }

    fn move_to_preferred_column_in_line(&mut self, line_char_idx: usize) {
        let cursor_char_idx = graphemes::nth_next_grapheme_boundary(
            self.rope.slice(..),
            line_char_idx,
            self.cursor_preferred_column_idx,
        );

        let line_idx = self.rope.char_to_line(line_char_idx);
        let line_len_chars = self.rope.line(line_idx).len_chars();

        self.cursor_char_idx = cursor_char_idx.min(line_char_idx + line_len_chars - 1);
    }
}

#[cfg(test)]
mod tests {
    use ropey::Rope;

    use crate::buffer::Buffer;

    #[test]
    fn basic_manipulation() {
        let mut buf = Buffer::new(Rope::from_str("Hello world!\nExample text"));

        buf.set_cursor_position_saturating(1, 2);
        buf.remove_char();
        assert_eq!(buf.rope, "Hello world!\nEample text");

        buf.cursor_backward();
        buf.cursor_backward();

        buf.remove_char();
        assert_eq!(buf.rope, "Hello world\nEample text");
    }

    #[test]
    fn cursor_grapheme_cluster_granularity() {
        const SINGLE_GRAPHEME_CLUSTER: &str = "\u{1100}\u{1161}\u{11A8}"; // 3 code points (ᄀ ᅡ ᆨ) = 1 grapheme cluster (각)
        let mut buf = Buffer::new(Rope::from_str(SINGLE_GRAPHEME_CLUSTER));

        // Going forward and backward should always move entire graphemes
        buf.cursor_char_idx = 0;
        buf.cursor_forward();
        assert_eq!(buf.cursor_char_idx, 3);
        buf.cursor_backward();
        assert_eq!(buf.cursor_char_idx, 0);

        // If the cursor is in the middle of a grapheme, backwards movements
        // moves to the start of the current grapheme. Forwards movements moves
        // to the next grapheme.
        buf.cursor_char_idx = 2;
        buf.cursor_backward();
        assert_eq!(buf.cursor_char_idx, 0);
        buf.cursor_char_idx = 2;
        buf.cursor_forward();
        assert_eq!(buf.cursor_char_idx, 3);
    }

    #[test]
    fn cursor_saturating_movements() {
        const S: &str = "ABC";
        let mut buf = Buffer::new(Rope::from_str(S));

        buf.cursor_char_idx = 0;
        buf.cursor_backward();
        assert_eq!(buf.cursor_char_idx, 0);

        buf.cursor_char_idx = 3;
        buf.cursor_forward();
        assert_eq!(buf.cursor_char_idx, 3);
    }
}
