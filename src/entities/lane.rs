use git2::Oid;

#[derive(Debug, Clone)]
pub struct Lane {
    pub commits: Vec<Option<Oid>>,
}

impl Lane {
    pub fn new(commits: Vec<Option<Oid>>) -> Self {
        Lane { commits }
    }
}
