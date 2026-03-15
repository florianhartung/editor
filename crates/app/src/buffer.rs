use core::communication::Action;

use ropey::Rope;

/// A buffer is the in-memory representation of a document.
///
/// It is not possible to view the same buffer more than once at a time, for
/// example in different tabs/windows.
#[derive(Debug, Default)]
pub struct Buffer {
    rope: Rope,
    /// The cursor position in chars. This points at where the next char will be inserted.
    ///
    /// Note: The cursor position is uninterpreted when `rope` is of length 0.
    cursor_position: usize,
}

impl Buffer {
    pub fn new(initial_contents: Rope) -> Self {
        Self {
            rope: initial_contents,
            cursor_position: 0,
        }
    }

    /// Returns new cursor position
    pub fn insert_char(&mut self, c: char) {
        self.rope.insert_char(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn insert(&mut self, s: &str) {
        self.rope.insert(self.cursor_position, s);
        self.cursor_position += s.len();
    }

    pub fn remove_char(&mut self) {
        if self.rope.len_chars() > 0 && self.cursor_position > 0 {
            self.rope
                .remove((self.cursor_position - 1)..(self.cursor_position));
            self.cursor_position -= 1;
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

        self.cursor_position = start_of_line + column_index;
    }

    /// Simple relative cursor movement
    pub fn cursor_forward(&mut self) {
        self.cursor_position = self
            .cursor_position
            .saturating_add(1)
            .min(self.rope.len_chars());
    }

    /// Simple relative cursor movement
    pub fn cursor_backward(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    pub fn current_cursor_position_column(&self) -> usize {
        let start_of_line = self.rope.line_to_char(self.current_cursor_position_line());
        self.cursor_position - start_of_line
    }

    pub fn current_cursor_position_line(&self) -> usize {
        self.rope.char_to_line(self.cursor_position)
    }

    pub fn cursor_seek_next_word_start(&mut self, include_punctuation_in_words: bool) {
        let chars_from_cursor = self.rope.chars_at(self.cursor_position);
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

        let new_cursor_position = self.cursor_position + num_chars_to_skip;
        self.cursor_position = new_cursor_position.min(self.rope.len_chars());
    }

    pub fn insert_new_line(&mut self) {
        // TODO add support for different line endings
        self.insert_char('\n');
    }

    pub fn full_info_to_action(&self) -> Action {
        Action::Batched(vec![
            Action::SetBufferContents(self.rope.clone()),
            Action::SetCursorPosition {
                line: self.current_cursor_position_line(),
                column: self.current_cursor_position_column(),
            },
        ])
    }
}
