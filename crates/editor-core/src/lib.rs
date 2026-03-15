use tokio::sync::mpsc::UnboundedSender;

use crate::communication::{FocusTarget, FrontendEvent, Key};

pub mod communication;
pub mod frontend;

#[derive(Clone)]
pub struct FrontendEventSender(UnboundedSender<FrontendEvent>);

impl FrontendEventSender {
    pub fn new(sender: UnboundedSender<FrontendEvent>) -> Self {
        Self(sender)
    }

    pub fn send(&self, event: FrontendEvent) {
        self.0
            .send(event)
            .expect("the event receiver, i.e. the core, to outlive the frontend");
    }

    pub fn focus_editor(&self /* , editor_id: usize */) {
        self.send(FrontendEvent::Focus(FocusTarget::Editor));
    }

    pub fn key_press(&self, key: Key) {
        self.send(FrontendEvent::KeyPress(key));
    }

    pub fn open_file(&self, path: String) {
        self.send(FrontendEvent::OpenFile { path });
    }
}
