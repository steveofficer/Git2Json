#[derive(Clone, Debug)]
pub enum ObjectType {
    Tree,
    Commit,
    Blob
}

#[derive(Clone, Debug)]
pub struct GitTreeEntry {
    pub code: String,
    pub file_name: String,
    pub hash: String
}

#[derive(Clone, Debug)]
pub struct GitTree {
    pub entries: Vec<GitTreeEntry>
}

#[derive(Clone, Debug)]
pub struct GitCommit {
    pub metadata: Vec<String>,
    pub content: Vec<String>
}

#[derive(Clone, Debug)]
pub struct GitBlob {
    pub content: Vec<String>
}

#[derive(Clone, Debug)]
pub enum GitObject {
    Commit(GitCommit),
    Tree(GitTree),
    Blob(GitBlob)
}

#[derive(Clone, Debug)]
pub struct GitIndexFile {
    pub object_names: Vec<String>,
    pub offsets: Vec<u32>
}