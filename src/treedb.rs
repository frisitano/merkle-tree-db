use hash_db::{HashDBRef, EMPTY_PREFIX};

use super::{ChildSelector, DBValue, Hasher, Key, Node, Tree, TreeError};

// TreeDBBuilder
// ================================================================================================

/// Used to construct a TreeDB
pub struct TreeDBBuilder<'db, const D: usize, H: Hasher> {
    db: &'db dyn HashDBRef<H, DBValue>,
    root: &'db H::Out,
    // recorder: Option<&'db mut dyn TreeRecorder<H>>,
}

/// Implementation of the TreeDBBuilder
impl<'db, const D: usize, H: Hasher> TreeDBBuilder<'db, D, H> {
    /// Construct a new db builder
    pub fn new(db: &'db dyn HashDBRef<H, DBValue>, root: &'db H::Out) -> Self {
        Self {
            db,
            root,
            // recorder: None,
        }
    }

    /// Add a recorder to the db buidler
    // pub fn with_recorder<'recorder: 'db>(
    //     mut self,
    //     recorder: &'recorder mut dyn TreeRecorder<H>,
    // ) -> Self {
    //     self.recorder = Some(recorder);
    //     self
    // }

    /// Add an optional recorder to the db builder
    // pub fn with_optional_recorder<'recorder: 'db>(
    //     mut self,
    //     recorder: Option<&'recorder mut dyn TreeRecorder<H>>,
    // ) -> Self {
    //     self.recorder = recorder.map(|r| r as _);
    //     self
    // }

    /// build a TreeDB
    pub fn build(self) -> TreeDB<'db, D, H> {
        TreeDB {
            db: self.db,
            root: self.root,
            // recorder: self.recorder.map(core::cell::RefCell::new),
        }
    }
}

// TreeDB
// ================================================================================================

/// TreeDB use to access binary merkle tree from a db backend
pub struct TreeDB<'db, const D: usize, H: Hasher> {
    db: &'db dyn HashDBRef<H, DBValue>,
    root: &'db H::Out,
    // depth: usize,
    // recorder: Option<core::cell::RefCell<&'db mut dyn TreeRecorder<H>>>,
}

/// Implementation of a TreeDB
impl<'db, const D: usize, H: Hasher> TreeDB<'db, D, H> {
    /// Return the db of a TreeDB
    pub fn db(&self) -> &dyn HashDBRef<H, DBValue> {
        self.db
    }

    fn lookup(
        &self,
        node_hash: &H::Out,
        proof: &mut Option<Vec<Node<H>>>,
    ) -> Result<Node<H>, TreeError> {
        let data = self
            .db
            .get(node_hash, EMPTY_PREFIX)
            .ok_or(TreeError::DataNotFound)?;
        let node: Node<H> = data.try_into()?;

        if let Some(proof) = proof.as_mut() {
            proof.push(node.clone());
        }

        Ok(node)
    }

    fn lookup_leaf_node(
        &self,
        key: &Key<D>,
        proof: &mut Option<Vec<Node<H>>>,
    ) -> Result<Option<Node<H>>, TreeError> {
        let mut current_node = self.lookup(self.root, proof)?;

        for bit in key.iter() {
            let child_hash = current_node.child_hash(&ChildSelector::new(bit))?;
            if child_hash.is_default() {
                return Ok(None);
            }

            current_node = self.lookup(child_hash.hash(), proof)?;
        }

        Ok(Some(current_node))
    }
}

/// Tree implementation for TreeDB
impl<'db, H: Hasher, const N: usize> Tree<H, N> for TreeDB<'db, N, H> {
    /// Returns a reference to the root hash
    fn root(&self) -> &H::Out {
        self.root
    }

    /// Iterates through the key and fetches the specified child hash for each inner
    /// node until we reach the leaf node. Returns the value associated with the leaf node.
    fn value(&self, key: &Key<N>) -> Result<Option<DBValue>, TreeError> {
        let node = self.lookup_leaf_node(key, &mut None)?;
        match node {
            Some(node) => Ok(Some(node.value()?.clone())),
            None => Ok(None),
        }
    }

    fn leaf(&self, key: &Key<N>) -> Result<Option<H::Out>, TreeError> {
        let node = self.lookup_leaf_node(key, &mut None)?;
        match node {
            Some(node) => Ok(Some(*node.hash())),
            None => Ok(None),
        }
    }

    fn proof(&self, key: &Key<N>) -> Result<Option<Vec<Node<H>>>, TreeError> {
        let mut proof = Some(Vec::new());
        match self.lookup_leaf_node(key, &mut proof)? {
            Some(_) => Ok(Some(proof.unwrap())),
            None => Ok(None),
        }
    }
}
