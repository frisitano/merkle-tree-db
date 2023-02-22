use super::{
    ChildSelector, DBValue, HashMap, Hasher, Key, Node, NodeHash, NodeStorage, TreeError, TreeMut,
};
use hash_db::{HashDB, EMPTY_PREFIX};

// TreeDBMutBuilder
// ================================================================================================

/// TreeDBMutBuilder use to build a TreeDBMut
pub struct TreeDBMutBuilder<'db, const D: usize, H: Hasher> {
    db: &'db mut dyn HashDB<H, DBValue>,
    root: &'db mut H::Out,
    // depth: usize,
    // recorder: Option<&'db mut dyn TreeRecorder<H>>,
}

/// Implementation of a TreeDBMutBuilder
impl<'db, const D: usize, H: Hasher> TreeDBMutBuilder<'db, D, H> {
    /// Construct a new db builder
    pub fn new(db: &'db mut dyn HashDB<H, DBValue>, root: &'db mut H::Out) -> Self {
        Self {
            db,
            root,
            // recorder: None,
        }
    }

    /// Consumes the builder and returns a TreeDBMut
    pub fn build(self) -> TreeDBMut<'db, D, H> {
        let root_handle = NodeHash::Database(*self.root);
        TreeDBMut {
            storage: NodeStorage::empty(),
            death_row: HashMap::new(),
            db: self.db,
            root: self.root,
            root_handle,
            null_hashes: null_hashes::<H>(D * 8),
            // recorder: self.recorder.map(core::cell::RefCell::new),
        }
    }
}

// TreeDBMut
// ================================================================================================

/// TreeDBMut use to access and mutate merkle tree from a db backend
pub struct TreeDBMut<'db, const D: usize, H: Hasher> {
    storage: NodeStorage<H>,
    death_row: HashMap<H::Out, usize>,
    db: &'db mut dyn HashDB<H, DBValue>,
    root: &'db mut H::Out,
    root_handle: NodeHash<H>,
    null_hashes: Vec<H::Out>,
    // depth: usize,
    // recorder: Option<core::cell::RefCell<&'db mut dyn TreeRecorder<H>>>,
}

impl<'db, const D: usize, H: Hasher> TreeDBMut<'db, D, H> {
    pub fn commit(&mut self) {}

    fn lookup(
        &self,
        node_hash: &H::Out,
        proof: &mut Option<Vec<Node<H>>>,
    ) -> Result<Node<H>, TreeError> {
        if let Some(node) = self.storage.get(node_hash) {
            return Ok(node.clone());
        }

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
        let mut current_node = self.lookup(self.root_handle.hash(), proof)?;

        for bit in key.iter() {
            let child_hash = current_node.child_hash(&ChildSelector::new(bit))?;
            if child_hash.is_default() {
                return Ok(None);
            }

            current_node = self.lookup(child_hash.hash(), proof)?;
        }

        Ok(Some(current_node))
    }

    fn remove_node(&mut self, node_hash: &NodeHash<H>) {
        match node_hash {
            NodeHash::InMemory(hash) => {
                self.storage.remove(hash);
            }
            NodeHash::Database(hash) => {
                self.death_row
                    .entry(*hash)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
            }
            NodeHash::Default(_) => {}
        }
    }

    /// Inserts a value at the specified key in the tree. New nodes are stored in memory until
    /// the tree is committed. This function recursively traverses the tree until it reaches
    /// the leaf node at the specified key. Old nodes are removed from the tree and replaced
    /// with new nodes.
    fn insert_at(
        &mut self,
        current_hash: &NodeHash<H>,
        key: &Key<D>,
        value: &[u8],
        key_index: u8,
    ) -> Result<(Node<H>, bool), TreeError> {
        // If we have reached the leaf node, create a new leaf node with the specified value.
        if key_index == (D * 8) as u8 {
            let node = Node::new_value(value);

            // If the new node has the same hash as the current node, return the current node
            // as the node has not changed.
            if node.hash() == current_hash.hash() {
                return Ok((node, false));
            }

            if !node.is_default() {
                self.storage.insert(node.clone());
            }

            self.remove_node(current_hash);

            return Ok((node, true));
        }

        // If we have not reached the leaf node lookup the current node.
        let mut current_node = self.lookup(current_hash.hash(), &mut None)?;

        // Select the appropriate child based on the key bit at the current index and lookup.
        let child_selector = ChildSelector::new(key.bit(key_index));
        let child_hash = current_node.child_hash(&child_selector)?;

        let (child_node, child_changed) = self.insert_at(child_hash, key, value, key_index + 1)?;

        if !child_changed {
            return Ok((current_node, false));
        }

        println!("new child node: {:?}", child_node.hash());
        let child_hash: NodeHash<H> = if child_node.is_default() {
            NodeHash::Default(*child_node.hash())
        } else {
            NodeHash::InMemory(*child_node.hash())
        };
        current_node.set_child_hash(&child_selector, child_hash)?;

        if !current_node.is_default() {
            self.storage.insert(current_node.clone());
        }
        self.remove_node(current_hash);

        Ok((current_node, true))
    }

    pub fn print(&self) {
        // print the root
        println!("root: {:?}", self.root);
        // print the root_handle
        println!("root_handle: {:?}", self.root_handle.hash());
        // print the storage
        println!("storage {}", self.storage.iter().count());
        // print the death_row
        println!("death_row: {:?}", self.death_row);
        // print the db
        // println!("db: {:?}", self.db);
    }
}

/// Implementation of a TreeDBMut
impl<'db, const N: usize, H: Hasher> TreeMut<H, N> for TreeDBMut<'db, N, H> {
    /// Return the root of the tree
    fn root(&mut self) -> &H::Out {
        self.commit();
        self.root()
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

    fn insert(&mut self, key: &Key<N>, value: DBValue) -> Result<(), TreeError> {
        let current_root = self.root_handle.clone();
        let (new_root, changed) = self.insert_at(&current_root, key, &value, 0)?;

        if changed {
            self.remove_node(&current_root);
            self.root_handle = NodeHash::InMemory(*new_root.hash());
            self.storage.insert(new_root);
        }

        Ok(())
    }

    fn remove(&mut self, key: &crate::Key<N>) -> Result<(), crate::TreeError> {
        todo!()
    }
}

// Helpers
// ================================================================================================

/// Return the null hashes of a tree of depth D
pub fn null_hashes<H: Hasher>(depth: usize) -> Vec<H::Out> {
    let mut hashes = Vec::with_capacity(depth);
    hashes.push(H::hash(&[]));
    for i in 1..depth {
        let hash = H::hash(&[hashes[i - 1].as_ref(), hashes[i - 1].as_ref()].concat());
        hashes.push(hash);
    }
    hashes
}
