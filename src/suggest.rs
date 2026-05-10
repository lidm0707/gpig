const MAX_SUGGESTIONS: usize = 10;
const MIN_QUERY_LEN: usize = 1;

pub struct SuggestState {
    paths: Vec<String>,
}

impl Default for SuggestState {
    fn default() -> Self {
        Self::new()
    }
}

impl SuggestState {
    pub fn new() -> Self {
        Self { paths: Vec::new() }
    }

    pub fn set_paths(&mut self, paths: Vec<String>) {
        self.paths = paths;
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn filter(&self, query: &str) -> Vec<&str> {
        if query.len() < MIN_QUERY_LEN {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let mut matches: Vec<&str> = Vec::new();

        for path in &self.paths {
            if matches.len() >= MAX_SUGGESTIONS {
                break;
            }
            if path_matches(path, &query_lower) {
                matches.push(path.as_str());
            }
        }

        matches
    }
}

fn path_matches(path: &str, query_lower: &str) -> bool {
    let path_lower = path.to_lowercase();

    if path_lower.contains(query_lower) {
        return true;
    }

    let query_parts: Vec<&str> = query_lower.split('/').collect();
    if query_parts.len() <= 1 {
        return false;
    }

    let path_parts: Vec<&str> = path_lower.split('/').collect();
    if query_parts.len() > path_parts.len() {
        return false;
    }

    let start = path_parts.len() - query_parts.len();
    path_parts[start..]
        .iter()
        .zip(query_parts.iter())
        .all(|(pp, qp)| pp.starts_with(qp))
}
