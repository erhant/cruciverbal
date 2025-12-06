use puz_parse::{Puzzle, parse_bytes};
use serde_json::Value;

pub async fn download() -> Result<(), crate::ProviderError> {
    // Set the date and PSID as cliptic does.
    // PSID 100000160 is used for Lovatt's cryptic crossword in cliptic.
    let date = "2025-12-04"; // change as desired.
    let psid = 100000160;
    let url = format!(
        "https://data.puzzlexperts.com/puzzleapp-v3/data.php?date={}&psid={}",
        date, psid
    );
    // https://data.puzzlexperts.com/puzzleapp-v3/data.php?date=2025-12-04&psid=100000160

    let res = reqwest::get(&url).await?;
    if !res.status().is_success() {
        println!("Failed to fetch puzzle! Status: {}", res.status());
        return Ok(());
    }

    // let body = res.bytes()?;
    let body = res.text().await?;
    print!("Downloaded puzzle data:\n\n{}\n\n", body);
    // let obj = parse_bytes(&body)?;
    let json: Value = serde_json::from_str(&body)?;
    let obj = json.as_object().unwrap();

    // Print out the JSON structure for inspection
    println!("Raw puzzle JSON: {:#?}", obj);

    // To handle clues and grid, you'd dig into the structure here, for example:
    // let clues_data = json["cells"][0]["meta"]["data"].as_str();
    // Further processing as in Cliptic would extract clues etc from this string.
    let x: Puzzle;
    Ok(())
}

/// Puzzleexperts return a `data` field which itself is a string, which must be separated by `&` character.
/// After splitting, the first three fields are metadata:
/// - `''` (empty)
/// - `num_rows=<number>`
/// - `num_cols=<number>`
///
/// The rest of the fields are repeating as follows:
///   - 'word0=<string>'
///   - 'clue0=<string>',
///   - 'dir0=<direction a|d>', (a: accross, d: down)
///   - 'start_j0=0',
///   - 'start_k0=0',
/// (and repeating for word1, word2, etc.)
///
/// Then, we have the following:
/// - 'id=<string>',
/// - 'title=<string>',
/// - 'category=<string>',
/// - 'difficulty=<number>',
/// - '',
/// - 'competition=<string Y|N>'
pub fn parse(bytes: &[u8]) -> Result<puz_parse::Puzzle, crate::ProviderError> {
    // Implement parsing logic here, similar to Cliptic provider.
    // This is a placeholder implementation.
    let puzzle = puz_parse::parse_bytes(bytes)?;
    Ok(puzzle)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download() {
        match download().await {
            Ok(_) => println!("Download test passed."),
            Err(e) => panic!("Download test failed: {}", e),
        }
    }
}
