use core::communication::{EditorEvent, Event};

use tokio::sync::mpsc::UnboundedSender;

use iced::{
    Element, Subscription, Task, keyboard,
    widget::{column, text, text_editor, text_editor::Edit},
};
use log::debug;

#[derive(Default)]
pub struct App {
    current_editor: text_editor::Content,
}

#[derive(Debug, Clone)]
pub enum Message {
    FromEditorAction(text_editor::Action),
    ToEditorAction(text_editor::Action),
    // KeyPress(char),
    Event(iced::Event),
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
                    if let keyboard::Event::KeyPressed { text: Some(s), .. } = keyboard_event {
                        assert_eq!(s.len(), 1);
                        let first_char = s.chars().next().unwrap();
                        debug!("Hello world");

                        send_event
                            .send(Event::Editor(EditorEvent::KeyPress(first_char)))
                            .unwrap();
                    }
                }
                _ => {}
            },
            Message::FromEditorAction(action) => match action {
                text_editor::Action::Edit(Edit::Insert(c)) => {
                    send_event
                        .send(Event::Editor(EditorEvent::KeyPress(c)))
                        .unwrap();
                }
                _ => {}
            },
            Message::ToEditorAction(action) => self.current_editor.perform(action),
        };

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![
            text("hello"),
            text("world"),
            text_editor(&self.current_editor)
                .placeholder("bla")
                .on_action(Message::FromEditorAction),
        ]
        .into()
    }
}
