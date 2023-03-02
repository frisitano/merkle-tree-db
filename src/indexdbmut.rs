use super::{
    DBValue, HashDB, Hasher, IndexTreeMut, Key, KeyedTreeMut, TreeDBMut, TreeDBMutBuilder,
    TreeError, TreeRecorder,
};

// IndexTreeDBMutBuilder
// ================================================================================================

/// Used to construct a IndexTreeDBMut
pub struct IndexTreeDBMutBuilder<'db, const D: usize, H: Hasher> {
    db: &'db mut dyn HashDB<H, DBValue>,
    root: &'db mut H::Out,
    recorder: Option<&'db mut dyn TreeRecorder<H>>,
}

impl<'db, const D: usize, H: Hasher> IndexTreeDBMutBuilder<'db, D, H> {
    /// Construct a new db builder
    pub fn new(
        db: &'db mut dyn HashDB<H, DBValue>,
        root: &'db mut H::Out,
    ) -> Result<Self, TreeError> {
        if D > usize::MAX / 8 {
            return Err(TreeError::DepthTooLarge(D, usize::MAX / 8));
        }
        Ok(Self {
            db,
            root,
            recorder: None,
        })
    }

    /// Add a recorder to the db buidler
    pub fn with_recorder(mut self, recorder: &'db mut dyn TreeRecorder<H>) -> Self {
        self.recorder = Some(recorder);
        self
    }

    /// Add an optional recorder to the db builder
    pub fn with_optional_recorder<'recorder: 'db>(
        mut self,
        recorder: Option<&'recorder mut dyn TreeRecorder<H>>,
    ) -> Self {
        self.recorder = recorder.map(|r| r as _);
        self
    }

    /// Consumes the builder and returns a IndexTreeDBMut
    pub fn build(self) -> IndexTreeDBMut<'db, D, H> {
        let keyed_db = TreeDBMutBuilder::new(self.db, self.root)
            .expect("checks are done in the IndexTreeDBBuilder constructor")
            .with_optional_recorder(self.recorder)
            .build();
        IndexTreeDBMut { keyed_db }
    }
}

/// A TreeDBMut that uses a u64 index to access the underlying database.
/// Wraps a TreeDB and converts a u64 index to a Key of the appropriate depth to access
/// the underlying TreeDB.
pub struct IndexTreeDBMut<'db, const D: usize, H: Hasher> {
    keyed_db: TreeDBMut<'db, D, H>,
}

impl<'db, const D: usize, H: Hasher> IndexTreeDBMut<'db, D, H> {
    /// Commit the changes to the underlying database
    pub fn commit(&mut self) {
        self.keyed_db.commit()
    }
}

impl<'db, H: Hasher + 'db, const D: usize> IndexTreeMut<H, D> for IndexTreeDBMut<'db, D, H> {
    fn root(&mut self) -> &<H as Hasher>::Out {
        self.keyed_db.root()
    }

    fn value(&self, index: &u64) -> Result<Option<DBValue>, TreeError> {
        let key = Key::<D>::try_from(index).map_err(TreeError::KeyError)?;
        self.keyed_db.value(key.as_slice())
    }

    fn leaf(&self, index: &u64) -> Result<Option<<H as Hasher>::Out>, TreeError> {
        let key = Key::<D>::try_from(index).map_err(TreeError::KeyError)?;
        self.keyed_db.leaf(key.as_slice())
    }

    fn proof(&self, index: &u64) -> Result<Option<Vec<DBValue>>, TreeError> {
        let key = Key::<D>::try_from(index).map_err(TreeError::KeyError)?;
        self.keyed_db.proof(key.as_slice())
    }

    fn insert(&mut self, index: &u64, value: DBValue) -> Result<Option<DBValue>, TreeError> {
        let key = Key::<D>::try_from(index).map_err(TreeError::KeyError)?;
        self.keyed_db.insert(key.as_slice(), value)
    }

    fn remove(&mut self, index: &u64) -> Result<Option<DBValue>, TreeError> {
        let key = Key::<D>::try_from(index).map_err(TreeError::KeyError)?;
        self.keyed_db.remove(key.as_slice())
    }

    fn verify(
        index: &u64,
        value: &[u8],
        proof: &[DBValue],
        root: &<H as Hasher>::Out,
    ) -> Result<bool, TreeError> {
        let key = Key::<D>::try_from(index).map_err(TreeError::KeyError)?;
        TreeDBMut::<'db, D, H>::verify(key.as_slice(), value, proof, root)
    }
}
