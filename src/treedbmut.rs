use super::{
    null_nodes, ChildSelector, DBValue, DataError, HashMap, Hasher, Key, KeyedTreeMut, Node,
    NodeHash, NodeStorage, TreeError, TreeRecorder,
};
use core::cmp::Ordering;
use hash_db::{HashDB, EMPTY_PREFIX};

// TreeDBMutBuilder
// ================================================================================================

/// TreeDBMutBuilder use to build a TreeDBMut
pub struct TreeDBMutBuilder<'db, const D: usize, H: Hasher> {
    db: &'db mut dyn HashDB<H, DBValue>,
    root: &'db mut H::Out,
    // depth: usize,
    recorder: Option<&'db mut dyn TreeRecorder<H>>,
}

/// Implementation of a TreeDBMutBuilder
impl<'db, const D: usize, H: Hasher> TreeDBMutBuilder<'db, D, H> {
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

    /// Consumes the builder and returns a TreeDBMut
    pub fn build(self) -> TreeDBMut<'db, D, H> {
        let root_handle = NodeHash::Database(*self.root);
        TreeDBMut {
            storage: NodeStorage::empty(),
            death_row: HashMap::new(),
            db: self.db,
            root: self.root,
            root_handle,
            null_nodes: null_nodes::<H>(D * 8),
            recorder: self.recorder.map(core::cell::RefCell::new),
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
    null_nodes: HashMap<H::Out, Node<H>>,
    // depth: usize,
    recorder: Option<core::cell::RefCell<&'db mut dyn TreeRecorder<H>>>,
}

impl<'db, const D: usize, H: Hasher> TreeDBMut<'db, D, H> {
    /// commit the changes of insertions held in storage and removes held in death row to the db
    pub fn commit(&mut self) {
        // iterate over storage and check if the node is in death row
        for (key, (node, insert_count)) in self.storage.drain() {
            // check if the node is in death row
            match self.death_row.remove(&key) {
                Some(death_count) => {
                    // compare the death count with the insert count
                    match insert_count.cmp(&death_count) {
                        // if they are the same do nothing
                        Ordering::Equal => {}
                        // if the count is greater than 0, insert the node to db
                        Ordering::Greater => {
                            for _ in 0..insert_count - death_count {
                                self.db.emplace(key, EMPTY_PREFIX, node.clone().into());
                            }
                        }
                        // if the count is less than 0, delete the node from db
                        Ordering::Less => {
                            for _ in 0..death_count - insert_count {
                                self.db.remove(&key, EMPTY_PREFIX);
                            }
                        }
                    }
                }
                // if the node is not in death row, insert the node to db count times
                None => {
                    for _ in 0..insert_count {
                        self.db.emplace(key, EMPTY_PREFIX, node.clone().into());
                    }
                }
            }
        }

        for (key, count) in self.death_row.drain() {
            for _ in 0..count {
                self.db.remove(&key, EMPTY_PREFIX);
            }
        }

        *self.root = *self.root_handle.hash();
        self.root_handle = NodeHash::Database(*self.root);
    }

    fn lookup(&self, node_hash: &NodeHash<H>) -> Result<Node<H>, TreeError> {
        let node = match node_hash {
            NodeHash::InMemory(hash) => self.storage.get(hash).cloned().ok_or(
                TreeError::DataError(DataError::InMemoryDataNotFound(hash.as_ref().to_vec())),
            ),
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
        let mut current_node = self.lookup(&self.root_handle)?;

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
        key_index: usize,
    ) -> Result<(Node<H>, Option<DBValue>, bool), TreeError> {
        // If we have reached the leaf node, create a new leaf node with the specified value.
        if key_index == D * 8 {
            let node = Node::new_value(value);

            // fetch the old node if it exists
            let old_node = match current_hash {
                NodeHash::InMemory(_) | NodeHash::Database(_) => Some(
                    self.lookup(current_hash)?
                        .value()
                        .map_err(TreeError::NodeError)?
                        .clone(),
                ),
                NodeHash::Default(_) => None,
            };

            // If the new node has the same hash as the current node, return the current node
            // as the node has not changed.
            if node.hash() == current_hash.hash() {
                return Ok((node, old_node, false));
            }

            if !node.is_default() {
                self.storage.insert(node.clone());
            }

            self.remove_node(current_hash);

            return Ok((node, old_node, true));
        }

        // If we have not reached the leaf node lookup the current node.
        let mut current_node = self.lookup(current_hash)?;

        // Select the appropriate child based on the key bit at the current index and lookup.
        let bit = key.bit(key_index).map_err(TreeError::KeyError)?;
        let child_selector = ChildSelector::new(bit);
        let child_hash = current_node
            .child_hash(&child_selector)
            .map_err(TreeError::NodeError)?;

        let (child_node, old_node, changed) =
            self.insert_at(child_hash, key, value, key_index + 1)?;

        if !changed {
            return Ok((current_node, old_node, false));
        }

        let child_hash: NodeHash<H> = if child_node.is_default() {
            NodeHash::Default(*child_node.hash())
        } else {
            NodeHash::InMemory(*child_node.hash())
        };
        current_node
            .set_child_hash(&child_selector, child_hash)
            .map_err(TreeError::NodeError)?;

        if !current_node.is_default() {
            self.storage.insert(current_node.clone());
        }
        self.remove_node(current_hash);

        Ok((current_node, old_node, true))
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
    }
}

/// Implementation of a TreeDBMut
impl<'db, const D: usize, H: Hasher> KeyedTreeMut<H, D> for TreeDBMut<'db, D, H> {
    /// Return the root of the tree
    fn root(&mut self) -> &H::Out {
        self.commit();
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

    fn insert(&mut self, key: &[u8], value: DBValue) -> Result<Option<DBValue>, TreeError> {
        let key = Key::<D>::new(key).map_err(TreeError::KeyError)?;
        let current_root = self.root_handle.clone();
        let (new_root, old_node, changed) = self.insert_at(&current_root, &key, &value, 0)?;

        if changed {
            self.remove_node(&current_root);
            self.root_handle = NodeHash::InMemory(*new_root.hash());
            self.storage.insert(new_root);
        }

        Ok(old_node)
    }

    fn remove(&mut self, key: &[u8]) -> Result<Option<DBValue>, TreeError> {
        self.insert(key, vec![])
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
