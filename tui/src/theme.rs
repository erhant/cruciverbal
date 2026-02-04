//! Theme system for Cruciverbal.
//!
//! Provides preset color schemes that can be selected by the user.

use ratatui::style::Color;

/// A color theme for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Unique identifier for the theme.
    pub id: &'static str,
    /// Display name for the theme.
    pub name: &'static str,

    // Semantic colors
    /// Primary color for selected items, cursor, action keys.
    pub primary: Color,
    /// Secondary color for titles, word highlight.
    pub secondary: Color,
    /// Normal text content.
    pub text: Color,
    /// Dimmed text for descriptions, inactive items.
    pub dimmed: Color,
    /// Success indicators (completion, save notification).
    pub success: Color,
    /// Error indicators.
    pub error: Color,

    // Grid colors
    /// Box-drawing characters for grid borders.
    pub grid_border: Color,
    /// Background color for filled (black) cells.
    pub filled_cell_bg: Color,
    /// Foreground color for filled cell rendering.
    pub filled_cell_fg: Color,
}

/// Default theme - the original Cruciverbal colors.
pub const DEFAULT: Theme = Theme {
    id: "default",
    name: "Default",
    primary: Color::Yellow,
    secondary: Color::Cyan,
    text: Color::White,
    dimmed: Color::DarkGray,
    success: Color::Green,
    error: Color::Red,
    grid_border: Color::White,
    filled_cell_bg: Color::Black,
    filled_cell_fg: Color::White,
};

/// Dark theme - warm gold and cool blue for high contrast.
pub const DARK: Theme = Theme {
    id: "dark",
    name: "Dark",
    primary: Color::Rgb(255, 215, 0),    // Gold
    secondary: Color::Rgb(100, 149, 237), // Cornflower blue
    text: Color::Rgb(220, 220, 220),      // Light gray
    dimmed: Color::Rgb(128, 128, 128),    // Gray
    success: Color::Rgb(50, 205, 50),     // Lime green
    error: Color::Rgb(255, 99, 71),       // Tomato
    grid_border: Color::Rgb(192, 192, 192), // Silver
    filled_cell_bg: Color::Rgb(32, 32, 32), // Dark gray
    filled_cell_fg: Color::Rgb(64, 64, 64), // Darker gray
};

/// Light theme - darker tones for light terminal backgrounds.
pub const LIGHT: Theme = Theme {
    id: "light",
    name: "Light",
    primary: Color::Rgb(184, 134, 11),   // Dark goldenrod
    secondary: Color::Rgb(0, 139, 139),  // Dark cyan
    text: Color::Rgb(33, 33, 33),        // Near black
    dimmed: Color::Rgb(105, 105, 105),   // Dim gray
    success: Color::Rgb(34, 139, 34),    // Forest green
    error: Color::Rgb(178, 34, 34),      // Firebrick
    grid_border: Color::Rgb(64, 64, 64), // Dark gray
    filled_cell_bg: Color::Rgb(48, 48, 48), // Charcoal
    filled_cell_fg: Color::Rgb(96, 96, 96), // Gray
};

/// Ocean theme - sandy gold and ocean blue palette.
pub const OCEAN: Theme = Theme {
    id: "ocean",
    name: "Ocean",
    primary: Color::Rgb(244, 208, 111),  // Sandy gold
    secondary: Color::Rgb(70, 130, 180), // Steel blue
    text: Color::Rgb(240, 248, 255),     // Alice blue
    dimmed: Color::Rgb(119, 136, 153),   // Light slate gray
    success: Color::Rgb(32, 178, 170),   // Light sea green
    error: Color::Rgb(205, 92, 92),      // Indian red
    grid_border: Color::Rgb(176, 196, 222), // Light steel blue
    filled_cell_bg: Color::Rgb(25, 25, 112), // Midnight blue
    filled_cell_fg: Color::Rgb(65, 105, 225), // Royal blue
};

/// Forest theme - sunlight gold and leaf green palette.
pub const FOREST: Theme = Theme {
    id: "forest",
    name: "Forest",
    primary: Color::Rgb(255, 223, 128),  // Soft gold (sunlight)
    secondary: Color::Rgb(107, 142, 35), // Olive drab (leaves)
    text: Color::Rgb(245, 245, 220),     // Beige
    dimmed: Color::Rgb(143, 143, 123),   // Dark khaki-ish
    success: Color::Rgb(60, 179, 113),   // Medium sea green
    error: Color::Rgb(210, 105, 30),     // Chocolate
    grid_border: Color::Rgb(189, 183, 107), // Dark khaki
    filled_cell_bg: Color::Rgb(34, 49, 34), // Very dark green
    filled_cell_fg: Color::Rgb(85, 107, 47), // Dark olive green
};

impl Theme {
    /// All available themes.
    pub const ALL: [Theme; 5] = [DEFAULT, DARK, LIGHT, OCEAN, FOREST];

    /// Look up a theme by its ID.
    ///
    /// Returns the DEFAULT theme if the ID is not found.
    pub fn by_id(id: &str) -> &'static Theme {
        Theme::ALL
            .iter()
            .find(|t| t.id == id)
            .unwrap_or(&DEFAULT)
    }
}
