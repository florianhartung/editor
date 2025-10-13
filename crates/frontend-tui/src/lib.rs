use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct Frontend {}

impl core::frontend::Frontend for Frontend {
    fn run(
        _send_event: UnboundedSender<core::communication::Event>,
        _receive_action: UnboundedReceiver<core::communication::Action>,
    ) {
        start().unwrap();
    }
}

fn start() -> Result<()> {
    color_eyre::install().unwrap();
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            return Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("hello world", frame.area());
}
