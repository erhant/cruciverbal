//! CrosswordCompiler XML format parser.
//!
//! This module handles parsing CrosswordCompiler XML format used by
//! providers like Simply Daily Puzzles and Daily Pop.

use crate::ProviderError;
use puz_parse::Puzzle;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::collections::HashMap;

/// Parse CrosswordCompiler XML content into a Puzzle.
///
/// Handles both `<crossword-compiler>` and `<crossword-compiler-applet>` root elements.
pub fn parse(xml_content: &str) -> Result<Puzzle, ProviderError> {
    // Try both root element variants
    let puzzle_data = if let Ok(cc) = from_str::<CrosswordCompiler>(xml_content) {
        cc.rectangular_puzzle
    } else if let Ok(cca) = from_str::<CrosswordCompilerApplet>(xml_content) {
        cca.rectangular_puzzle
    } else {
        return Err(ProviderError::Other(
            "Failed to parse CrosswordCompiler XML".to_string(),
        ));
    };

    let width = puzzle_data.crossword.grid.width;
    let height = puzzle_data.crossword.grid.height;

    if width == 0 || height == 0 {
        return Err(ProviderError::InvalidPuzzleData(
            "Puzzle has zero dimensions".to_string(),
        ));
    }

    // Build cell map
    let mut cell_map: HashMap<(u8, u8), &Cell> = HashMap::new();
    for cell in &puzzle_data.crossword.grid.cells {
        cell_map.insert((cell.x, cell.y), cell);
    }

    // Build grids
    let mut blank_grid = Vec::with_capacity(height as usize);
    let mut solution_grid = Vec::with_capacity(height as usize);

    for y in 1..=height {
        let mut blank_row = String::with_capacity(width as usize);
        let mut sol_row = String::with_capacity(width as usize);

        for x in 1..=width {
            if let Some(cell) = cell_map.get(&(x, y)) {
                if cell.cell_type.as_deref() == Some("block") {
                    blank_row.push('.');
                    sol_row.push('.');
                } else if let Some(sol) = &cell.solution {
                    blank_row.push('-');
                    sol_row.push(sol.chars().next().unwrap_or('-'));
                } else {
                    blank_row.push('-');
                    sol_row.push('-');
                }
            } else {
                blank_row.push('.');
                sol_row.push('.');
            }
        }

        blank_grid.push(blank_row);
        solution_grid.push(sol_row);
    }

    // Parse clues
    let mut all_clues: Vec<(u16, bool, String)> = Vec::new();

    for (idx, clue_list) in puzzle_data.crossword.clues.iter().enumerate() {
        let is_across = idx == 0; // First list is typically across
        for clue in &clue_list.clues {
            let clue_no: u16 = clue.number.parse().unwrap_or(0);
            let mut clue_text = clue.text.clone().unwrap_or_default();

            // Append format/enumeration if present
            if let Some(fmt) = &clue.format {
                clue_text = format!("{} ({})", clue_text, fmt);
            }

            all_clues.push((clue_no, is_across, clue_text));
        }
    }

    let mut across_clues = HashMap::new();
    let mut down_clues = HashMap::new();

    for (num, is_across, text) in all_clues {
        if is_across {
            across_clues.insert(num, text);
        } else {
            down_clues.insert(num, text);
        }
    }

    let title = puzzle_data.metadata.title.unwrap_or_default();
    let author = puzzle_data.metadata.creator.unwrap_or_default();
    let copyright = puzzle_data.metadata.copyright.unwrap_or_default();

    Ok(Puzzle {
        info: puz_parse::PuzzleInfo {
            title,
            height,
            width,
            author,
            copyright,
            notes: String::new(),
            version: "1.4".to_string(),
            is_scrambled: false,
        },
        grid: puz_parse::Grid {
            blank: blank_grid,
            solution: solution_grid,
        },
        clues: puz_parse::Clues {
            across: across_clues,
            down: down_clues,
        },
        extensions: puz_parse::Extensions {
            rebus: None,
            circles: None,
            given: None,
        },
    })
}

/// Extract XML content from JavaScript wrapper.
///
/// Handles format: `var CrosswordPuzzleData = "...xml content...";`
pub fn extract_xml_from_js(js_content: &str) -> Result<String, ProviderError> {
    let prefix = "var CrosswordPuzzleData = \"";
    let suffix = "\";";

    let start = js_content.find(prefix).ok_or_else(|| {
        ProviderError::Other("Could not find CrosswordPuzzleData in JS".to_string())
    })? + prefix.len();

    let rest = &js_content[start..];
    let end = rest.find(suffix).ok_or_else(|| {
        ProviderError::Other("Could not find end of CrosswordPuzzleData".to_string())
    })?;

    let encoded = &rest[..end];

    // Unescape the content (backslash escapes)
    let xml = encoded.replace('\\', "");

    Ok(xml)
}

// ============================================================================
// XML Structures
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename = "crossword-compiler")]
struct CrosswordCompiler {
    #[serde(rename = "rectangular-puzzle")]
    rectangular_puzzle: RectangularPuzzle,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "crossword-compiler-applet")]
struct CrosswordCompilerApplet {
    #[serde(rename = "rectangular-puzzle")]
    rectangular_puzzle: RectangularPuzzle,
}

#[derive(Debug, Deserialize)]
struct RectangularPuzzle {
    metadata: Metadata,
    crossword: CrosswordData,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    title: Option<String>,
    creator: Option<String>,
    copyright: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CrosswordData {
    grid: Grid,
    clues: Vec<ClueList>,
}

#[derive(Debug, Deserialize)]
struct Grid {
    #[serde(rename = "@width")]
    width: u8,
    #[serde(rename = "@height")]
    height: u8,
    #[serde(rename = "cell")]
    cells: Vec<Cell>,
}

#[derive(Debug, Deserialize)]
struct Cell {
    #[serde(rename = "@x")]
    x: u8,
    #[serde(rename = "@y")]
    y: u8,
    #[serde(rename = "@solution")]
    solution: Option<String>,
    #[serde(rename = "@type")]
    cell_type: Option<String>,
    // Note: @background-shape exists in some puzzles but is currently unused
    #[serde(rename = "@background-shape")]
    #[allow(dead_code)]
    background_shape: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClueList {
    #[serde(rename = "clue")]
    clues: Vec<ClueItem>,
}

#[derive(Debug, Deserialize)]
struct ClueItem {
    #[serde(rename = "@number")]
    number: String,
    #[serde(rename = "@format")]
    format: Option<String>,
    #[serde(rename = "$text")]
    text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_xml_from_js() {
        let js = r#"var CrosswordPuzzleData = "<puzzle>test</puzzle>";"#;
        let xml = extract_xml_from_js(js).unwrap();
        assert_eq!(xml, "<puzzle>test</puzzle>");
    }

    #[test]
    fn test_extract_xml_with_escapes() {
        let js = r#"var CrosswordPuzzleData = "<puzzle attr=\"value\">test</puzzle>";"#;
        let xml = extract_xml_from_js(js).unwrap();
        assert_eq!(xml, "<puzzle attr=\"value\">test</puzzle>");
    }
}
