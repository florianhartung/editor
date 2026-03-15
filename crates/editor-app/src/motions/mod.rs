//! Sets of motions define how inputs from the user manipulate the cursor and
//! the buffer contents.
//!
//! Their interface is defined through the [`Motions`] trait, allowing different
//! sets of motions to co-exist.

use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use editor_core::communication::{AppEvent, Key};

use crate::Buffer;

#[expect(unused, reason = "no support for switching between motions yet")]
mod non_modal;
#[expect(unused, reason = "no support for switching between motions yet")]
pub use non_modal::NonModalMotions;

mod helix;
pub use helix::HelixMotions;

// TODO allow motions to have per-buffer data and shared data
pub trait Motions: std::fmt::Debug + Send + Sync {
    fn handle_key_press(
        &mut self,
        editor: &mut Buffer,
        key: Key,
        send_app_event: &UnboundedSender<AppEvent>,
    ) -> Result<(), SendError<AppEvent>>;
}
