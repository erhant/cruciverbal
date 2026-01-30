use crate::util::http_client;
use crate::ProviderError;
use puz_parse::Puzzle;
use std::collections::HashMap;

/// Download the Lovatt's Cryptic crossword for the given date.
///
/// ## Arguments
/// - `date` - Date string in "yyyy-mm-dd" format (TODO: accept chrono or something)
pub async fn download(date: &str) -> Result<puz_parse::Puzzle, ProviderError> {
    // PSID 100000160 is used for Lovatt's cryptic crossword.
    let psid = 100000160;
    let url =
        format!("https://data.puzzlexperts.com/puzzleapp-v3/data.php?date={date}&psid={psid}");
    // example:
    // https://data.puzzlexperts.com/puzzleapp-v3/data.php?date=2025-15-12&psid=100000160

    let client = http_client();
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        todo!("handle error")
    }

    let body: serde_json::Value = res.json().await?;
    // get cells[0].meta.data
    let data = body
        .get("cells")
        .and_then(|cells| cells.get(0))
        .and_then(|cell0| cell0.get("meta"))
        .and_then(|meta| meta.get("data"))
        .and_then(|data| data.as_str())
        .ok_or_else(|| {
            ProviderError::Other("Failed to get puzzle data from response".to_string())
        })?;
    let puz = parse(data)?;

    // validate puzzle dimensions
    // sometimes some providers return 200 code but invalid puzzle data
    if puz.info.height == 0 || puz.info.width == 0 {
        return Err(ProviderError::InvalidPuzzleData(
            "Parsed puzzle has zero height or width".to_string(),
        ));
    }

    Ok(puz)
}

