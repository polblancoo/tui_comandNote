#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub description: String,
    pub source: SearchSource,
}

#[derive(Debug, Clone)]
pub enum SearchSource {
    CheatsRs,
    CratesIo,
    Local,
}

impl std::fmt::Display for SearchSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchSource::CheatsRs => write!(f, "Cheats.rs"),
            SearchSource::CratesIo => write!(f, "Crates.io"),
            SearchSource::Local => write!(f, "Local"),
        }
    }
}