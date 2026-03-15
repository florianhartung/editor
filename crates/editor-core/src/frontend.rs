//! The definition of the interface a frontend must provide to the app.

use tokio::sync::mpsc::UnboundedReceiver;

use crate::{EventSender, communication::Action};

pub trait Frontend {
    // TODO check if this should really be an async fn. This might break for GUIs where the main thread must be used for windowing.
    #[allow(async_fn_in_trait)]
    async fn run(send_event: EventSender, receive_action: UnboundedReceiver<Action>); // -> Option<JoinHandle<()>>;
}