/// Puzzleexperts return a `data` field which itself is a string, which must be separated by `&` character.
/// - `num_rows=<number>`
/// - `num_cols=<number>`
/// - `id=<string>`
/// - `title=<string>`
/// - `category=<string>`
/// - `difficulty=<number>`
/// - `competition=<string Y|N`
///
/// The rest of the fields are repeating as follows:
///   - `word0=<string>`
///   - `clue0=<string>`,
///   - `dir0=<direction a|d>`, (a: accross, d: down)
///   - `start_j0=0`, (starting row)
///   - `start_k0=0`, (starting column)
/// (and repeating for word1, word2, etc.)
fn parse(data: &str) -> Result<puz_parse::Puzzle, ProviderError> {
    #[derive(Default, Debug)]
    struct ClueWord {
        word: String,
        clue: String,
        direction: Direction,
        row: usize,
        col: usize,
    }
    #[derive(Default, Debug)]
    enum Direction {
        #[default]
        Across,
        Down,
    }

    let mut cluewords = HashMap::<usize, ClueWord>::new();
    let mut title: String = String::new();
    let mut id: String = String::new();
    let (mut height, mut width): (u8, u8) = (0, 0);

    let lines = data
        .split('&')
        .filter(|s| !s.is_empty()) // some lines are empty, ignore
        .collect::<Vec<&str>>();

    for line in lines {
        // expect format to be `key=value`
        let mut kv = line.split('=');
        let lhs = kv.next().expect("expected lhs");
        let rhs = kv.next().expect("expected rhs");
        assert!(kv.next().is_none(), "expected only one '=' in line");

        enum ParsedField {
            Word,
            Clue,
            Direction,
            Row,
            Col,
        }

        // parse based on lhs
        match lhs {
            "num_rows" => height = rhs.parse()?,
            "num_columns" => width = rhs.parse()?,
            "title" => title = rhs.to_string(),
            "id" => id = rhs.to_string(),
            "category" | "difficulty" | "competition" => {
                // ignore other fields for now, maybe can add them to `PuzzleInfo` notes?
            }
            lhs => {
                // we now have a key-value pair as `<prefix><index>=<value>`
                let (idx, field): (usize, ParsedField);
                if let Some(i) = lhs.strip_prefix("word") {
                    (idx, field) = (i.parse()?, ParsedField::Word);
                } else if let Some(i) = lhs.strip_prefix("clue") {
                    (idx, field) = (i.parse()?, ParsedField::Clue);
                } else if let Some(i) = lhs.strip_prefix("dir") {
                    (idx, field) = (i.parse()?, ParsedField::Direction);
                } else if let Some(i) = lhs.strip_prefix("start_j") {
                    (idx, field) = (i.parse()?, ParsedField::Row);
                } else if let Some(i) = lhs.strip_prefix("start_k") {
                    (idx, field) = (i.parse()?, ParsedField::Col);
                } else {
                    return Err(ProviderError::Other(format!("Unknown key: {lhs}")));
                }

                // insert if not exists
                let entry = cluewords.entry(idx).or_default();
                match field {
                    ParsedField::Word => entry.word = rhs.to_string(),
                    ParsedField::Clue => entry.clue = rhs.to_string(),
                    ParsedField::Direction => {
                        entry.direction = match rhs {
                            "a" => Direction::Across,
                            "d" => Direction::Down,
                            _ => {
                                return Err(ProviderError::Other(format!(
                                    "Invalid direction: {rhs}"
                                )));
                            }
                        }
                    }
                    ParsedField::Row => entry.row = rhs.parse()?,
                    ParsedField::Col => entry.col = rhs.parse()?,
                };
            }
        };
    }

    // now we have cluewords, we construct the puzzle grid
    // first, create an empty grid with all filled squares
    let mut blank_grid = Vec::with_capacity(height as usize);
    for _ in 0..height as usize {
        let row = ".".repeat(width as usize);
        blank_grid.push(row);
    }

    // then, go through each clueword and fill in the grid
    let mut solution_grid = blank_grid.clone();
    let mut clue_positions: Vec<usize> = Vec::with_capacity(cluewords.len());
    for clueword in cluewords.values() {
        let word = &clueword.word;
        let row = clueword.row;
        let col = clueword.col;
        clue_positions.push(row * (height as usize) + col);
        match clueword.direction {
            Direction::Across => {
                for (i, ch) in word.chars().enumerate() {
                    blank_grid[row].replace_range(col + i..col + i + 1, "-");
                    solution_grid[row].replace_range(col + i..col + i + 1, &ch.to_string());
                }
            }
            Direction::Down => {
                for (i, ch) in word.chars().enumerate() {
                    blank_grid[row + i].replace_range(col..col + 1, "-");
                    solution_grid[row + i].replace_range(col..col + 1, &ch.to_string());
                }
            }
        }
    }

    // derive clue numbers based on positions (sorted by row-major order)
    clue_positions.sort_unstable();
    clue_positions.dedup(); // important: remove duplicates
    let clue_number_map: HashMap<usize, u16> = HashMap::from_iter(
        clue_positions
            .iter()
            .enumerate()
            .map(|(idx, pos)| (*pos, (idx + 1) as u16)),
    );
    // split the hashmap into across and down clues
    let mut across_clues = HashMap::<u16, String>::new();
    let mut down_clues = HashMap::<u16, String>::new();
    for clueword in cluewords.into_values() {
        let clue_no = *clue_number_map
            .get(&(clueword.row * (height as usize) + clueword.col))
            .unwrap();
        match clueword.direction {
            Direction::Across => across_clues.insert(clue_no, clueword.clue),
            Direction::Down => down_clues.insert(clue_no, clueword.clue),
        };
    }

    Ok(Puzzle {
        info: puz_parse::PuzzleInfo {
            title,
            height,
            width,
            author: "Lovatt's Cryptic".to_string(),
            copyright: "Copyright 2025 Lovatts Media Group".to_string(), // TODO: ???
            notes: format!("Puzzle ID: {}", id),
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
    async fn test_download() {
        match download("2026-01-25").await {
            Ok(obj) => println!("Download test passed: {:#?}", obj),
            Err(e) => panic!("Download test failed: {}", e),
        }
    }
}
