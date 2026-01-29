use crate::ProviderError;
use puz_parse::Puzzle;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::collections::HashMap;

/// Simply Daily Puzzles variants
#[derive(Debug, Clone, Copy)]
pub enum SimplyDailyVariant {
    /// Regular daily crossword
    Regular,
    /// Cryptic crossword
    Cryptic,
    /// Quick crossword
    Quick,
}

impl SimplyDailyVariant {
    fn url_subdir(&self) -> &'static str {
        match self {
            SimplyDailyVariant::Regular => "daily-crossword",
            SimplyDailyVariant::Cryptic => "daily-cryptic",
            SimplyDailyVariant::Quick => "daily-quick-crossword",
        }
    }

    fn qs_prefix(&self) -> &'static str {
        match self {
            SimplyDailyVariant::Regular => "dc1",
            SimplyDailyVariant::Cryptic => "dc1",
            SimplyDailyVariant::Quick => "dq1",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SimplyDailyVariant::Regular => "Simply Daily Puzzles",
            SimplyDailyVariant::Cryptic => "Simply Daily Cryptic",
            SimplyDailyVariant::Quick => "Simply Daily Quick",
        }
    }
}

/// Download a Simply Daily puzzle for the given date.
///
/// ## Arguments
/// - `variant` - Which puzzle variant to download
/// - `date` - Date string in "yyyy-mm-dd" format
pub async fn download(variant: SimplyDailyVariant, date: &str) -> Result<Puzzle, ProviderError> {
    // Parse date to get year-month for path
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return Err(ProviderError::Other(format!(
            "Invalid date format: {}. Expected yyyy-mm-dd",
            date
        )));
    }

    let year_month = format!("{}-{}", parts[0], parts[1]);
    let prefix = variant.qs_prefix();
    let subdir = variant.url_subdir();

    let url = format!(
        "https://simplydailypuzzles.com/{}/puzzles/{}/{}-{}.js",
        subdir, year_month, prefix, date
    );

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("User-Agent", "cruciverbal/0.1")
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(ProviderError::Other(format!(
            "Failed to fetch puzzle: {} (status: {})",
            url,
            res.status()
        )));
    }

    let js_content = res.text().await?;
    let xml_content = extract_xml_from_js(&js_content)?;
    parse(&xml_content)
}

/// Download the latest Simply Daily puzzle.
pub async fn download_latest(variant: SimplyDailyVariant) -> Result<Puzzle, ProviderError> {
    use chrono::Local;

    let today = Local::now().date_naive();
    let date_str = today.format("%Y-%m-%d").to_string();
    download(variant, &date_str).await
}

fn extract_xml_from_js(js_content: &str) -> Result<String, ProviderError> {
    // JS format: var CrosswordPuzzleData = "...xml content...";
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
    let xml = encoded.replace("\\", "");

    Ok(xml)
}

// CrosswordCompiler XML structures
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
    #[serde(rename = "@background-shape")]
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

fn parse(xml_content: &str) -> Result<Puzzle, ProviderError> {
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

    // Parse clues - combine both clue lists and sort by number
    let mut all_clues: Vec<(u16, bool, String)> = Vec::new(); // (number, is_across, text)

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download_regular() {
        match download(SimplyDailyVariant::Regular, "2025-01-28").await {
            Ok(puzzle) => {
                println!("Downloaded: {}", puzzle.info.title);
                println!("Author: {}", puzzle.info.author);
                println!("Size: {}x{}", puzzle.info.width, puzzle.info.height);
                assert!(puzzle.info.width > 0);
                assert!(puzzle.info.height > 0);
            }
            Err(e) => panic!("Download failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_download_cryptic() {
        match download(SimplyDailyVariant::Cryptic, "2025-01-28").await {
            Ok(puzzle) => {
                println!("Downloaded: {}", puzzle.info.title);
                assert!(puzzle.info.width > 0);
            }
            Err(e) => panic!("Download failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_download_quick() {
        match download(SimplyDailyVariant::Quick, "2025-01-28").await {
            Ok(puzzle) => {
                println!("Downloaded: {}", puzzle.info.title);
                assert!(puzzle.info.width > 0);
            }
            Err(e) => panic!("Download failed: {}", e),
        }
    }
}
