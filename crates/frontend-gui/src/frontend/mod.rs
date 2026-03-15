use iced::{
    Element, Subscription, Task,
    widget::text_editor::{self, Edit},
};
use iced_futures::MaybeSend;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

use editor_core::communication::{Action, Event};

mod app;

pub struct Frontend {
    app: app::App,
    send_event: UnboundedSender<Event>,
}

impl core::frontend::Frontend for Frontend {
    fn run(send_event: UnboundedSender<Event>, receive_action: UnboundedReceiver<Action>) {
        let frontend = Self {
            app: app::App::new(),
            send_event,
        };

        let receive_action_task =
            Task::stream(UnboundedReceiverStream::new(receive_action)).map(Message::FromCore);

        iced::application("[WIP] Just another text editor", Self::update, Self::view)
            .subscription(Self::subscription)
            .executor::<TokioProvidedExecutor>()
            .run_with(|| (frontend, receive_action_task))
            .unwrap();
    }
}

#[derive(Debug, Clone)]
enum Message {
    /// An [`Action`] was sent by the core to the GUI.
    FromCore(Action),

    /// The GUI emitted a message
    FromGui(app::Message),
    // Caused by user
}

impl Frontend {
    fn subscription(&self) -> Subscription<Message> {
        self.app.subscription().map(Message::FromGui)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FromCore(action) => match action {
                Action::Insert(c) => {
                    let message_for_gui =
                        app::Message::ToEditorAction(text_editor::Action::Edit(Edit::Insert(c)));
                    self.app
                        .update(message_for_gui, &self.send_event)
                        .map(Message::FromGui)
                }
            },
            Message::FromGui(gui_message) => self
                .app
                .update(gui_message, &self.send_event)
                .map(Message::FromGui),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        self.app.view().map(Message::FromGui)
    }
}

/// A custom executor for use by the Iced runtime that uses the existing Tokio runtime.
///
/// All executors defined by Iced initialize an async runtime by themselves.
/// However, for this application, we want Iced to use an already initialized Tokio runtime.
struct TokioProvidedExecutor(tokio_executor::DefaultExecutor);

impl iced::executor::Executor for TokioProvidedExecutor {
    fn new() -> Result<Self, iced::futures::io::Error>
    where
        Self: Sized,
    {
        Ok(Self(tokio_executor::DefaultExecutor::current()))
    }

    fn spawn(&self, future: impl Future<Output = ()> + MaybeSend + 'static) {
        tokio::runtime::Handle::try_current()
            .expect("tokio runtime to be provided")
            .spawn(future);
    }

    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        let _guard = tokio::runtime::Handle::try_current()
            .expect("tokio runtime to be provided")
            .enter();
        f()
    }
}
