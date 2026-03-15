use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use editor_core::communication::{AppEvent, Key, KeyNamed};

use crate::{buffer::Buffer, motions::Motions};

#[derive(Debug)]
pub struct HelixMotions {
    cursor_optimal_column: usize,
    current_mode: Mode,
}

#[derive(Debug, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
}

impl HelixMotions {
    pub fn new(buffer: &Buffer) -> Self {
        Self {
            cursor_optimal_column: buffer.current_cursor_position_column(),
            current_mode: Mode::Normal,
        }
    }

    fn up(
        &mut self,
        buffer: &mut Buffer,
        event_sender: &UnboundedSender<AppEvent>,
    ) -> Result<(), SendError<AppEvent>> {
        let line = buffer.current_cursor_position_line();
        buffer.set_cursor_position_saturating(line.saturating_sub(1), self.cursor_optimal_column);
        event_sender.send(buffer.full_info_to_app_event())
    }

    fn down(
        &mut self,
        buffer: &mut Buffer,
        event_sender: &UnboundedSender<AppEvent>,
    ) -> Result<(), SendError<AppEvent>> {
        let line = buffer.current_cursor_position_line();
        buffer.set_cursor_position_saturating(line + 1, self.cursor_optimal_column);
        event_sender.send(buffer.full_info_to_app_event())
    }

    fn left(
        &mut self,
        buffer: &mut Buffer,
        event_sender: &UnboundedSender<AppEvent>,
    ) -> Result<(), SendError<AppEvent>> {
        buffer.cursor_backward();
        self.cursor_optimal_column = buffer.current_cursor_position_column();
        event_sender.send(buffer.full_info_to_app_event())
    }

    fn right(
        &mut self,
        buffer: &mut Buffer,
        event_sender: &UnboundedSender<AppEvent>,
    ) -> Result<(), SendError<AppEvent>> {
        buffer.cursor_forward();
        self.cursor_optimal_column = buffer.current_cursor_position_column();
        event_sender.send(buffer.full_info_to_app_event())
    }

    fn handle_key_press_normal(
        &mut self,
        buffer: &mut Buffer,
        key: Key,
        event_sender: &UnboundedSender<AppEvent>,
    ) -> Result<(), SendError<AppEvent>> {
        debug_assert_eq!(self.current_mode, Mode::Normal);
        match key {
            Key::Named(KeyNamed::Backspace) => self.left(buffer, event_sender),
            Key::Char('h') => self.left(buffer, event_sender),
            Key::Char('j') => self.down(buffer, event_sender),
            Key::Char('k') => self.up(buffer, event_sender),
            Key::Char('l') | Key::Char(' ') => self.right(buffer, event_sender),
            Key::Char('i') => {
                self.current_mode = Mode::Insert;
                Ok(())
            }
            Key::Char('w') => {
                buffer.cursor_seek_next_word_start(false);
                event_sender.send(buffer.full_info_to_app_event())
            }
            Key::Char('W') => {
                buffer.cursor_seek_next_word_start(true);
                event_sender.send(buffer.full_info_to_app_event())
            }
            // TODO pretty much all other hotkeys
            _ => Ok(()),
        }
    }

    fn handle_key_press_insert(
        &mut self,
        buffer: &mut Buffer,
        key: Key,
        event_sender: &UnboundedSender<AppEvent>,
    ) -> Result<(), SendError<AppEvent>> {
        debug_assert_eq!(self.current_mode, Mode::Insert);
        match key {
            Key::Named(KeyNamed::Escape) => {
                self.current_mode = Mode::Normal;
            }
            Key::Named(KeyNamed::Backspace) => {
                buffer.remove_char();
                event_sender.send(buffer.full_info_to_app_event())?;
            }
            Key::Named(KeyNamed::Enter) => {
                buffer.insert_new_line();
                event_sender.send(buffer.full_info_to_app_event())?;
            }
            Key::Named(KeyNamed::Tab) => {
                let num_spaces_to_insert = 4 - buffer.current_cursor_position_column() % 4;
                const SPACES: &str = "    ";
                buffer.insert(&SPACES[0..num_spaces_to_insert]);
                event_sender.send(buffer.full_info_to_app_event())?;
            }
            Key::Char(c) => {
                buffer.insert_char(c);
                event_sender.send(buffer.full_info_to_app_event())?;
            }
            _ => {}
        }

        Ok(())
    }
}

impl Motions for HelixMotions {
    fn handle_key_press(
        &mut self,
        buffer: &mut Buffer,
        key: Key,
        event_sender: &UnboundedSender<AppEvent>,
    ) -> Result<(), SendError<AppEvent>> {
        match key {
            // Arrow keys are always movements
            Key::Named(KeyNamed::Up) => self.up(buffer, event_sender),
            Key::Named(KeyNamed::Down) => self.down(buffer, event_sender),
            Key::Named(KeyNamed::Left) => self.left(buffer, event_sender),
            Key::Named(KeyNamed::Right) => self.right(buffer, event_sender),
            _ => match self.current_mode {
                Mode::Normal => self.handle_key_press_normal(buffer, key, event_sender),
                Mode::Insert => self.handle_key_press_insert(buffer, key, event_sender),
            },
        }
    }
}
