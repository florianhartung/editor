//! Types for communication between the app and the frontend

use ropey::Rope;

/// Events are emitted by frontends and get sent to the core application
/// They describe state changes, that are not frontend-specific.
#[derive(Debug, Clone)]
pub enum Event {
    // only one editor for now, thus no id required
    Editor(/* editor_id, */ EditorEvent),
    OpenFile { path: String },
}

#[derive(Debug, Clone)]
pub enum EditorEvent {
    KeyPress(Key),
}

#[derive(Debug, Clone)]
pub enum Key {
    Named(KeyNamed),
    Char(char),
}

#[derive(Debug, Clone)]
pub enum KeyNamed {
    Escape,
}

/// Actions are emitted by the core to the editor state and the frontend.
/// Actions may or may not result from handling of an event
#[derive(Debug, Clone)]
pub enum Action {
    HintBufferLoading,
    SetBuffer { contents: Rope, title: String },
    SetBufferContents(Rope),
}
