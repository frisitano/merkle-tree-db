use super::{DBValue, Hasher, IndexTreeMut, Key, KeyedTreeMut, TreeDBMut, TreeError};

/// A TreeDBMut that uses a u64 index to access the underlying database.
/// Wraps a TreeDB and converts a u64 index to a Key of the appropriate depth to access
/// the underlying TreeDB.
pub struct IndexTreeDBMut<'db, const D: usize, H: Hasher> {
    keyed_db: TreeDBMut<'db, D, H>,
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
