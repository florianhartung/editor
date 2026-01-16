use core::communication::{Action, Key};

use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use crate::Buffer;

mod non_modal;
pub use non_modal::NonModalMotions;

mod helix;
pub use helix::HelixMotions;

// TODO allow motions to have per-buffer data and shared data
pub trait Motions: std::fmt::Debug + Send + Sync {
    fn handle_key_press(
        &mut self,
        editor: &mut Buffer,
        key: Key,
        send_action: &UnboundedSender<Action>,
    ) -> Result<(), SendError<Action>>;
}
