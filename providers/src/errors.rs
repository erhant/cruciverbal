pub enum ProviderError {
    FetchError(reqwest::Error),
    SerdeParseError(serde_json::Error),
    PuzParseError(puz_parse::PuzError),
}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        ProviderError::FetchError(err)
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(err: serde_json::Error) -> Self {
        ProviderError::SerdeParseError(err)
    }
}

impl From<puz_parse::PuzError> for ProviderError {
    fn from(err: puz_parse::PuzError) -> Self {
        ProviderError::PuzParseError(err)
    }
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::FetchError(e) => write!(f, "Fetch error: {}", e),
            ProviderError::SerdeParseError(e) => write!(f, "Serde parse error: {}", e),
            ProviderError::PuzParseError(e) => write!(f, "PUZ parse error: {}", e),
        }
    }
}
