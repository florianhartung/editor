//! The entry point for the text editor application.
//! It calls the core with a specific frontend.

use anyhow::Result;
use core::communication::{Action, EditorEvent, Event};
use core::frontend::Frontend;
use log::info;
use ropey::Rope;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub fn run<F: Frontend>() -> Result<()> {
    let mut editor = Editor::new();

    // These are the two main communication channels
    let (send_event, receive_event) = mpsc::unbounded_channel::<Event>();
    let (send_action, receive_action) = mpsc::unbounded_channel::<Action>();

    // Spawn the backend main loop in a separate task
    tokio::task::spawn(async move {
        let mut receive_event = UnboundedReceiverStream::new(receive_event);
        while let Some(event) = receive_event.next().await {
            match event {
                Event::Editor(ee @ EditorEvent::KeyPress(c)) => {
                    info!("handling event {ee:?}");
                    editor
                        .buffer
                        .rope
                        .insert_char(editor.buffer.rope.len_chars(), c);

                    // editor.perform(&action);
                    if send_action.send(Action::Insert(c)).is_err() {
                        log::debug!("frontend action receiver is dropped");
                    }
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
