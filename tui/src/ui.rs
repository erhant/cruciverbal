use cruciverbal_core::{Crossword, Direction, Cell};
use ratatui::{
    layout::{Constraint, Direction as LayoutDirection, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub struct UI {
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub current_direction: Direction,
}

impl UI {
    pub fn new() -> Self {
        Self {
            cursor_row: 0,
            cursor_col: 0,
            current_direction: Direction::Across,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, crossword: &Crossword) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Min(30),   // Grid area
                Constraint::Length(40), // Clues area
            ])
            .split(frame.area());

        self.render_grid(frame, chunks[0], crossword);
        self.render_clues(frame, chunks[1], crossword);
    }

    fn render_grid(&self, frame: &mut Frame, area: Rect, crossword: &Crossword) {
        let grid = &crossword.grid;
        let cell_width = 3;
        let cell_height = 3;
        
        let grid_width = (grid.width * cell_width) as u16;
        let grid_height = (grid.height * cell_height) as u16;
        
        // Center the grid in the available area
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
                            Color::Reset
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
                        
                        // Middle line: letter in center
                        let middle_line = if let Cell::Letter { user_input: Some(c), .. } = cell {
                            format!(" {} ", c)
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

        let grid_paragraph = Paragraph::new(Text::from(grid_lines))
            .block(Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} - {} ", crossword.title, crossword.author)));

        frame.render_widget(grid_paragraph, grid_area);

        // Render status info
        let status_area = Rect {
            x: area.x,
            y: area.y + area.height - 3,
            width: area.width,
            height: 3,
        };

        let current_clue = crossword.get_clue_at_position(
            self.cursor_row, 
            self.cursor_col, 
            self.current_direction
        );

        let status_text = if let Some(clue) = current_clue {
            format!("{}{}: {}", 
                clue.number, 
                match clue.direction {
                    Direction::Across => "A",
                    Direction::Down => "D",
                },
                clue.text
            )
        } else {
            "Move cursor to a letter cell to see clue".to_string()
        };

        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title(" Current Clue "))
            .wrap(Wrap { trim: true });

        frame.render_widget(status, status_area);
    }

    fn render_clues(&self, frame: &mut Frame, area: Rect, crossword: &Crossword) {
        let clues_chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Across clues
        let across_clues: Vec<ListItem> = crossword
            .get_clues_by_direction(Direction::Across)
            .iter()
            .map(|clue| {
                ListItem::new(format!("{}. {}", clue.number, clue.text))
            })
            .collect();

        let across_list = List::new(across_clues)
            .block(Block::default().borders(Borders::ALL).title(" Across "));

        frame.render_widget(across_list, clues_chunks[0]);

        // Down clues
        let down_clues: Vec<ListItem> = crossword
            .get_clues_by_direction(Direction::Down)
            .iter()
            .map(|clue| {
                ListItem::new(format!("{}. {}", clue.number, clue.text))
            })
            .collect();

        let down_list = List::new(down_clues)
            .block(Block::default().borders(Borders::ALL).title(" Down "));

        frame.render_widget(down_list, clues_chunks[1]);
    }


    pub fn find_next_empty_cell(&mut self, crossword: &Crossword) {
        if let Some(clue) = crossword.get_clue_at_position(
            self.cursor_row, 
            self.cursor_col, 
            self.current_direction
        ) {
            for (row, col) in clue.positions() {
                if let Some(Cell::Letter { user_input: None, .. }) = crossword.grid.get_cell(row, col) {
                    self.cursor_row = row;
                    self.cursor_col = col;
                    return;
                }
            }
        }
    }

    pub fn toggle_direction(&mut self) {
        self.current_direction = match self.current_direction {
            Direction::Across => Direction::Down,
            Direction::Down => Direction::Across,
        };
    }
}