/// Errors for the tree library
#[derive(Debug)]
pub enum TreeError {
    DataNotFound,
    DecodeNodeFailed,
    DecodeHashFailed,
    InconsistentDefaultHashes,
    MissingChild,
    UnexpectedNodeType,
    KeyError(KeyError),
}

#[derive(Debug)]
pub enum KeyError {
    KeyTooLarge(usize, usize),
}
