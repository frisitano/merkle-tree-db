use super::{DBValue, Hasher, NodeError};
use core::ops::Deref;

// NodeHash
// ================================================================================================

/// NodeHash is used to store the hash of a node
/// If the node is stored in memory, the hash is stored in the InMemory variant
/// If the node is stored in database backend, the hash is stored in the Database variant
/// If the node is a default node, the hash is stored in the Default variant
#[derive(PartialEq, Eq, Hash)]
pub enum NodeHash<H: Hasher> {
    /// Hash associated with a node stored in memory
    InMemory(H::Out),
    /// Hash associated with a node stored in database backend
    Database(H::Out),
    /// Hash associated with a default node
    Default(H::Out),
}

impl<H: Hasher> core::fmt::Display for NodeHash<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeHash::InMemory(hash) => write!(f, "InMemory({hash:?})"),
            NodeHash::Database(hash) => write!(f, "Database({hash:?})"),
            NodeHash::Default(hash) => write!(f, "Default({hash:?})"),
        }
    }
}

impl<H: Hasher> NodeHash<H> {
    /// Returns the inner hash of a node
    pub fn hash(&self) -> &H::Out {
        match self {
            NodeHash::InMemory(hash) => hash,
            NodeHash::Database(hash) => hash,
            NodeHash::Default(hash) => hash,
        }
    }

    /// Returns true if the node is a default node
    pub fn is_default(&self) -> bool {
        matches!(self, NodeHash::Default(_))
    }
}

/// Implementation of Clone for NodeHash
impl<H: Hasher> Clone for NodeHash<H> {
    fn clone(&self) -> Self {
        match self {
            NodeHash::Database(hash) => NodeHash::Database(*hash),
            NodeHash::InMemory(hash) => NodeHash::InMemory(*hash),
            NodeHash::Default(hash) => NodeHash::Default(*hash),
        }
    }
}

/// Implement default for NodeHash
impl<H: Hasher> Default for NodeHash<H> {
    fn default() -> Self {
        NodeHash::Default(H::Out::default())
    }
}

/// Implement Deref for NodeHash
impl<H: Hasher> Deref for NodeHash<H> {
    type Target = H::Out;

    fn deref(&self) -> &Self::Target {
        self.hash()
    }
}

// Node
// ================================================================================================

/// ChildSelector is used to specify a child node of an inner node
pub enum ChildSelector {
    Left,
    Right,
}

impl ChildSelector {
    /// Constructs a ChildSelector from the provided bool. If the bool is false then Left is returned,
    /// if the bool is true then Right is returned.
    pub fn new(child: bool) -> Self {
        if child {
            ChildSelector::Right
        } else {
            ChildSelector::Left
        }
    }

    /// Returns a ChildSelector that represents the sibling of the current ChildSelector
    pub fn sibling(&self) -> Self {
        match self {
            ChildSelector::Left => ChildSelector::Right,
            ChildSelector::Right => ChildSelector::Left,
        }
    }
}

/// Node is used to store the data of a node. A value node stores the value and leaf hash. An inner
/// node stores the left child hash and right child hash.
#[derive(PartialEq, Eq)]
pub enum Node<H: Hasher> {
    Value {
        hash: H::Out,
        value: DBValue,
    },
    Inner {
        hash: H::Out,
        left: NodeHash<H>,
        right: NodeHash<H>,
    },
}

impl<H: Hasher> std::fmt::Display for Node<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Value { hash, value } => write!(f, "Value({hash:?}, {value:?})"),
            Node::Inner { hash, left, right } => write!(f, "Inner({hash:?}, {left}, {right})"),
        }
    }
}

/// Node implementation
impl<H: Hasher> Node<H> {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// constructs a new value node
    pub fn new_value(value: &[u8]) -> Self {
        Node::Value {
            hash: H::hash(value),
            value: value.to_vec(),
        }
    }

    /// Constructs a new inner node
    pub fn new_inner(left: NodeHash<H>, right: NodeHash<H>) -> Result<Self, NodeError> {
        // if both left and right are default hashes that do not match, return an error
        if matches!(
            (&left, &right),
            (NodeHash::Default(_), NodeHash::Default(_))
        ) && left.hash() != right.hash()
        {
            return Err(NodeError::InconsistentDefaultHashes);
        }

        let hash = H::hash(&[left.hash().as_ref(), right.hash().as_ref()].concat());

        Ok(Node::Inner { hash, left, right })
    }

    // ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns a reference to the specified child hash of an inner node.  This accessor is only
    /// valid for inner nodes.
    /// Errors:
    /// - UnexpectedNodeType: if the node is a value node
    pub fn child_hash(&self, child: &ChildSelector) -> Result<&NodeHash<H>, NodeError> {
        match self {
            Node::Value { hash: _, value: _ } => Err(NodeError::InvalidNodeType(
                "Value".to_string(),
                "Inner".to_string(),
            )),
            Node::Inner {
                hash: _,
                left,
                right,
            } => match child {
                ChildSelector::Left => Ok(left),
                ChildSelector::Right => Ok(right),
            },
        }
    }

    /// Returns a reference to the value of a value node.  This accessor is only valid for value
    /// nodes.
    /// Errors:
    /// - UnexpectedNodeType: if the node is an inner node
    pub fn value(&self) -> Result<&DBValue, NodeError> {
        match self {
            Node::Value { hash: _, value } => Ok(value),
            Node::Inner {
                hash: _,
                left: _,
                right: _,
            } => Err(NodeError::InvalidNodeType(
                "Inner".to_string(),
                "Value".to_string(),
            )),
        }
    }

    /// Returns a reference to the hash of a node. This accessor is valid for both value and inner
    /// nodes.
    pub fn hash(&self) -> &H::Out {
        match self {
            Node::Value { hash, value: _ } => hash,
            Node::Inner {
                hash,
                left: _,
                right: _,
            } => hash,
        }
    }

