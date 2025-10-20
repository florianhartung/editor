use core::communication::{Action, EditorEvent, Key};
use std::{borrow::Cow, io, time::Duration};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::Alignment,
    text::{Line, Text},
    widgets::{Block, Widget},
};
use ropey::Rope;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct Frontend {}

impl core::frontend::Frontend for Frontend {
    fn run(
        send_event: UnboundedSender<core::communication::Event>,
        receive_action: UnboundedReceiver<core::communication::Action>,
    ) {
        send_event
            .send(core::communication::Event::OpenFile {
                path: "/home/flo/test.md".to_owned(),
            })
            .unwrap();

        let mut terminal = ratatui::init();
        let mut app = App {
            exit: false,
            title: None,
            buffer_lines: Vec::new(),
            receive_action,
            send_event,
        };

        let app_result = app.run(&mut terminal);

        ratatui::restore();
        app_result.unwrap();
    }
}

struct App {
    exit: bool,
    /// None if unnamed
    title: Option<String>,
    buffer_lines: Vec<Line<'static>>,
    receive_action: UnboundedReceiver<core::communication::Action>,
    send_event: UnboundedSender<core::communication::Event>,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        while let Ok(action) = self.receive_action.try_recv() {
            match action {
                Action::SetBufferContents(contents) => {
                    self.buffer_lines = rope_to_lines(contents);
                }
                Action::SetBuffer { contents, title } => {
                    self.buffer_lines = rope_to_lines(contents);
                    self.title = Some(title);
                }
                Action::HintBufferLoading => self.title = Some("Loading...".to_owned()),
            }
        }

        if event::poll(Duration::ZERO)? {
            let event = event::read()?;
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => {
                    self.send_event
                        .send(core::communication::Event::Editor(EditorEvent::KeyPress(
                            Key::Char(c),
                        )))
                        .unwrap();
                }
                _ => {
                    self.exit = true;
                }
            }
        }

        Ok(())
    }
}

fn rope_to_lines(rope: Rope) -> Vec<Line<'static>> {
    rope.lines()
        .map(|line| {
            let line = line.chars().collect::<String>();
            Line::raw(line)
        })
        .collect()
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let x = Text::from(self.buffer_lines.clone());

        let title = self
            .title
            .clone()
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed("Unnamed"));

        let my_block = Block::bordered()
            .title(title)
            .title_alignment(Alignment::Center);

        let block_inner = my_block.inner(area);

        x.render(block_inner, buf);
        my_block.render(area, buf);

        // Paragraph::new("hello world")
        //     .centered()
        //     .block(my_block)
        //     .render(area, buf);
    }
}
