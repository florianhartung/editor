use core::communication::{Action, Key, KeyNamed};

use tokio::sync::mpsc::{UnboundedSender, error::SendError};

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
        send_action: &UnboundedSender<Action>,
    ) -> Result<(), SendError<Action>> {
        let line = buffer.current_cursor_position_line();
        buffer.set_cursor_position_saturating(line.saturating_sub(1), self.cursor_optimal_column);
        send_action.send(buffer.full_info_to_action())
    }

    fn down(
        &mut self,
        buffer: &mut Buffer,
        send_action: &UnboundedSender<Action>,
    ) -> Result<(), SendError<Action>> {
        let line = buffer.current_cursor_position_line();
        buffer.set_cursor_position_saturating(line + 1, self.cursor_optimal_column);
        send_action.send(buffer.full_info_to_action())
    }

    fn left(
        &mut self,
        buffer: &mut Buffer,
        send_action: &UnboundedSender<Action>,
    ) -> Result<(), SendError<Action>> {
        buffer.cursor_backward();
        self.cursor_optimal_column = buffer.current_cursor_position_column();
        send_action.send(buffer.full_info_to_action())
    }

    fn right(
        &mut self,
        buffer: &mut Buffer,
        send_action: &UnboundedSender<Action>,
    ) -> Result<(), SendError<Action>> {
        buffer.cursor_forward();
        self.cursor_optimal_column = buffer.current_cursor_position_column();
        send_action.send(buffer.full_info_to_action())
    }

    fn handle_key_press_normal(
        &mut self,
        buffer: &mut Buffer,
        key: Key,
        send_action: &UnboundedSender<Action>,
    ) -> Result<(), SendError<Action>> {
        debug_assert_eq!(self.current_mode, Mode::Normal);
        match key {
            Key::Named(KeyNamed::Backspace) => self.left(buffer, send_action),
            Key::Char('h') => self.left(buffer, send_action),
            Key::Char('j') => self.down(buffer, send_action),
            Key::Char('k') => self.up(buffer, send_action),
            Key::Char('l') | Key::Char(' ') => self.right(buffer, send_action),
            Key::Char('i') => {
                self.current_mode = Mode::Insert;
                Ok(())
            }
            // TODO pretty much all other hotkeys
            _ => Ok(()),
        }
    }

    fn handle_key_press_insert(
        &mut self,
        buffer: &mut Buffer,
        key: Key,
        send_action: &UnboundedSender<Action>,
    ) -> Result<(), SendError<Action>> {
        debug_assert_eq!(self.current_mode, Mode::Insert);
        match key {
            Key::Named(KeyNamed::Escape) => {
                self.current_mode = Mode::Normal;
            }
            Key::Named(KeyNamed::Backspace) => {
                buffer.remove_char();
                send_action.send(buffer.full_info_to_action())?;
            }
            Key::Named(KeyNamed::Enter) => {
                buffer.insert_new_line();
                send_action.send(buffer.full_info_to_action())?;
            }
            Key::Named(KeyNamed::Tab) => {
                let num_spaces_to_insert = 4 - buffer.current_cursor_position_column() % 4;
                const SPACES: &str = "    ";
                buffer.insert(&SPACES[0..num_spaces_to_insert]);
                send_action.send(buffer.full_info_to_action())?;
            }
            Key::Char(c) => {
                buffer.insert_char(c);
                send_action.send(buffer.full_info_to_action())?;
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
        send_action: &UnboundedSender<Action>,
    ) -> Result<(), SendError<Action>> {
        match key {
            // Arrow keys are always movements
            Key::Named(KeyNamed::Up) => self.up(buffer, send_action),
            Key::Named(KeyNamed::Down) => self.down(buffer, send_action),
            Key::Named(KeyNamed::Left) => self.left(buffer, send_action),
            Key::Named(KeyNamed::Right) => self.right(buffer, send_action),
            _ => match self.current_mode {
                Mode::Normal => self.handle_key_press_normal(buffer, key, send_action),
                Mode::Insert => self.handle_key_press_insert(buffer, key, send_action),
            },
        }
    }
}
