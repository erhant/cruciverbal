use crate::ProviderError;
use puz_parse::Puzzle;
use serde::Deserialize;
use std::collections::HashMap;

/// Download the Washington Post Sunday crossword for the given date.
///
/// ## Arguments
/// - `date` - Date string in "yyyy/mm/dd" format (with slashes)
///
/// Note: The WaPo API only keeps puzzles available for a limited time.
/// Older puzzles may not be accessible.
pub async fn download(date: &str) -> Result<Puzzle, ProviderError> {
    let url = format!(
        "https://games-service-prod.site.aws.wapo.pub/crossword/levels/sunday/{}",
        date
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

    // Check for empty response (puzzle not available)
    let text = res.text().await?;
    if text.is_empty() {
        return Err(ProviderError::InvalidPuzzleData(format!(
            "No puzzle available for date: {}. The WaPo API only keeps recent puzzles.",
            date
        )));
    }

    let data: WaPoData = serde_json::from_str(&text)?;
    parse(data)
}

/// Download the latest (most recent Sunday) Washington Post crossword.
pub async fn download_latest() -> Result<Puzzle, ProviderError> {
    use chrono::{Datelike, Local};

    let today = Local::now().date_naive();

    // Find the most recent Sunday
    let days_since_sunday = today.weekday().num_days_from_sunday();
    let most_recent_sunday = today - chrono::Duration::days(days_since_sunday as i64);

    let date_str = most_recent_sunday.format("%Y/%m/%d").to_string();
    download(&date_str).await
}

#[derive(Debug, Deserialize)]
struct WaPoData {
    title: Option<String>,
    creator: Option<String>,
    copyright: Option<String>,
    description: Option<String>,
    width: usize,
    cells: Vec<WaPoCell>,
    words: Vec<WaPoWord>,
}

#[derive(Debug, Deserialize)]
struct WaPoCell {
    answer: Option<String>,
    circle: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
struct WaPoWord {
    clue: String,
    direction: String,
    indexes: Vec<usize>,
}

fn parse(data: WaPoData) -> Result<Puzzle, ProviderError> {
    let width = data.width;
    if width == 0 {
        return Err(ProviderError::InvalidPuzzleData(
            "Puzzle width is zero".to_string(),
        ));
    }

    let height = data.cells.len() / width;
    if height == 0 {
        return Err(ProviderError::InvalidPuzzleData(
            "Puzzle height is zero".to_string(),
        ));
    }

    // Build solution and blank grids
    let mut solution_chars: Vec<char> = Vec::with_capacity(data.cells.len());
    let mut blank_chars: Vec<char> = Vec::with_capacity(data.cells.len());

    for cell in &data.cells {
        if let Some(ans) = &cell.answer {
            solution_chars.push(ans.chars().next().unwrap_or('-'));
            blank_chars.push('-');
        } else {
            solution_chars.push('.');
            blank_chars.push('.');
        }
    }

    // Convert to grid rows
    let mut blank_grid = Vec::with_capacity(height);
    let mut solution_grid = Vec::with_capacity(height);

    for row in 0..height {
        let start = row * width;
        let end = start + width;
        blank_grid.push(blank_chars[start..end].iter().collect());
        solution_grid.push(solution_chars[start..end].iter().collect());
    }

    // Build clues - sort by first index then direction
    let mut words_sorted = data.words.clone();
    words_sorted.sort_by(|a, b| {
        let a_idx = a.indexes.first().copied().unwrap_or(0);
        let b_idx = b.indexes.first().copied().unwrap_or(0);
        a_idx.cmp(&b_idx).then_with(|| a.direction.cmp(&b.direction))
    });

    // Calculate clue numbers based on grid positions
    let mut clue_positions: Vec<usize> = words_sorted
        .iter()
        .filter_map(|w| w.indexes.first().copied())
        .collect();
    clue_positions.sort_unstable();
    clue_positions.dedup();

    let clue_number_map: HashMap<usize, u16> = clue_positions
        .iter()
        .enumerate()
        .map(|(idx, &pos)| (pos, (idx + 1) as u16))
        .collect();

    let mut across_clues = HashMap::new();
    let mut down_clues = HashMap::new();

    for word in &words_sorted {
        let first_idx = word.indexes.first().copied().unwrap_or(0);
        let clue_no = *clue_number_map.get(&first_idx).unwrap_or(&1);

        if word.direction == "across" {
            across_clues.insert(clue_no, word.clue.trim().to_string());
        } else {
            down_clues.insert(clue_no, word.clue.trim().to_string());
        }
    }

    let title = data.title.unwrap_or_default();
    let author = data.creator.unwrap_or_default();
    let copyright = data.copyright.unwrap_or_default();
    let notes = data.description.unwrap_or_default();

    Ok(Puzzle {
        info: puz_parse::PuzzleInfo {
            title,
            height: height as u8,
            width: width as u8,
            author,
            copyright,
            notes,
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
    async fn test_download_by_date() {
        // Use the most recent Sunday dynamically
        use chrono::{Datelike, Local};
        let today = Local::now().date_naive();
        let days_since_sunday = today.weekday().num_days_from_sunday();
        let most_recent_sunday = today - chrono::Duration::days(days_since_sunday as i64);
        let date_str = most_recent_sunday.format("%Y/%m/%d").to_string();

        match download(&date_str).await {
            Ok(puzzle) => {
                println!("Downloaded: {}", puzzle.info.title);
                println!("Author: {}", puzzle.info.author);
                println!("Size: {}x{}", puzzle.info.width, puzzle.info.height);
                assert!(puzzle.info.width > 0);
                assert!(puzzle.info.height > 0);
            }
            Err(e) => panic!("Download failed for {}: {}", date_str, e),
        }
    }

    #[tokio::test]
    async fn test_download_latest() {
        match download_latest().await {
            Ok(puzzle) => {
                println!("Downloaded: {}", puzzle.info.title);
                assert!(puzzle.info.width > 0);
            }
            Err(e) => panic!("Download failed: {}", e),
        }
    }
}
