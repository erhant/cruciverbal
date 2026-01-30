pub mod formats;
pub mod providers;
pub mod util;

mod errors;
pub use errors::ProviderError;

// Re-export provider modules for convenience
pub use providers::guardian::{self, GuardianVariant};
pub use providers::simply_daily::{self, SimplyDailyVariant};

/// Available puzzle providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PuzzleProvider {
    #[default]
    LovattsCryptic,
    // Guardian variants
    GuardianCryptic,
    GuardianEveryman,
    GuardianSpeedy,
    GuardianQuick,
    GuardianPrize,
    GuardianWeekend,
    GuardianQuiptic,
    // Washington Post
    WashingtonPost,
    // USA Today
    UsaToday,
    // Simply Daily variants
    SimplyDaily,
    SimplyDailyCryptic,
    SimplyDailyQuick,
    // Universal
    Universal,
    // Daily Pop
    DailyPop,
}

impl PuzzleProvider {
    pub const ALL: [PuzzleProvider; 15] = [
        PuzzleProvider::SimplyDaily,
        PuzzleProvider::SimplyDailyCryptic,
        PuzzleProvider::SimplyDailyQuick,
        PuzzleProvider::GuardianCryptic,
        PuzzleProvider::GuardianEveryman,
        PuzzleProvider::GuardianSpeedy,
        PuzzleProvider::GuardianQuick,
        PuzzleProvider::GuardianPrize,
        PuzzleProvider::GuardianWeekend,
        PuzzleProvider::GuardianQuiptic,
        PuzzleProvider::WashingtonPost,
        PuzzleProvider::UsaToday,
        PuzzleProvider::Universal,
        PuzzleProvider::DailyPop,
        PuzzleProvider::LovattsCryptic,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            PuzzleProvider::LovattsCryptic => "Lovatts Cryptic",
            PuzzleProvider::GuardianCryptic => "Guardian Cryptic",
            PuzzleProvider::GuardianEveryman => "Guardian Everyman",
            PuzzleProvider::GuardianSpeedy => "Guardian Speedy",
            PuzzleProvider::GuardianQuick => "Guardian Quick",
            PuzzleProvider::GuardianPrize => "Guardian Prize",
            PuzzleProvider::GuardianWeekend => "Guardian Weekend",
            PuzzleProvider::GuardianQuiptic => "Guardian Quiptic",
            PuzzleProvider::WashingtonPost => "Washington Post",
            PuzzleProvider::UsaToday => "USA Today",
            PuzzleProvider::SimplyDaily => "Simply Daily",
            PuzzleProvider::SimplyDailyCryptic => "Simply Daily Cryptic",
            PuzzleProvider::SimplyDailyQuick => "Simply Daily Quick",
            PuzzleProvider::Universal => "Universal",
            PuzzleProvider::DailyPop => "Daily Pop",
        }
    }

    /// Get the Guardian variant if this is a Guardian provider
    pub fn guardian_variant(&self) -> Option<GuardianVariant> {
        match self {
            PuzzleProvider::GuardianCryptic => Some(GuardianVariant::Cryptic),
            PuzzleProvider::GuardianEveryman => Some(GuardianVariant::Everyman),
            PuzzleProvider::GuardianSpeedy => Some(GuardianVariant::Speedy),
            PuzzleProvider::GuardianQuick => Some(GuardianVariant::Quick),
            PuzzleProvider::GuardianPrize => Some(GuardianVariant::Prize),
            PuzzleProvider::GuardianWeekend => Some(GuardianVariant::Weekend),
            PuzzleProvider::GuardianQuiptic => Some(GuardianVariant::Quiptic),
            _ => None,
        }
    }

    /// Get the Simply Daily variant if this is a Simply Daily provider
    pub fn simply_daily_variant(&self) -> Option<SimplyDailyVariant> {
        match self {
            PuzzleProvider::SimplyDaily => Some(SimplyDailyVariant::Regular),
            PuzzleProvider::SimplyDailyCryptic => Some(SimplyDailyVariant::Cryptic),
            PuzzleProvider::SimplyDailyQuick => Some(SimplyDailyVariant::Quick),
            _ => None,
        }
    }
}
