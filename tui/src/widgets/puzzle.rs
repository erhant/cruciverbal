use puz_parse::Puzzle;
use ratatui::widgets::Widget;

struct PuzzleWidget {
    puzzle: Puzzle,
}

impl PuzzleWidget {
    pub fn new(puzzle: Puzzle) -> Self {
        Self { puzzle }
    }
}

impl Widget for PuzzleWidget {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        // self.puzzle.grid.
        todo!()
    }
}
