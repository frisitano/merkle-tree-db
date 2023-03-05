use super::{
    DBValue, HashDBRef, Hasher, IndexTree, Key, KeyedTree, TreeDB, TreeDBBuilder, TreeError,
    TreeRecorder,
};

// IndexTreeDBBuilder
// ================================================================================================

/// Used to construct an IndexTreeDB
pub struct IndexTreeDBBuilder<'db, const D: usize, H: Hasher> {
    db: &'db dyn HashDBRef<H, DBValue>,
    root: &'db H::Out,
    recorder: Option<&'db mut dyn TreeRecorder<H>>,
}

impl<'db, const D: usize, H: Hasher> IndexTreeDBBuilder<'db, D, H> {
    /// Construct a new IndexTreeDBBuilder
    pub fn new(db: &'db dyn HashDBRef<H, DBValue>, root: &'db H::Out) -> Result<Self, TreeError> {
        if D > usize::MAX / 8 {
            return Err(TreeError::DepthTooLarge(D, usize::MAX / 8));
        }
        Ok(Self {
            db,
            root,
            recorder: None,
        })
    }

    /// Add a recorder to the IndexTreeDBBuilder
    pub fn with_recorder(mut self, recorder: &'db mut dyn TreeRecorder<H>) -> Self {
        self.recorder = Some(recorder);
        self
    }

    /// Add an optional recorder to the IndexTreeDBBuilder
    pub fn with_optional_recorder<'recorder: 'db>(
        mut self,
        recorder: Option<&'recorder mut dyn TreeRecorder<H>>,
    ) -> Self {
        self.recorder = recorder.map(|r| r as _);
        self
    }

    /// build an IndexTreeDB
    pub fn build(self) -> IndexTreeDB<'db, D, H> {
        let keyed_db = TreeDBBuilder::new(self.db, self.root)
            .expect("checks are applied in IndexTreeDBBuilder constructor")
            .with_optional_recorder(self.recorder)
            .build();
        IndexTreeDB { keyed_db }
    }
}

// IndexTreeDB
// ================================================================================================

/// An immutable merkle tree db that uses a u64 index to specify the leaves in the tree. Wraps a KeyedTreeDB
/// and converts a u64 index to a Key of the appropriate depth to access the underlying TreeDB.
pub struct IndexTreeDB<'db, const D: usize, H: Hasher> {
    keyed_db: TreeDB<'db, D, H>,
}

impl<'db, H: Hasher + 'db, const D: usize> IndexTree<H, D> for IndexTreeDB<'db, D, H> {
    /// Returns the root of the tree
    fn root(&self) -> &<H as Hasher>::Out {
        self.keyed_db.root()
    }

    /// Returns the value at the given index
    fn value(&self, index: &u64) -> Result<Option<DBValue>, TreeError> {
        let key = Key::<D>::try_from(index).map_err(TreeError::KeyError)?;
        self.keyed_db.value(key.as_slice())
    }

    /// Returns the leaf at the given index
    fn leaf(&self, index: &u64) -> Result<Option<<H as Hasher>::Out>, TreeError> {
        let key = Key::<D>::try_from(index).map_err(TreeError::KeyError)?;
        self.keyed_db.leaf(key.as_slice())
    }

    /// Returns an inclusion proof of a value a the specified index.
    /// Returns a tuple of form: (value, root, proof)  
    fn proof(&self, index: &u64) -> Result<(Option<DBValue>, H::Out, Vec<DBValue>), TreeError> {
        let key = Key::<D>::try_from(index).map_err(TreeError::KeyError)?;
        self.keyed_db.proof(key.as_slice())
    }

    /// Verifies that the given value is in the tree with the given root at the given index
    fn verify(
        index: &u64,
        value: &[u8],
        proof: &[DBValue],
        root: &H::Out,
    ) -> Result<bool, TreeError> {
        let key = Key::<D>::try_from(index).map_err(TreeError::KeyError)?;
        TreeDB::<'db, D, H>::verify(key.as_slice(), value, proof, root)
    }
}
