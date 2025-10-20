use core::communication::{Action, EditorEvent, Key};
use std::io;

use crossterm::event::EventStream;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{Event, KeyCode, KeyEvent},
    layout::Alignment,
    text::{Line, Text},
    widgets::{Block, Widget},
};
use ropey::Rope;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};

pub struct Frontend {}

impl core::frontend::Frontend for Frontend {
    async fn run(
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
            receive_action: UnboundedReceiverStream::new(receive_action),
            send_event,
            terminal_event_stream: EventStream::new(),

            buffer: None,
        };

        let app_result = app.run(&mut terminal).await;
        ratatui::restore();
        app_result.unwrap();
    }
}

struct App {
    exit: bool,
    receive_action: UnboundedReceiverStream<core::communication::Action>,
    send_event: UnboundedSender<core::communication::Event>,
    terminal_event_stream: crossterm::event::EventStream,

    /// None if loading
    buffer: Option<Buffer>,
}

impl App {
    async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        // TODO merge streams, because this is basically a `loop { select! { ...} }`
        tokio::select! {
            biased; // for reproducibility, fairness does not matter
            Some(event) = self.terminal_event_stream.next() => self.handle_terminal_event(event?).await,
            Some(action) = self.receive_action.next() => self.handle_action(action).await,
        }

        Ok(())
    }

    async fn handle_terminal_event(&mut self, event: Event) {
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

    async fn handle_action(&mut self, action: Action) {
        match action {
            Action::SetBufferContents(contents) => {
                self.buffer.get_or_insert_default().contents = rope_to_lines(contents);
            }
            Action::SetBuffer { contents, title } => {
                self.buffer = Some(Buffer {
                    contents: rope_to_lines(contents),
                    filename: Some(title),
                });
            }
            Action::HintBufferLoading => self.buffer = None,
        }
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
        let (title, text) = match &self.buffer {
            Some(buffer) => {
                let text = Text::from(buffer.contents.clone());

                let title = buffer.filename.as_deref().unwrap_or("Unnamed");
                (title, text)
            }
            None => ("Loading...", Text::raw("Loading...")),
        };

        let my_block = Block::bordered()
            .title(title)
            .title_alignment(Alignment::Center);

        let block_inner = my_block.inner(area);

        text.render(block_inner, buf);
        my_block.render(area, buf);

        // Paragraph::new("hello world")
        //     .centered()
        //     .block(my_block)
        //     .render(area, buf);
    }
}

#[derive(Default)]
struct Buffer {
    /// None if unnamed
    filename: Option<String>,
    contents: Vec<Line<'static>>,
}
