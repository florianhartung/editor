//! Types for communication between the app and the frontend

use ropey::Rope;

/// An event emitted by the frontend.
///
/// This is send to the main application.
#[derive(Debug, Clone)]
pub enum FrontendEvent {
    /// A key was pressed by the user.
    KeyPress(Key),

    /// A new element was focused.
    Focus(FocusTarget),

    // TODO use custom CoreOsString type that is converted into a [u8] when send
    // to the frontend and validated when it is received in the core
    /// A file was requested to be opened.
    OpenFile { path: String },
}

#[derive(Debug, Clone)]
pub enum FocusTarget {
    // only one editor for now, thus no id required
    Editor, /* { id: usize } */
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
    Backspace,
    Enter,
    Tab,
    Up,
    Down,
    Left,
    Right,
}

/// App events are emitted by the main applicatin and send to the frontend. They
/// may or may not result from handling of an event.
#[derive(Debug, Clone)]
pub enum AppEvent {
    HintBufferViewLoading,
    SetBufferView { contents: Rope, title: String },
    SetBufferViewContents(Rope),
    SetBufferViewCursorPosition { line: usize, column: usize },
    Batched(Vec<AppEvent>),
    // TODO allow core to request program termination?
    // RequestExit,
}
