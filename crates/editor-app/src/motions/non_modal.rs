use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use editor_core::communication::{AppEvent, Key, KeyNamed};

use crate::{buffer::Buffer, motions::Motions};

#[derive(Debug)]
pub struct NonModalMotions;

impl NonModalMotions {
    pub fn new(buffer: &Buffer) -> Self {
        Self
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
            Key::Named(KeyNamed::Escape) => {
                todo!()
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
            Key::Named(KeyNamed::Up) => {
                buffer.cursor_up();
                event_sender.send(buffer.full_info_to_app_event())?;
            }
            Key::Named(KeyNamed::Down) => {
                buffer.cursor_down();
                event_sender.send(buffer.full_info_to_app_event())?;
            }
            Key::Named(KeyNamed::Left) => {
                buffer.cursor_backward();
                event_sender.send(buffer.full_info_to_app_event())?;
            }
            Key::Named(KeyNamed::Right) => {
                buffer.cursor_forward();
                event_sender.send(buffer.full_info_to_app_event())?;
            }
        }

        Ok(())
    }
}
