use crate::util::{http_client, url_decode};
use crate::ProviderError;
use puz_parse::Puzzle;
use serde::Deserialize;
use std::collections::HashMap;

/// The URL blob for Universal crosswords
const UNIVERSAL_BLOB: &str = "U2FsdGVkX18YuMv20%2B8cekf85%2Friz1H%2FzlWW4bn0cizt8yclLsp7UYv34S77X0aX%0Axa513fPTc5RoN2wa0h4ED9QWuBURjkqWgHEZey0WFL8%3D";

/// Download the Universal crossword for the given date.
///
/// ## Arguments
/// - `date` - Date string in "yyyy-mm-dd" format
pub async fn download(date: &str) -> Result<Puzzle, ProviderError> {
    let url = format!(
        "https://gamedata.services.amuniversal.com/c/uucom/l/{}/g/fcx/d/{}/data.json",
        UNIVERSAL_BLOB, date
    );

    let client = http_client();

    // Retry logic - sometimes the API is flaky
    let mut attempts = 3;
    let data: AMUniversalData = loop {
        let res = client.get(&url).send().await?;

        if !res.status().is_success() {
            return Err(ProviderError::Other(format!(
                "Failed to fetch puzzle: {} (status: {})",
                url,
                res.status()
            )));
        }

        match res.json::<AMUniversalData>().await {
            Ok(data) => break data,
            Err(e) => {
                attempts -= 1;
                if attempts == 0 {
                    return Err(ProviderError::Other(format!(
                        "Failed to parse puzzle data after retries: {}",
                        e
                    )));
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    };

    parse(data)
}

/// Download the latest Universal crossword.
pub async fn download_latest() -> Result<Puzzle, ProviderError> {
    use chrono::Local;

    let today = Local::now().date_naive();
    let date_str = today.format("%Y-%m-%d").to_string();
    download(&date_str).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AMUniversalData {
    title: Option<String>,
    author: Option<String>,
    editor: Option<String>,
    copyright: Option<String>,
    width: String,
    height: String,
    all_answer: String,
    across_clue: String,
    down_clue: String,
}

fn parse(data: AMUniversalData) -> Result<Puzzle, ProviderError> {
    let width: u8 = data.width.parse()?;
    let height: u8 = data.height.parse()?;

    if width == 0 || height == 0 {
        return Err(ProviderError::InvalidPuzzleData(
            "Puzzle has zero dimensions".to_string(),
        ));
    }

    // Parse solution - '-' means black square
    let solution_chars: Vec<char> = data
        .all_answer
        .chars()
        .map(|c| if c == '-' { '.' } else { c })
        .collect();

    // Build grids
    let mut blank_grid = Vec::with_capacity(height as usize);
    let mut solution_grid = Vec::with_capacity(height as usize);

    for row in 0..height as usize {
        let start = row * width as usize;
        let end = start + width as usize;

        let sol_row: String = solution_chars[start..end].iter().collect();
        let blank_row: String = solution_chars[start..end]
            .iter()
            .map(|&c| if c == '.' { '.' } else { '-' })
            .collect();

        solution_grid.push(sol_row);
        blank_grid.push(blank_row);
    }

    // Parse clues - format: "number|clue\nnumber|clue\n..."
    let across_lines: Vec<&str> = data.across_clue.lines().collect();
    let down_lines: Vec<&str> = data.down_clue.lines().collect();

    let mut clues_list: Vec<(u16, bool, String)> = Vec::new();

    for line in across_lines {
        if let Some((num_str, clue)) = line.split_once('|') {
            if let Ok(num) = num_str.parse::<u16>() {
                clues_list.push((num, true, url_decode(clue)));
            }
        }
    }

    for line in down_lines {
        // Skip end marker if present (USA Today has one, Universal doesn't but let's be safe)
        if line.is_empty() || line == "end" {
            continue;
        }
        if let Some((num_str, clue)) = line.split_once('|') {
            if let Ok(num) = num_str.parse::<u16>() {
                clues_list.push((num, false, url_decode(clue)));
            }
        }
    }

    // Sort clues by number, then by direction (across before down)
    clues_list.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));

    let mut across_clues = HashMap::new();
    let mut down_clues = HashMap::new();

    for (num, is_across, clue) in clues_list {
        if is_across {
            across_clues.insert(num, clue);
        } else {
            down_clues.insert(num, clue);
        }
    }

    let title = url_decode(data.title.as_deref().unwrap_or(""));
    let author = {
        let author_part = url_decode(data.author.as_deref().unwrap_or(""));
        let editor_part = url_decode(data.editor.as_deref().unwrap_or(""));
        if !editor_part.is_empty() {
            format!("{} / Ed. {}", author_part, editor_part)
        } else {
            author_part
        }
    };
    let copyright = url_decode(data.copyright.as_deref().unwrap_or(""));

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
    async fn test_download_by_date() {
        match download("2025-01-27").await {
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
