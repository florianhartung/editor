use tokio::sync::mpsc::UnboundedSender;

use crate::communication::{EditorEvent, Event, Key};

pub mod communication;
pub mod frontend;

#[derive(Clone)]
pub struct EventSender(UnboundedSender<Event>);

impl EventSender {
    pub fn new(sender: UnboundedSender<Event>) -> Self {
        Self(sender)
    }

    pub fn send(&self, event: Event) {
        self.0
            .send(event)
            .expect("the event receiver, i.e. the core, to outlive the frontend");
    }

    pub fn editor_event(&self, editor_event: EditorEvent) {
        self.send(Event::Editor(editor_event));
    }

    pub fn key_press(&self, key: Key) {
        self.editor_event(EditorEvent::KeyPress(key));
    }

    pub fn open_file(&self, path: String) {
        self.send(Event::OpenFile { path });
    }
}
