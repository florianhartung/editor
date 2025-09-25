//! Types for communication between the app and the frontend

/// Events are emitted by frontends and get sent to the core application
/// They describe state changes, that are not frontend-specific.
#[derive(Debug, Clone)]
pub enum Event {
    // only one editor for now, thus no id required
    Editor(/* editor_id, */ EditorEvent),
}

#[derive(Debug, Clone)]
pub enum EditorEvent {
    KeyPress(char),
}

/// Actions are emitted by the core to the editor state and the frontend.
/// Actions may or may not result from handling of an event
#[derive(Debug, Clone)]
pub enum Action {
    Insert(char),
}
