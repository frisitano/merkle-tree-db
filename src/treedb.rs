use hash_db::{HashDBRef, EMPTY_PREFIX};

use super::{
    null_nodes, ChildSelector, DBValue, DataError, HashMap, Hasher, Key, KeyedTree, Node, NodeHash,
    TreeError, TreeRecorder,
};

// TreeDBBuilder
// ================================================================================================

/// Used to construct a TreeDB
pub struct TreeDBBuilder<'db, const D: usize, H: Hasher> {
    db: &'db dyn HashDBRef<H, DBValue>,
    root: &'db H::Out,
    recorder: Option<&'db mut dyn TreeRecorder<H>>,
}

/// Implementation of the TreeDBBuilder
impl<'db, const D: usize, H: Hasher> TreeDBBuilder<'db, D, H> {
    /// Construct a new db builder
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

    /// build a TreeDB
    pub fn build(self) -> TreeDB<'db, D, H> {
        TreeDB {
            db: self.db,
            root: self.root,
            recorder: self.recorder.map(core::cell::RefCell::new),
            null_nodes: null_nodes::<H>(D * 8),
        }
    }
}

// TreeDB
// ================================================================================================

/// TreeDB use to access binary merkle tree from a db backend
pub struct TreeDB<'db, const D: usize, H: Hasher> {
    db: &'db dyn HashDBRef<H, DBValue>,
    root: &'db H::Out,
    null_nodes: HashMap<H::Out, Node<H>>,
    // depth: usize,
    recorder: Option<core::cell::RefCell<&'db mut dyn TreeRecorder<H>>>,
}

/// Implementation of a TreeDB
impl<'db, const D: usize, H: Hasher> TreeDB<'db, D, H> {
    /// Return the db of a TreeDB
    pub fn db(&self) -> &dyn HashDBRef<H, DBValue> {
        self.db
    }

    fn lookup(&self, node_hash: &NodeHash<H>) -> Result<Node<H>, TreeError> {
        let node = match node_hash {
            NodeHash::InMemory(_) => {
                return Err(TreeError::DataError(DataError::InMemoryNotSupported))
            }
            NodeHash::Database(hash) => {
                let data = self.db.get(hash, EMPTY_PREFIX).ok_or(TreeError::DataError(
                    DataError::DatabaseDataNotFound(hash.as_ref().to_vec()),
                ))?;
                let node: Node<H> = data.try_into().map_err(TreeError::NodeError)?;

                if let Some(recorder) = self.recorder.as_ref() {
                    recorder.borrow_mut().record(&node);
                }

                Ok(node)
            }
            NodeHash::Default(hash) => {
                self.null_nodes
                    .get(hash)
                    .cloned()
                    .ok_or(TreeError::DataError(DataError::NullNodeDataNotFound(
                        hash.as_ref().to_vec(),
                    )))
            }
        }?;

        Ok(node)
    }

    fn lookup_leaf_node(
        &self,
        key: &Key<D>,
        proof: &mut Option<Vec<DBValue>>,
    ) -> Result<Option<Node<H>>, TreeError> {
        let mut current_node = self.lookup(&NodeHash::<H>::Database(*self.root))?;

        for bit in key.iter() {
            let child_selector = ChildSelector::new(bit);
            let child_hash = current_node
                .child_hash(&child_selector)
                .map_err(TreeError::NodeError)?;
            if child_hash.is_default() && proof.is_none() {
                return Ok(None);
            }

            // store the sibling hash in the proof
            if let Some(proof) = proof.as_mut() {
                let sibling_hash: H::Out = **current_node
                    .child_hash(&child_selector.sibling())
                    .map_err(TreeError::NodeError)?;
                proof.push(sibling_hash.as_ref().to_vec());
            }

            current_node = self.lookup(child_hash)?;
        }

        Ok(Some(current_node))
    }
}

/// Tree implementation for TreeDB
impl<'db, H: Hasher, const D: usize> KeyedTree<H, D> for TreeDB<'db, D, H> {
    /// Returns a reference to the root hash
    fn root(&self) -> &H::Out {
        self.root
    }

    /// Iterates through the key and fetches the specified child hash for each inner
    /// node until we reach the leaf node. Returns the value associated with the leaf node.
    fn value(&self, key: &[u8]) -> Result<Option<DBValue>, TreeError> {
        let key = Key::<D>::new(key).map_err(TreeError::KeyError)?;
        let node = self.lookup_leaf_node(&key, &mut None)?;
        match node {
            Some(node) => Ok(Some(node.value().map_err(TreeError::NodeError)?.clone())),
            None => Ok(None),
        }
    }

    fn leaf(&self, key: &[u8]) -> Result<Option<H::Out>, TreeError> {
        let key = Key::<D>::new(key).map_err(TreeError::KeyError)?;
        let node = self.lookup_leaf_node(&key, &mut None)?;
        match node {
            Some(node) => Ok(Some(*node.hash())),
            None => Ok(None),
        }
    }

    fn proof(&self, key: &[u8]) -> Result<Option<Vec<DBValue>>, TreeError> {
        let key = Key::<D>::new(key).map_err(TreeError::KeyError)?;
        let mut proof = Some(Vec::new());
        match self.lookup_leaf_node(&key, &mut proof)? {
            Some(_) => {
                let mut proof = proof.unwrap();
                proof.reverse();
                Ok(Some(proof))
            }
            None => Ok(None),
        }
    }

    fn verify(
        key: &[u8],
        value: &[u8],
        proof: &[DBValue],
        root: &H::Out,
    ) -> Result<bool, TreeError> {
        let key = Key::<D>::new(key).map_err(TreeError::KeyError)?;
        let mut hash = H::hash(value);
        // iterate over the bits in the key in reverse order
        for (bit, sibling) in (0..D * 8).rev().zip(proof.iter()) {
            let bit = key.bit(bit).map_err(TreeError::KeyError)?;
            let child_selector = ChildSelector::new(bit);
            match child_selector {
                ChildSelector::Left => {
                    hash = H::hash(&[hash.as_ref(), sibling].concat());
                }
                ChildSelector::Right => {
                    hash = H::hash(&[sibling, hash.as_ref()].concat());
                }
            }
        }
        Ok(hash == *root)
    }
}
