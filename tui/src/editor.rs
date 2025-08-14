use cruciverbal_core::{Crossword, Direction, Cell, Clue};
use ratatui::{
    layout::{Constraint, Direction as LayoutDirection, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum EditorMode {
    GridEdit,
    ClueEdit,
}

pub struct Editor {
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub mode: EditorMode,
    pub input_buffer: String,
    pub selected_clue: Option<usize>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            cursor_row: 0,
            cursor_col: 0,
            mode: EditorMode::GridEdit,
            input_buffer: String::new(),
            selected_clue: None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, crossword: &Crossword) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Min(30),   // Grid area
                Constraint::Length(40), // Controls area
            ])
            .split(frame.area());

        self.render_grid(frame, chunks[0], crossword);
        self.render_controls(frame, chunks[1], crossword);
    }

    fn render_grid(&self, frame: &mut Frame, area: Rect, crossword: &Crossword) {
        let grid = &crossword.grid;
        let cell_width = 3;
        let cell_height = 3;
        
        let grid_width = (grid.width * cell_width) as u16;
        let grid_height = (grid.height * cell_height) as u16;
        
        let grid_area = Rect {
            x: area.x + (area.width.saturating_sub(grid_width)) / 2,
            y: area.y + (area.height.saturating_sub(grid_height)) / 2,
            width: grid_width.min(area.width),
            height: grid_height.min(area.height),
        };

        // Create 3x3 representation of each cell
        let mut grid_lines = Vec::new();
        
        for (row_idx, row) in grid.cells.iter().enumerate() {
            // Each grid row becomes 3 display lines
            let mut line1_spans = Vec::new(); // Top line (with numbers)
            let mut line2_spans = Vec::new(); // Middle line (with letter)
            let mut line3_spans = Vec::new(); // Bottom line
            
            for (col_idx, cell) in row.iter().enumerate() {
                let is_cursor = row_idx == self.cursor_row && col_idx == self.cursor_col;
                
                match cell {
                    Cell::Blocked => {
                        // Blocked cells are solid white 3x3 blocks
                        let style = Style::default().bg(Color::White);
                        line1_spans.push(Span::styled("███", style));
                        line2_spans.push(Span::styled("███", style));
                        line3_spans.push(Span::styled("███", style));
                    }
                    Cell::Letter { .. } | Cell::Empty => {
                        let bg_color = if is_cursor {
                            Color::Yellow
                        } else if cell.is_letter() {
                            Color::White
                        } else {
                            Color::DarkGray
                        };
                        
                        let fg_color = if is_cursor { Color::Black } else { Color::Black };
                        let style = Style::default().bg(bg_color).fg(fg_color);
                        let empty_style = Style::default().bg(bg_color).fg(fg_color);
                        
                        // Top line: number in top-left, rest empty
                        let top_line = if let Some(n) = cell.get_number() {
                            if n < 10 {
                                format!("{}  ", n)
                            } else {
                                format!("{} ", n)
                            }
                        } else {
                            "   ".to_string()
                        };
                        line1_spans.push(Span::styled(top_line, style));
                        
                        // Middle line: letter in center (show correct letter in editor)
                        let middle_line = if let Cell::Letter { correct, .. } = cell {
                            format!(" {} ", correct)
                        } else {
                            "   ".to_string()
                        };
                        line2_spans.push(Span::styled(middle_line, style));
                        
                        // Bottom line: empty
                        line3_spans.push(Span::styled("   ", empty_style));
                    }
                }
            }
            
            // Add all three lines for this row
            grid_lines.push(Line::from(line1_spans));
            grid_lines.push(Line::from(line2_spans));
            grid_lines.push(Line::from(line3_spans));
        }

        let title = format!(" Editor: {} - {} ", crossword.title, crossword.author);
        let grid_paragraph = Paragraph::new(Text::from(grid_lines))
            .block(Block::default().borders(Borders::ALL).title(title));

        frame.render_widget(grid_paragraph, grid_area);
    }

    fn render_controls(&self, frame: &mut Frame, area: Rect, crossword: &Crossword) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(5),  // Mode info
                Constraint::Length(8),  // Controls
                Constraint::Min(10),    // Clues list
            ])
            .split(area);

        // Mode info
        let mode_text = match self.mode {
            EditorMode::GridEdit => "Grid Edit Mode\nPress SPACE to toggle cell type\nPress C to add clue\nPress TAB to switch to clue edit",
            EditorMode::ClueEdit => "Clue Edit Mode\nEdit clues for current puzzle\nPress TAB to return to grid edit",
        };
        
        let mode_paragraph = Paragraph::new(mode_text)
            .block(Block::default().borders(Borders::ALL).title(" Mode "))
            .wrap(Wrap { trim: true });
        
        frame.render_widget(mode_paragraph, chunks[0]);

        // Controls
        let controls_text = "Controls:\n↑↓←→ Move cursor\nSPACE Toggle cell\nC Add clue\nS Save\nQ Quit\nTAB Switch mode";
        
        let controls_paragraph = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::ALL).title(" Controls "));
        
        frame.render_widget(controls_paragraph, chunks[1]);

        // Clues list
        let mut clue_items = Vec::new();
        for (i, clue) in crossword.clues.iter().enumerate() {
            let direction_str = match clue.direction {
                Direction::Across => "A",
                Direction::Down => "D",
            };
            
            let style = if Some(i) == self.selected_clue {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            
            clue_items.push(
                ListItem::new(format!("{}{}. {} ({})", clue.number, direction_str, clue.text, clue.answer))
                    .style(style)
            );
        }
        
        let clues_list = List::new(clue_items)
            .block(Block::default().borders(Borders::ALL).title(" Clues "));
        
        frame.render_widget(clues_list, chunks[2]);
    }

    pub fn move_cursor(&mut self, crossword: &Crossword, dr: i32, dc: i32) {
        let new_row = (self.cursor_row as i32 + dr).max(0) as usize;
        let new_col = (self.cursor_col as i32 + dc).max(0) as usize;

        if new_row < crossword.grid.height && new_col < crossword.grid.width {
            self.cursor_row = new_row;
            self.cursor_col = new_col;
        }
    }

    pub fn toggle_cell(&mut self, crossword: &mut Crossword) -> Result<(), String> {
        let current_cell = crossword.grid.get_cell(self.cursor_row, self.cursor_col);
        
        match current_cell {
            Some(Cell::Empty) => {
                crossword.grid.set_cell(self.cursor_row, self.cursor_col, Cell::Blocked)?;
            }
            Some(Cell::Blocked) => {
                crossword.grid.set_cell(self.cursor_row, self.cursor_col, Cell::Empty)?;
            }
            Some(Cell::Letter { .. }) => {
                crossword.grid.set_cell(self.cursor_row, self.cursor_col, Cell::Empty)?;
                // Remove clues that reference this position
                crossword.clues.retain(|clue| {
                    !clue.positions().contains(&(self.cursor_row, self.cursor_col))
                });
            }
            None => {}
        }
        
        Ok(())
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            EditorMode::GridEdit => EditorMode::ClueEdit,
            EditorMode::ClueEdit => EditorMode::GridEdit,
        };
    }

    pub fn start_clue_creation(&mut self) -> bool {
        matches!(self.mode, EditorMode::GridEdit)
    }

    pub fn create_clue(&mut self, crossword: &mut Crossword, direction: Direction, text: String, answer: String) -> Result<(), String> {
        // Find next available clue number
        let clue_number = crossword.clues.iter()
            .map(|c| c.number)
            .max()
            .unwrap_or(0) + 1;

        let clue = Clue::new(
            clue_number,
            direction,
            text,
            answer,
            self.cursor_row,
            self.cursor_col,
        );

        crossword.add_clue(clue)?;
        Ok(())
    }
}