use crate::ProviderError;
use crate::util::http_client;
use puz_parse::Puzzle;
use serde::Deserialize;
use std::collections::HashMap;

/// Guardian crossword variants
#[derive(Debug, Clone, Copy)]
pub enum GuardianVariant {
    Cryptic,
    Everyman,
    Speedy,
    Quick,
    Prize,
    Weekend,
    Quiptic,
}

impl GuardianVariant {
    fn series_path(&self) -> &'static str {
        match self {
            GuardianVariant::Cryptic => "series/cryptic",
            GuardianVariant::Everyman => "series/everyman",
            GuardianVariant::Speedy => "series/speedy",
            GuardianVariant::Quick => "series/quick",
            GuardianVariant::Prize => "series/prize",
            GuardianVariant::Weekend => "series/weekend-crossword",
            GuardianVariant::Quiptic => "series/quiptic",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            GuardianVariant::Cryptic => "Guardian Cryptic",
            GuardianVariant::Everyman => "Guardian Everyman",
            GuardianVariant::Speedy => "Guardian Speedy",
            GuardianVariant::Quick => "Guardian Quick",
            GuardianVariant::Prize => "Guardian Prize",
            GuardianVariant::Weekend => "Guardian Weekend",
            GuardianVariant::Quiptic => "Guardian Quiptic",
        }
    }
}

/// Download the latest Guardian crossword for the given variant.
pub async fn download_latest(variant: GuardianVariant) -> Result<Puzzle, ProviderError> {
    let landing_url = format!(
        "https://www.theguardian.com/crosswords/{}",
        variant.series_path()
    );

    let client = http_client();
    let res = client.get(&landing_url).send().await?;

    if !res.status().is_success() {
        return Err(ProviderError::Other(format!(
            "Failed to fetch landing page: {}",
            res.status()
        )));
    }

    let html = res.text().await?;

    // Find the latest puzzle link
    let puzzle_url = extract_latest_puzzle_url(&html)?;

    download_from_url(&puzzle_url).await
}

/// Download a Guardian crossword from a specific URL.
pub async fn download_from_url(url: &str) -> Result<Puzzle, ProviderError> {
    let client = http_client();
    let res = client.get(url).send().await?;

    if !res.status().is_success() {
        return Err(ProviderError::Other(format!(
            "Failed to fetch puzzle page: {}",
            res.status()
        )));
    }

    let html = res.text().await?;
    let json_data = extract_crossword_json(&html)?;
    parse(json_data)
}

