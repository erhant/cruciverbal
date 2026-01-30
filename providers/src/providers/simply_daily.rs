use crate::formats::crossword_compiler;
use crate::util::http_client;
use crate::ProviderError;
use puz_parse::Puzzle;

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

    let client = http_client();
    let res = client.get(&url).send().await?;

    if !res.status().is_success() {
        return Err(ProviderError::Other(format!(
            "Failed to fetch puzzle: {} (status: {})",
            url,
            res.status()
        )));
    }

    let js_content = res.text().await?;
    let xml_content = crossword_compiler::extract_xml_from_js(&js_content)?;
    crossword_compiler::parse(&xml_content)
}

/// Download the latest Simply Daily puzzle.
pub async fn download_latest(variant: SimplyDailyVariant) -> Result<Puzzle, ProviderError> {
    use chrono::Local;

    let today = Local::now().date_naive();
    let date_str = today.format("%Y-%m-%d").to_string();
    download(variant, &date_str).await
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
