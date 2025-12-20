use git2::{Oid, Time};
#[derive(Debug, Clone)]
pub struct CommitNode {
    pub oid: Oid,
    pub message: String,
    pub author: String,
    pub timestamp: Time,
    pub parents: Vec<Oid>,
}

impl CommitNode {
    pub fn new(
        oid: Oid,
        message: String,
        author: String,
        timestamp: Time,
        parents: Vec<Oid>,
    ) -> Self {
        CommitNode {
            oid,
            message,
            author,
            timestamp,
            parents,
        }
    }
}
