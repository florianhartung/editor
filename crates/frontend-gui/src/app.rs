use editor_core::communication::{EditorEvent, Event, Key, KeyNamed};

use iced::keyboard::key::Named;
use iced_futures::keyboard;
use tokio::sync::mpsc::UnboundedSender;

use iced::widget::text_editor;
use iced::{
    widget::{
        column, text,
        text_editor::{Edit, KeyPress, Status},
    },
    Element, Subscription, Task,
};
use log::debug;

#[derive(Default)]
pub struct App {
    current_editor: text_editor::Content,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Actions from the editor that were emitted as a result of some user action
    FromEditorAction(text_editor::Action),
    /// Actions that will update the editor state
    ToEditorAction(text_editor::Action),
    // KeyPress(char),
    Event(iced::Event),
    EditorKeyPress {
        key: iced::keyboard::Key,
    },
}

impl App {
    pub fn new() -> Self {
        Self {
            current_editor: text_editor::Content::with_text("placeholder"),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen_raw(|event, _status, _id| {
            matches!(event, iced::Event::Keyboard(_)).then_some(Message::Event(event))
        })
    }

    pub fn update(
        &mut self,
        message: Message,
        send_event: &UnboundedSender<Event>,
    ) -> Task<Message> {
        match message {
            Message::Event(event) => match event {
                iced::Event::Keyboard(keyboard_event) => {
                    // if let keyboard::Event::KeyPressed { text: Some(s), .. } = keyboard_event {
                    //     assert_eq!(s.len(), 1);
                    //     let first_char = s.chars().next().unwrap();
                    //     debug!("Hello world");

                    //     send_event
                    //         .send(Event::Editor(EditorEvent::KeyPress(first_char)))
                    //         .unwrap();
                    // }
                }
                _ => {}
            },
            Message::FromEditorAction(action) => {
                self.current_editor.perform(action);
                // match action {
                //         text_editor::Action::Edit(Edit::Insert(c)) => {
                //             send_event
                //                 .send(Event::Editor(EditorEvent::KeyPress(c)))
                //                 .unwrap();
                //         }
                //         _ => {}
                //     },
            }
            Message::ToEditorAction(action) => self.current_editor.perform(action),
            // Message::EditorKey(key) => {
            //     key.
            //     todo!()
            // }
            Message::EditorKeyPress { key } => {
                let core_key = match key {
                    iced::keyboard::Key::Named(Named::Escape) => Key::Named(KeyNamed::Escape),
                    iced::keyboard::Key::Character(smol_str) => {
                        // We only care about one char for now
                        let first_char = smol_str.chars().next().unwrap();
                        Key::Char(first_char)
                    }
                    iced::keyboard::Key::Unidentified => todo!(),
                    _ => return Task::none(),
                };

                send_event
                    .send(Event::Editor(EditorEvent::KeyPress(core_key)))
                    .unwrap();
            }
        };

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![
            text("hello"),
            text("world"),
            text_editor(&self.current_editor)
                .placeholder("bla")
                .key_binding(|key: KeyPress| {
                    (key.status == Status::Focused).then(|| {
                        text_editor::Binding::Custom(Message::EditorKeyPress { key: key.key })
                    })
                })
                .on_action(Message::FromEditorAction)
        ]
        .into()
    }
}
