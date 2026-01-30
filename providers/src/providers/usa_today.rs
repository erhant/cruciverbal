use crate::util::{http_client, url_decode};
use crate::ProviderError;
use puz_parse::Puzzle;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::collections::HashMap;

/// Download the USA Today crossword for the given date.
///
/// ## Arguments
/// - `date` - Date string in "yyyy-mm-dd" format
pub async fn download(date: &str) -> Result<Puzzle, ProviderError> {
    // Parse date and convert to yymmdd format
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return Err(ProviderError::Other(format!(
            "Invalid date format: {}. Expected yyyy-mm-dd",
            date
        )));
    }

    let year = &parts[0][2..]; // Get last 2 digits of year
    let month = parts[1];
    let day = parts[2];
    let date_formatted = format!("{}{}{}", year, month, day);

    let url = format!(
        "http://picayune.uclick.com/comics/usaon/data/usaon{}-data.xml",
        date_formatted
    );

    let client = http_client();
    let res = client.get(&url).send().await?;

    if !res.status().is_success() {
        return Err(ProviderError::Other(format!(
            "Failed to fetch puzzle: {} (status: {})",
            url,
            res.status()
        )));
    }

    let xml_content = res.text().await?;
    parse(&xml_content)
}

/// Download the latest USA Today crossword (tries today, then previous days).
pub async fn download_latest() -> Result<Puzzle, ProviderError> {
    use chrono::Local;

    let mut check_date = Local::now().date_naive();
    let mut days_to_check = 3;

    while days_to_check > 0 {
        let date_str = check_date.format("%Y-%m-%d").to_string();
        match download(&date_str).await {
            Ok(puzzle) => return Ok(puzzle),
            Err(_) => {
                days_to_check -= 1;
                check_date -= chrono::Duration::days(1);
            }
        }
    }

    Err(ProviderError::Other(
        "Unable to find latest puzzle".to_string(),
    ))
}

#[derive(Debug, Deserialize)]
struct Crossword {
    #[serde(rename = "Title")]
    title: Option<AttributeValue>,
    #[serde(rename = "Author")]
    author: Option<AttributeValue>,
    #[serde(rename = "Copyright")]
    copyright: Option<AttributeValue>,
    #[serde(rename = "Width")]
    width: AttributeValue,
    #[serde(rename = "Height")]
    height: AttributeValue,
    #[serde(rename = "AllAnswer")]
    all_answer: AttributeValue,
    across: ClueDirection,
    down: ClueDirection,
}

#[derive(Debug, Deserialize)]
struct AttributeValue {
    #[serde(rename = "@v")]
    v: String,
}

#[derive(Debug, Deserialize)]
struct ClueDirection {
    #[serde(rename = "$value", default)]
    clues: Vec<Clue>,
}

#[derive(Debug, Deserialize)]
struct Clue {
    #[serde(rename = "@cn")]
    cn: String,
    #[serde(rename = "@c")]
    c: Option<String>,
}

fn parse(xml_content: &str) -> Result<Puzzle, ProviderError> {
    let crossword: Crossword = from_str(xml_content).map_err(|e| {
        ProviderError::Other(format!("Failed to parse XML: {}", e))
    })?;

    let width: u8 = crossword.width.v.parse()?;
    let height: u8 = crossword.height.v.parse()?;

    if width == 0 || height == 0 {
        return Err(ProviderError::InvalidPuzzleData(
            "Puzzle has zero dimensions".to_string(),
        ));
    }

    // Parse solution - '-' in the XML means black square
    let solution_raw = &crossword.all_answer.v;
    let solution_chars: Vec<char> = solution_raw
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

    // Parse clues
    let mut across_clues = HashMap::new();
    let mut down_clues = HashMap::new();

    for clue in &crossword.across.clues {
        let clue_no: u16 = clue.cn.parse().unwrap_or(0);
        let clue_text = url_decode(clue.c.as_deref().unwrap_or(""));
        across_clues.insert(clue_no, clue_text);
    }

    for clue in &crossword.down.clues {
        let clue_no: u16 = clue.cn.parse().unwrap_or(0);
        let clue_text = url_decode(clue.c.as_deref().unwrap_or(""));
        down_clues.insert(clue_no, clue_text);
    }

    let title = url_decode(crossword.title.as_ref().map(|t| t.v.as_str()).unwrap_or(""));
    let author = url_decode(crossword.author.as_ref().map(|a| a.v.as_str()).unwrap_or(""));
    let copyright = url_decode(
        crossword
            .copyright
            .as_ref()
            .map(|c| c.v.as_str())
            .unwrap_or(""),
    );

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
