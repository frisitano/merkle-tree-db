/// Errors for the tree library
#[derive(Debug)]
pub enum TreeError {
    DataNotFound,
    DecodeNodeFailed,
    DecodeHashFailed,
    InvalidHeight,
    InconsistentDefaultHashes,
    MissingChild,
    UnexpectedNodeType,
}
