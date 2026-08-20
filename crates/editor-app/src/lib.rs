//! The entry point for the text editor application.
//! It calls the core with a specific frontend.

use std::{fs::File, io::BufReader, path::PathBuf, time::Duration};

use anyhow::{Context, Result};
use ropey::Rope;
use tokio::{
    sync::mpsc::{self, UnboundedSender, error::SendError},
    time::sleep,
};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};

use editor_core::{
    FrontendEventSender,
    communication::{AppEvent, FocusTarget, FrontendEvent, Key},
    frontend::Frontend,
};

use crate::{
    buffer::Buffer,
    motions::{HelixMotions, Motions},
};

mod buffer;
mod motions;
mod util;

#[tokio::main]
pub async fn run<F: Frontend>(post_init: impl FnOnce(&FrontendEventSender)) -> Result<()> {
    // These are the two main communication channels
    let (frontend_event_sender, frontend_event_receiver) =
        mpsc::unbounded_channel::<FrontendEvent>();
    let (app_event_sender, app_event_receiver) = mpsc::unbounded_channel::<AppEvent>();

    let frontend_event_sender = FrontendEventSender::new(frontend_event_sender);

    let mut editor = Editor::new(app_event_sender).context("failed to initialize editor state")?;

    post_init(&frontend_event_sender);

    // Spawn the backend main loop in a separate task
    tokio::task::spawn(async move {
        let mut receive_event = UnboundedReceiverStream::new(frontend_event_receiver);
        while let Some(event) = receive_event.next().await {
            log::debug!("handling event {event:?}");
            let event_handling_result = editor.handle_event(event).await;
            if event_handling_result.is_err() {
                break;
            }
        }
    });

    F::run(frontend_event_sender, app_event_receiver).await;

    Ok(())
}

#[derive(Debug)]
pub struct Editor {
    pub event_sender: UnboundedSender<AppEvent>,

    pub motions: Box<dyn Motions>,
    pub buffer: Buffer,
    // An index for the currently opened buffer into [`Self::buffers`].
    // If there are no buffers, the value of this is undefined.
    // pub current_buffer: usize,
}

impl Editor {
    fn new(event_sender: UnboundedSender<AppEvent>) -> Result<Self, SendError<AppEvent>> {
        let buffer = Buffer::default();
        // let motions = Box::new(NonModalMotions::new(&buffer));
        let motions = Box::new(HelixMotions::new(&event_sender)?);

        Ok(Self {
            event_sender,
            motions,
            buffer,
            // buffers: vec![Buffer::default()],
            // current_buffer: 0,
        })
    }

    async fn handle_event(&mut self, event: FrontendEvent) -> Result<(), SendError<AppEvent>> {
        match event {
            FrontendEvent::KeyPress(key) => self.handle_key_press(key)?,
            FrontendEvent::Focus(FocusTarget::Editor) => {
                // TODO implement focus system
            }
            FrontendEvent::OpenFile { path } => {
                let path = PathBuf::from(path);
                let file = File::open(&path).unwrap();
                self.event_sender.send(AppEvent::HintBufferViewLoading)?;

                self.buffer = Buffer::new(Rope::from_reader(BufReader::new(file)).unwrap());
                sleep(Duration::from_secs(1)).await;

                self.event_sender
                    .send(self.buffer.full_info_to_app_event())?;
            }
        }

        Ok(())
    }

    fn handle_key_press(&mut self, key: Key) -> Result<(), SendError<AppEvent>> {
        self.motions
            .handle_key_press(&mut self.buffer, key, &self.event_sender)?;

        Ok(())
    }

    //     fn perform(&mut self, _app_event: &AppEvent) {
    //         todo!();
    //     }
}
