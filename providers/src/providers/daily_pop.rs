use crate::formats::crossword_compiler;
use crate::util::http_client;
use crate::ProviderError;
use puz_parse::Puzzle;

/// Fetch the API key from the Daily Pop setup script.
async fn get_api_key(client: &reqwest::Client) -> Result<String, ProviderError> {
    let setup_url = "http://dailypopcrosswordsweb.puzzlenation.com/crosswordSetup.js";

    let res = client.get(setup_url).send().await?;

    if !res.status().is_success() {
        return Err(ProviderError::Other(
            "Failed to fetch Daily Pop setup script".to_string(),
        ));
    }

    let script = res.text().await?;

    // Look for: const API_KEY = "..."
    for line in script.lines() {
        if line.starts_with("const API_KEY = ") {
            let start = line.find('"').ok_or_else(|| {
                ProviderError::Other("Could not parse API key from script".to_string())
            })? + 1;
            let end = line.rfind('"').ok_or_else(|| {
                ProviderError::Other("Could not parse API key from script".to_string())
            })?;
            return Ok(line[start..end].to_string());
        }
    }

    Err(ProviderError::Other(
        "Could not find API key in setup script".to_string(),
    ))
}

/// Download the Daily Pop crossword for the given date.
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

    let client = http_client();

    // Get API key
    let api_key = get_api_key(&client).await?;

    let url = format!(
        "https://api.puzzlenation.com/dailyPopCrosswords/puzzles/daily/{}",
        date_formatted
    );

    let res = client
        .get(&url)
        .header("x-api-key", &api_key)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(ProviderError::Other(format!(
            "Failed to fetch puzzle: {} (status: {})",
            url,
            res.status()
        )));
    }

    let xml_content = res.text().await?;
    crossword_compiler::parse(&xml_content)
}

/// Download the latest Daily Pop crossword.
pub async fn download_latest() -> Result<Puzzle, ProviderError> {
    use chrono::Local;

    let today = Local::now().date_naive();
    let date_str = today.format("%Y-%m-%d").to_string();
    download(&date_str).await
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
