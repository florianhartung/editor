use std::io;

use crossterm::event::{EventStream, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Alignment, Rect},
    text::{Line, Text},
    widgets::{Block, Widget},
};
use ropey::Rope;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};

use editor_core::{
    EventSender,
    communication::{Action, Key, KeyNamed},
};

type TerminalEvent = crossterm::event::Event;

pub struct Frontend {}

impl editor_core::frontend::Frontend for Frontend {
    async fn run(event_sender: EventSender, receive_action: UnboundedReceiver<Action>) {
        let mut terminal = ratatui::init();
        let mut app = App {
            exit: false,
            receive_action: UnboundedReceiverStream::new(receive_action),
            event_sender,
            terminal_event_stream: EventStream::new(),

            buffer: None,
            cursor_block_for_current_frame: None,
        };

        let app_result = app.run(&mut terminal).await;
        ratatui::restore();
        app_result.unwrap();
    }
}

struct App {
    exit: bool,
    receive_action: UnboundedReceiverStream<Action>,
    event_sender: EventSender,
    terminal_event_stream: crossterm::event::EventStream,

    /// None if loading
    buffer: Option<Buffer>,

    cursor_block_for_current_frame: Option<Rect>,
}

impl App {
    async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            // terminal
            //     .backend_mut()
            //     .execute(crossterm::cursor::SetCursorStyle::DefaultUserShape)?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(&mut *self, frame.area());

        // Render the cursor. This cannot be done through widgets.
        if let Some(buffer) = &self.buffer {
            let editor_rect = self
                .cursor_block_for_current_frame
                .take()
                .expect("this to be set to something during rendering");

            // TODO this is not very clean
            // Offset the cursor by the position of the editor rect
            let cursor_position = (
                buffer.cursor.0 + editor_rect.x,
                buffer.cursor.1 + editor_rect.y,
            );
            if editor_rect.contains(cursor_position.into()) {
                frame.set_cursor_position(cursor_position);
            }
        }
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        // TODO merge streams, because this is basically a `loop { select! { ...} }`
        tokio::select! {
            biased; // fairness does not matter, so we might as well do this for reproducibility
            Some(event) = self.terminal_event_stream.next() => self.handle_terminal_event(event?).await,
            Some(action) = self.receive_action.next() => self.handle_action(action),
        }

        Ok(())
    }

    async fn handle_terminal_event(&mut self, event: TerminalEvent) {
        match event {
            TerminalEvent::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                self.exit = true;
            }
            TerminalEvent::Key(KeyEvent { code, .. }) => {
                let key_pressed = match code {
                    KeyCode::Char(c) => Key::Char(c),
                    KeyCode::Backspace => Key::Named(KeyNamed::Backspace),
                    KeyCode::Esc => Key::Named(KeyNamed::Escape),
                    KeyCode::Enter => Key::Named(KeyNamed::Enter),
                    KeyCode::Tab => Key::Named(KeyNamed::Tab),
                    KeyCode::Up => Key::Named(KeyNamed::Up),
                    KeyCode::Down => Key::Named(KeyNamed::Down),
                    KeyCode::Left => Key::Named(KeyNamed::Left),
                    KeyCode::Right => Key::Named(KeyNamed::Right),
                    _ => return,
                };

                self.event_sender.key_press(key_pressed);
            }
            _ => {}
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::SetBufferContents(contents) => {
                self.buffer.get_or_insert_default().contents = rope_to_lines(contents);
            }
            Action::SetBuffer { contents, title } => {
                self.buffer = Some(Buffer {
                    contents: rope_to_lines(contents),
                    filename: Some(title),
                    cursor: (0, 0),
                });
            }
            Action::HintBufferLoading => self.buffer = None,
            Action::SetCursorPosition { line, column } => {
                if let Some(buffer) = &mut self.buffer {
                    buffer.cursor = (u16::try_from(column).unwrap(), u16::try_from(line).unwrap());
                }
            }
            Action::Batched(actions) => {
                for action in actions {
                    self.handle_action(action);
                }
            }
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

impl Widget for &mut App {
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
        self.cursor_block_for_current_frame = Some(block_inner);
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
    cursor: (u16, u16),
}
