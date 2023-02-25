use super::{DBValue, Hasher, IndexTree, Key, KeyedTree, TreeDB, TreeError};

/// A TreeDB that uses a u64 index to access the underlying database.
/// Wraps a TreeDB and converts a u64 index to a Key of the appropriate depth to access
/// the underlying TreeDB.
pub struct IndexedTreeDB<'db, const D: usize, H: Hasher> {
    keyed_db: TreeDB<'db, D, H>,
}

impl<'db, H: Hasher + 'db, const D: usize> IndexTree<H, D> for IndexedTreeDB<'db, D, H> {
    fn root(&self) -> &<H as Hasher>::Out {
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
