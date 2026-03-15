//! The entry point for the text editor application.
//! It calls the core with a specific frontend.

use std::{fs::File, io::BufReader, path::PathBuf, time::Duration};

use anyhow::Result;
use ropey::Rope;
use tokio::{
    sync::mpsc::{self, UnboundedSender, error::SendError},
    time::sleep,
};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};

use editor_core::{
    EventSender,
    communication::{Action, EditorEvent, Event},
    frontend::Frontend,
};

use crate::{
    buffer::Buffer,
    motions::{HelixMotions, Motions, NonModalMotions},
};

mod buffer;
mod motions;

#[tokio::main]
pub async fn run<F: Frontend>(post_init: impl FnOnce(&EventSender)) -> Result<()> {
    let mut editor = Editor::new();

    // These are the two main communication channels
    let (send_event, receive_event) = mpsc::unbounded_channel::<Event>();
    let (send_action, receive_action) = mpsc::unbounded_channel::<Action>();

    let event_sender = EventSender::new(send_event);

    post_init(&event_sender);

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

    F::run(event_sender, receive_action).await;

    Ok(())
}

#[derive(Debug)]
pub struct Editor {
    pub motions: Box<dyn Motions>,
    pub buffer: Buffer,
    // An index for the currently opened buffer into [`Self::buffers`].
    // If there are no buffers, the value of this is undefined.
    // pub current_buffer: usize,
}

impl Editor {
    fn new() -> Self {
        let buffer = Buffer::default();
        // let motions = Box::new(NonModalMotions::new(&buffer));
        let motions = Box::new(HelixMotions::new(&buffer));

        Self {
            motions,
            buffer,
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
            Event::Editor(editor_event) => self.handle_editor_event(editor_event, send_action)?,
            Event::OpenFile { path } => {
                let path = PathBuf::from(path);
                let file = File::open(&path).unwrap();
                send_action.send(Action::HintBufferLoading)?;

                self.buffer = Buffer::new(Rope::from_reader(BufReader::new(file)).unwrap());
                sleep(Duration::from_secs(1)).await;

                send_action.send(self.buffer.full_info_to_action())?;
            }
        }

        Ok(())
    }

    fn handle_editor_event(
        &mut self,
        editor_event: EditorEvent,
        send_action: &UnboundedSender<Action>,
    ) -> Result<(), SendError<Action>> {
        match editor_event {
            EditorEvent::KeyPress(key) => {
                self.motions
                    .handle_key_press(&mut self.buffer, key, send_action)?;
                // editor.perform(&action);
            }
        }

        Ok(())
    }

    //     fn perform(&mut self, _action: &Action) {
    //         todo!();
    //     }
}
