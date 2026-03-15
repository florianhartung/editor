use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use editor_core::communication::{AppEvent, Key, KeyNamed};

use crate::{buffer::Buffer, motions::Motions};

#[derive(Debug)]
pub struct NonModalMotions {
    cursor_optimal_column: usize,
}

impl NonModalMotions {
    pub fn new(buffer: &Buffer) -> Self {
        Self {
            cursor_optimal_column: buffer.current_cursor_position_column(),
        }
    }
}

impl Motions for NonModalMotions {
    fn handle_key_press(
        &mut self,
        buffer: &mut Buffer,
        key: Key,
        event_sender: &UnboundedSender<AppEvent>,
    ) -> Result<(), SendError<AppEvent>> {
        match key {
            editor_core::communication::Key::Named(KeyNamed::Escape) => {
                todo!()
            }
            editor_core::communication::Key::Named(KeyNamed::Backspace) => {
                buffer.remove_char();
                event_sender.send(buffer.full_info_to_app_event())?;
            }
            editor_core::communication::Key::Named(KeyNamed::Enter) => {
                buffer.insert_new_line();
                event_sender.send(buffer.full_info_to_app_event())?;
            }
            editor_core::communication::Key::Named(KeyNamed::Tab) => {
                let num_spaces_to_insert = 4 - buffer.current_cursor_position_column() % 4;

                const SPACES: &str = "    ";
                buffer.insert(&SPACES[0..num_spaces_to_insert]);
                event_sender.send(buffer.full_info_to_app_event())?;
            }
            editor_core::communication::Key::Char(c) => {
                buffer.insert_char(c);

                event_sender.send(buffer.full_info_to_app_event())?;
            }
            editor_core::communication::Key::Named(KeyNamed::Up) => {
                let line = buffer.current_cursor_position_line();

                buffer.set_cursor_position_saturating(
                    line.saturating_sub(1),
                    self.cursor_optimal_column,
                );
                event_sender.send(buffer.full_info_to_app_event())?;
            }
            editor_core::communication::Key::Named(KeyNamed::Down) => {
                let line = buffer.current_cursor_position_line();

                buffer.set_cursor_position_saturating(line + 1, self.cursor_optimal_column);
                event_sender.send(buffer.full_info_to_app_event())?;
            }
            editor_core::communication::Key::Named(KeyNamed::Left) => {
                buffer.cursor_backward();
                self.cursor_optimal_column = buffer.current_cursor_position_column();
                event_sender.send(buffer.full_info_to_app_event())?;
            }
            editor_core::communication::Key::Named(KeyNamed::Right) => {
                buffer.cursor_forward();
                self.cursor_optimal_column = buffer.current_cursor_position_column();
                event_sender.send(buffer.full_info_to_app_event())?;
            }
        }

        Ok(())
    }
}
