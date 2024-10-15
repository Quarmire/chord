mod chord;

use std::f64::consts::PI;

use chord::{Chord, ChordError, MAX_ID};

use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind}, layout::{Constraint, Layout, Position, Rect}, style::{Color, Modifier, Style, Stylize}, symbols::Marker, text::{Line, Text}, widgets::{canvas::Canvas, Block, Paragraph, Widget}, DefaultTerminal, Frame
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    /// Sorted map of nodes in chord ring
    chord: Chord,
    /// Lookup and deletion message
    result: String,
    marker: Marker,
}

enum InputMode {
    Normal,
    Searching,
    Deleting,
}

impl App {
    const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            result: String::new(),
            character_index: 0,
            chord: Chord::new(),
            marker: Marker::Dot,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_query(&mut self) {
        let key: u16 = match self.input.parse().ok() {
            Some(num) => {num}
            None => {0}
        };

        match self.chord.search(key) {
            Ok(node) => {
                self.result = format!("Key {} is located at node: {}", key, node.id);
            }
            Err(e) => {
                match e {
                    ChordError::OutOfRange => {
                        self.result = format!("Key {} is out of range of the chord ring.", key);
                    }
                    _ => { panic!("Unhandled error in submit_query: {:?}", e); }
                }
            }
        };

        self.input.clear();
        self.reset_cursor();
    }

    fn submit_deletion(&mut self) {
        let node_id: u16 = match self.input.parse().ok() {
            Some(num) => {num}
            None => {0}
        };

        match self.chord.delete_node(node_id) {
            Ok(()) => {
                self.result = format!("Node {} deleted", node_id);
            }
            Err(e) => {
                match e {
                    ChordError::NodeDoesNotExist => {
                        self.result = format!("Node {} does not exist.", node_id);
                    }
                    _ => { panic!("Unhandled error in submit_query: {:?}", e); }
                }
            }
        };

        self.input.clear();
        self.reset_cursor();
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.chord.add_node().unwrap(); // Add a random node as a starting point

        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('s') => {
                            self.input_mode = InputMode::Searching;
                        }
                        KeyCode::Char('a') => {
                            self.result = match self.chord.add_node() {
                                Ok(node_id) => {
                                    format!("Node {} added", node_id)
                                }
                                Err(e) => {
                                    format!("Node add error: {:?}", e)
                                }
                            }
                        }
                        KeyCode::Char('d') => {
                            self.input_mode = InputMode::Deleting;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    InputMode::Searching if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => self.submit_query(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Esc => self.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Deleting if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => self.submit_deletion(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Esc => self.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Searching | InputMode::Deleting => {}
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let horzontal = Layout::horizontal([
            Constraint::Length(80),
            Constraint::Min(160),
        ]);

        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
        ]);
        let [txt_area, vis_area] = horzontal.areas(frame.area());
        let [help_area, input_area, message_area] = vertical.areas(txt_area);

        frame.render_widget(self.chord_canvas(vis_area), vis_area);

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "s".bold(),
                    " to lookup node, ".into(),
                    "a".bold(),
                    " to add node, ".into(),
                    "d".bold(),
                    " to delete node".into(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Searching => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to return, ".into(),
                    "Enter".bold(),
                    format!(" to lookup key (0-{})", MAX_ID-1).into(),
                ],
                Style::default(),
            ),
            InputMode::Deleting => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to return, ".into(),
                    "Enter".bold(),
                    format!(" to delete node (0-{})", MAX_ID-1).into(),
                ],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Searching => Style::default().fg(Color::Yellow),
                InputMode::Deleting => Style::default().fg(Color::Red),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[allow(clippy::cast_possible_truncation)]
            InputMode::Searching | InputMode::Deleting => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                input_area.x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }

        let message = Paragraph::new(self.result.clone()).block(Block::bordered().title("Result"));
        frame.render_widget(message, message_area);
    }

    fn chord_canvas(&self, area: Rect) -> impl Widget + '_ {
        // Calculate center of the available area dynamically
        let center_x = ((area.width / 2) as f64) + (area.width / 5) as f64;
        let center_y = (area.height / 2) as f64;
        let radius = (area.width.min(area.height) as f64 / 2.5).min(40.0);  // Adjust radius dynamically based on terminal size

        Canvas::default()
            .block(Block::bordered().title("Chord Ring"))
            .marker(self.marker)
            .paint(move |ctx| {
                // Draw the circle
                ctx.draw(&ratatui::widgets::canvas::Circle {
                    x: center_x,
                    y: center_y,
                    radius,
                    color: Color::Blue,
                });

                // Draw numbers around the circle
                let mut ring = self.chord.get_ring().clone();
                let num_pos = ring.len();
                for i in 0..num_pos {
                    let angle = 2.0 * PI * (i as f64) / num_pos as f64;
                    let x_offset = ((radius*1.1) * angle.cos()) + center_x;
                    let y_offset = ((radius*1.1) * angle.sin()) + center_y;

                    // Draw the number at the calculated position
                    ctx.print(x_offset, y_offset, format!("{}", ring.pop().unwrap()).green());
                }
            })
            .x_bounds([area.x as f64, (area.width) as f64])
            .y_bounds([area.y as f64, (area.height as f64) * 1.2])
    }
}