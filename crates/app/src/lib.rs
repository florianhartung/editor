//! The entry point for the text editor application.
//! It calls the core with a specific frontend.

use anyhow::Result;
use core::communication::{Action, EditorEvent, Event, KeyNamed};
use core::frontend::Frontend;
use ropey::Rope;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::main]
pub async fn run<F: Frontend>() -> Result<()> {
    let mut editor = Editor::new();

    // These are the two main communication channels
    let (send_event, receive_event) = mpsc::unbounded_channel::<Event>();
    let (send_action, receive_action) = mpsc::unbounded_channel::<Action>();

    // Spawn the backend main loop in a separate task
    tokio::task::spawn(async move {
        let mut receive_event = UnboundedReceiverStream::new(receive_event);
        while let Some(event) = receive_event.next().await {
            log::debug!("handling event {event:?}");
            match event {
                Event::Editor(EditorEvent::KeyPress(key)) => {
                    match key {
                        core::communication::Key::Named(KeyNamed::Escape) => {
                            todo!()
                        }
                        core::communication::Key::Char(c) => {
                            editor
                                .buffer
                                .rope
                                .insert_char(editor.buffer.rope.len_chars(), c);

                            let _ = send_action.send(Action::Insert(c));
                        }
                    }

                    // editor.perform(&action);
                }
            }
        }
    });

    F::run(send_event, receive_action);

    Ok(())
}

#[derive(Debug)]
pub struct Editor {
    pub buffer: Buffer,
    // An index for the currently opened buffer into [`Self::buffers`].
    // If there are no buffers, the value of this is undefined.
    // pub current_buffer: usize,
}

impl Editor {
    fn new() -> Self {
        Self {
            buffer: Buffer::default(),
            // buffers: vec![Buffer::default()],
            // current_buffer: 0,
        }
    }

    //     fn perform(&mut self, _action: &Action) {
    //         todo!();
    //     }
}

#[derive(Debug, Default)]
pub struct Buffer {
    rope: Rope,
}