    /// Returns true if both children are default hashes
    /// Errors:
    /// - UnexpectedNodeType: if the node is a value node
    pub fn is_default(&self) -> bool {
        match self {
            Node::Value { hash: _, value } => value.is_empty(),
            Node::Inner {
                hash: _,
                left,
                right,
            } => matches!((left, right), (NodeHash::Default(_), NodeHash::Default(_))),
        }
    }

    // MODIFIERS
    // --------------------------------------------------------------------------------------------
    /// Sets the specified child hash of an inner node.  This modifier is only valid for inner node.
    pub fn set_child_hash(
        &mut self,
        child: &ChildSelector,
        child_hash: NodeHash<H>,
    ) -> Result<(), NodeError> {
        match self {
            Node::Value { hash: _, value: _ } => Err(NodeError::InvalidNodeType(
                "Value".to_string(),
                "Inner".to_string(),
            )),
            Node::Inner { hash, left, right } => match child {
                ChildSelector::Left => {
                    *left = child_hash;
                    *hash = H::hash(&[left.hash().as_ref(), right.hash().as_ref()].concat());
                    Ok(())
                }
                ChildSelector::Right => {
                    *right = child_hash;
                    *hash = H::hash(&[left.hash().as_ref(), right.hash().as_ref()].concat());
                    Ok(())
                }
            },
        }
    }
}

/// Returns a clone of the node
impl<H: Hasher> Clone for Node<H> {
    fn clone(&self) -> Self {
        match self {
            Node::Value { hash, value } => Node::Value {
                hash: *hash,
                value: value.clone(),
            },
            Node::Inner { hash, left, right } => Node::Inner {
                hash: *hash,
                left: left.clone(),
                right: right.clone(),
            },
        }
    }
}

/// Implements default for Node
impl<H: Hasher> Default for Node<H> {
    fn default() -> Self {
        Node::Value {
            hash: H::Out::default(),
            value: DBValue::default(),
        }
    }
}

// Node Serialization
// ================================================================================================

/// Serialize a node to a vector of bytes. A value node is prefixed with a 0. Inner nodes are
/// prefixed as follows:
/// 0 - Value node
/// 1 - Inner node with both children
/// 2 - Inner node with left child and default right child
/// 3 - Inner node with right child and default left child
impl<H: Hasher> From<Node<H>> for Vec<u8> {
    fn from(node: Node<H>) -> Self {
        match node {
            Node::Value { hash: _, value } => {
                let mut bytes = vec![0];
                bytes.extend_from_slice(&value);
                bytes
            }
            Node::Inner {
                hash: _,
                left,
                right,
            } => {
                let mut bytes = vec![];
                match (&left, &right) {
                    // if the left child is default value then push 2
                    (_, NodeHash::Default(_)) => {
                        bytes.push(2);
                    }
                    // if the right child is default value then push 3
                    (NodeHash::Default(_), _) => {
                        bytes.push(3);
                    }
                    // else push 1
                    _ => {
                        bytes.push(1);
                    }
                }
                bytes.extend_from_slice(left.hash().as_ref());
                bytes.extend_from_slice(right.hash().as_ref());
                bytes
            }
        }
    }
}

/// Deserialize a node from a vector of bytes. The first byte of the vector is used to determine the
/// type of node. A value node is prefixed with a 0. Inner nodes are prefixed as follows:
/// 1 - Inner node with both children
/// 2 - Inner node with left child and default right child
/// 3 - Inner node with right child and default left child
impl<H: Hasher> TryFrom<Vec<u8>> for Node<H> {
    type Error = NodeError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match value.first() {
            // Construct Value node
            Some(0) => {
                if value.len() == 1 {
                    return Err(NodeError::DecodeNodeEmptyValue);
                }

                Ok(Node::new_value(&value[1..]))
            }
            // Construct Inner node when both children are not default
            Some(inner_node_type) => {
                // Length of byte vector should be 2 * H::Length + 1
                let expected_length = 2 * H::LENGTH + 1;
                let actual_length = value.len();
                if actual_length != expected_length {
                    return Err(NodeError::DecodeNodeInvalidLength(
                        value.len(),
                        2 * H::LENGTH + 1,
                    ));
                }

                // Decode and construct inner node
                let left_hash = decode_hash::<H>(&value[1..1 + H::LENGTH])?;
                let right_hash = decode_hash::<H>(&value[1 + H::LENGTH..])?;
                match inner_node_type {
                    1 => Node::new_inner(
                        NodeHash::Database(left_hash),
                        NodeHash::Database(right_hash),
                    ),
                    2 => Node::new_inner(
                        NodeHash::Database(left_hash),
                        NodeHash::Default(right_hash),
                    ),
                    3 => Node::new_inner(
                        NodeHash::Default(left_hash),
                        NodeHash::Database(right_hash),
                    ),
                    _ => Err(NodeError::DecodeNodeInvalidPrefix(*inner_node_type)),
                }
            }
            _ => Err(NodeError::DecodeNodeNoData),
        }
    }
}

// HELPERS
// ================================================================================================

/// Decode a hash from a byte vector. The byte vector must be exactly H::LENGTH bytes long.
///
/// Errors:
/// - DecodeHashFailed: if the byte vector is not exactly H::LENGTH bytes long
fn decode_hash<H: Hasher>(data: &[u8]) -> Result<H::Out, NodeError> {
    if data.len() != H::LENGTH {
        return Err(NodeError::DecodeNodeHashFailed(data.to_vec()));
    }
    let mut hash = H::Out::default();
    hash.as_mut().copy_from_slice(data);
    Ok(hash)
}
