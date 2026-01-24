pub mod providers;

mod errors;
pub use errors::ProviderError;

/// Available puzzle puzzleproviders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PuzzleProvider {
    #[default]
    LovattsCryptic,
}

impl PuzzleProvider {
    pub const ALL: [PuzzleProvider; 1] = [PuzzleProvider::LovattsCryptic];

    pub fn name(&self) -> &'static str {
        match self {
            PuzzleProvider::LovattsCryptic => "Lovatts Cryptic",
        }
    }
}
