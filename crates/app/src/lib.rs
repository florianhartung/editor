//! The entry point for the text editor application.
//! It calls the core with a specific frontend.

use anyhow::Result;
use core::communication::{Action, EditorEvent, Event, KeyNamed};
use core::frontend::Frontend;
use ropey::Rope;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::time::sleep;
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
            let event_handling_result = editor.handle_event(event, &send_action).await;
            if event_handling_result.is_err() {
                break;
            }
        }
    });

    F::run(send_event, receive_action).await;

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

    async fn handle_event(
        &mut self,
        event: Event,
        send_action: &UnboundedSender<Action>,
    ) -> Result<(), SendError<Action>> {
        match event {
            Event::Editor(EditorEvent::KeyPress(key)) => {
                match key {
                    core::communication::Key::Named(KeyNamed::Escape) => {
                        todo!()
                    }
                    core::communication::Key::Char(c) => {
                        self.buffer
                            .rope
                            .insert_char(self.buffer.rope.len_chars(), c);

                        send_action.send(Action::SetBufferContents(self.buffer.rope.clone()))?;
                    }
                }

                // editor.perform(&action);
            }
            Event::OpenFile { path } => {
                let path = PathBuf::from(path);
                let file = File::open(&path).unwrap();
                send_action.send(Action::HintBufferLoading)?;

                self.buffer.rope = Rope::from_reader(BufReader::new(file)).unwrap();
                sleep(Duration::from_secs(2)).await;

                send_action.send(Action::SetBuffer {
                    contents: self.buffer.rope.clone(),
                    title: path.file_name().unwrap().to_owned().into_string().unwrap(),
                })?;
            }
        }

        Ok(())
    }

    //     fn perform(&mut self, _action: &Action) {
    //         todo!();
    //     }
}

#[derive(Debug, Default)]
pub struct Buffer {
    rope: Rope,
}