fn extract_latest_puzzle_url(html: &str) -> Result<String, ProviderError> {
    // Look for links matching /crosswords/<type>/<number>
    for line in html.lines() {
        if let Some(start) = line.find("href=\"/crosswords/") {
            let rest = &line[start + 6..]; // skip 'href="'
            if let Some(end) = rest.find('"') {
                let path = &rest[..end];
                // Make sure it's a puzzle link (ends with digits)
                if path.contains("/crosswords/")
                    && path
                        .rsplit('/')
                        .next()
                        .is_some_and(|s| s.chars().all(|c| c.is_ascii_digit()))
                {
                    return Ok(format!("https://www.theguardian.com{}", path));
                }
            }
        }
    }

    // Fallback: use regex-like manual search
    let search_start = "/crosswords/";
    let mut pos = 0;
    while let Some(idx) = html[pos..].find(search_start) {
        let start = pos + idx;
        let rest = &html[start..];
        if let Some(end) = rest.find('"') {
            let path = &rest[..end];
            if path
                .rsplit('/')
                .next()
                .is_some_and(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
            {
                return Ok(format!("https://www.theguardian.com{}", path));
            }
        }
        pos = start + 1;
    }

    Err(ProviderError::Other(
        "Could not find puzzle link on landing page".to_string(),
    ))
}

fn extract_crossword_json(html: &str) -> Result<GuardianData, ProviderError> {
    // Find <gu-island name="CrosswordComponent" ... props="...">
    let marker = r#"name="CrosswordComponent""#;
    let pos = html.find(marker).ok_or_else(|| {
        ProviderError::Other("Could not find CrosswordComponent in page".to_string())
    })?;

    // Find props attribute after the marker
    let rest = &html[pos..];
    let props_marker = "props=\"";
    let props_start = rest
        .find(props_marker)
        .ok_or_else(|| ProviderError::Other("Could not find props attribute".to_string()))?
        + props_marker.len();

    let props_rest = &rest[props_start..];

    // Find the end of the props attribute (next unescaped quote)
    let mut in_escape = false;
    let mut end_pos = 0;
    for (i, c) in props_rest.char_indices() {
        if in_escape {
            in_escape = false;
            continue;
        }
        if c == '\\' {
            in_escape = true;
            continue;
        }
        if c == '"' {
            end_pos = i;
            break;
        }
    }

    let props_encoded = &props_rest[..end_pos];

    // Decode HTML entities
    let props_json = props_encoded
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#x27;", "'")
        .replace("&#39;", "'");

    let wrapper: GuardianWrapper = serde_json::from_str(&props_json)?;

    Ok(wrapper.data)
}

#[derive(Debug, Deserialize)]
struct GuardianWrapper {
    data: GuardianData,
}

#[derive(Debug, Deserialize)]
struct GuardianData {
    name: Option<String>,
    creator: Option<GuardianCreator>,
    dimensions: GuardianDimensions,
    entries: Vec<GuardianEntry>,
    #[allow(unused)]
    date: i64,
}

#[derive(Debug, Deserialize)]
struct GuardianCreator {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GuardianDimensions {
    rows: u8,
    cols: u8,
}

#[derive(Debug, Deserialize)]
struct GuardianEntry {
    direction: String,
    position: GuardianPosition,
    length: usize,
    solution: Option<String>,
    clue: String,
    number: u16,
}

#[derive(Debug, Deserialize)]
struct GuardianPosition {
    x: usize,
    y: usize,
}

fn parse(data: GuardianData) -> Result<Puzzle, ProviderError> {
    let height = data.dimensions.rows;
    let width = data.dimensions.cols;

    // Build the grid from entries
    let mut grid: Vec<Vec<char>> = vec![vec!['.'; width as usize]; height as usize];

    for entry in &data.entries {
        let solution = entry.solution.as_deref().unwrap_or("");
        let mut x = entry.position.x;
        let mut y = entry.position.y;

        for (i, ch) in solution.chars().enumerate() {
            if i >= entry.length {
                break;
            }
            if y < height as usize && x < width as usize {
                grid[y][x] = ch;
            }
            if entry.direction == "across" {
                x += 1;
            } else {
                y += 1;
            }
        }
    }

    // Build blank and solution grids
    let mut blank_grid = Vec::with_capacity(height as usize);
    let mut solution_grid = Vec::with_capacity(height as usize);

    for row in &grid {
        let mut blank_row = String::with_capacity(width as usize);
        let mut sol_row = String::with_capacity(width as usize);
        for &ch in row {
            if ch == '.' {
                blank_row.push('.');
                sol_row.push('.');
            } else {
                blank_row.push('-');
                sol_row.push(ch);
            }
        }
        blank_grid.push(blank_row);
        solution_grid.push(sol_row);
    }

    // Build clues - sort by number then direction (across before down)
    let mut entries_sorted: Vec<_> = data.entries.iter().collect();
    entries_sorted.sort_by(|a, b| {
        a.number
            .cmp(&b.number)
            .then_with(|| a.direction.cmp(&b.direction))
    });

    let mut across_clues = HashMap::new();
    let mut down_clues = HashMap::new();

    for entry in entries_sorted {
        if entry.direction == "across" {
            across_clues.insert(entry.number, entry.clue.clone());
        } else {
            down_clues.insert(entry.number, entry.clue.clone());
        }
    }

    let title = data.name.unwrap_or_default();
    let author = data
        .creator
        .map(|c| c.name)
        .unwrap_or_else(|| "The Guardian".to_string());

    // Check if we have a solution
    let has_solution = data.entries.iter().all(|e| e.solution.is_some());
    let notes = if has_solution {
        String::new()
    } else {
        "No solution provided".to_string()
    };

    Ok(Puzzle {
        info: puz_parse::PuzzleInfo {
            title,
            height,
            width,
            author,
            copyright: "Copyright The Guardian".to_string(),
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
    async fn test_download_cryptic() {
        match download_latest(GuardianVariant::Cryptic).await {
            Ok(puzzle) => {
                println!("Downloaded: {}", puzzle.info.title);
                assert!(puzzle.info.width > 0);
                assert!(puzzle.info.height > 0);
            }
            Err(e) => panic!("Download failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_download_quick() {
        match download_latest(GuardianVariant::Quick).await {
            Ok(puzzle) => {
                println!("Downloaded: {}", puzzle.info.title);
                assert!(puzzle.info.width > 0);
            }
            Err(e) => panic!("Download failed: {}", e),
        }
    }
}
