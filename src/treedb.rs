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

impl<'db, const D: usize, H: Hasher> TreeDBBuilder<'db, D, H> {
    /// Construct a new TreeDBBuilder
    pub fn new(db: &'db dyn HashDBRef<H, DBValue>, root: &'db H::Out) -> Result<Self, TreeError> {
        //TODO: warm user if default root provided
        if D > usize::MAX / 8 {
            return Err(TreeError::DepthTooLarge(D, usize::MAX / 8));
        }
        Ok(Self {
            db,
            root,
            recorder: None,
        })
    }

    /// Add a recorder to the TreeDBBuilder
    pub fn with_recorder(mut self, recorder: &'db mut dyn TreeRecorder<H>) -> Self {
        self.recorder = Some(recorder);
        self
    }

    /// Add an optional recorder to the TreeDBBuilder
    pub fn with_optional_recorder<'recorder: 'db>(
        mut self,
        recorder: Option<&'recorder mut dyn TreeRecorder<H>>,
    ) -> Self {
        self.recorder = recorder.map(|r| r as _);
        self
    }

    /// build a TreeDB
    pub fn build(self) -> TreeDB<'db, D, H> {
        let (null_nodes, default_root) = null_nodes::<H>(D * 8);
        let root = if self.root == &H::Out::default() || self.root == &default_root {
            NodeHash::Default(default_root)
        } else {
            NodeHash::Database(*self.root)
        };
        TreeDB {
            db: self.db,
            root,
            recorder: self.recorder.map(core::cell::RefCell::new),
            null_nodes,
        }
    }
}

// TreeDB
// ================================================================================================

/// An immutable merkle tree db that uses a byte slice key to specify the leaves in the tree.
pub struct TreeDB<'db, const D: usize, H: Hasher> {
    db: &'db dyn HashDBRef<H, DBValue>,
    root: NodeHash<H>,
    null_nodes: HashMap<H::Out, Node<H>>,
    recorder: Option<core::cell::RefCell<&'db mut dyn TreeRecorder<H>>>,
}

impl<'db, const D: usize, H: Hasher> TreeDB<'db, D, H> {
    /// Return the underlying db of a TreeDB
    pub fn db(&self) -> &dyn HashDBRef<H, DBValue> {
        self.db
    }

    /// Return the node associated with the provided hash. Retrieves the node from either the database
    /// or the null node map if it is a default node.
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

    /// Returns a leaf node for the provided key. If the leaf node does not exist, returns None.
    /// If a proof is provided, the sibling hashes along the lookup path are stored in the proof.
    fn lookup_leaf_node(
        &self,
        key: &Key<D>,
        proof: &mut Option<Vec<DBValue>>,
    ) -> Result<Option<Node<H>>, TreeError> {
        let mut current_node = self.lookup(&self.root)?;

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

impl<'db, H: Hasher, const D: usize> KeyedTree<H, D> for TreeDB<'db, D, H> {
    /// Returns the root of the tree
    fn root(&self) -> &H::Out {
        &self.root
    }

    /// Returns the value associated with the given key
    fn value(&self, key: &[u8]) -> Result<Option<DBValue>, TreeError> {
        let key = Key::<D>::new(key).map_err(TreeError::KeyError)?;
        let node = self.lookup_leaf_node(&key, &mut None)?;
        match node {
            Some(node) => Ok(Some(node.value().map_err(TreeError::NodeError)?.clone())),
            None => Ok(None),
        }
    }

    /// Returns the leaf associated with the given key
    fn leaf(&self, key: &[u8]) -> Result<Option<H::Out>, TreeError> {
        let key = Key::<D>::new(key).map_err(TreeError::KeyError)?;
        let node = self.lookup_leaf_node(&key, &mut None)?;
        match node {
            Some(node) => Ok(Some(*node.hash())),
            None => Ok(None),
        }
    }

    /// Returns a proof that a value exists in the tree at the given key
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

    /// Verifies that the given value is in the tree with the given root at the given index
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
