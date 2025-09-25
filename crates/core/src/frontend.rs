//! The definition of the interface a frontend must provide to the app.

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::communication::{Action, Event};

pub trait Frontend {
    fn run(send_event: UnboundedSender<Event>, receive_action: UnboundedReceiver<Action>);
}
