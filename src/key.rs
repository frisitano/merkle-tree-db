use super::KeyError;

// CONSTANTS
// ================================================================================================

/// The number of bits in a byte
const BYTE_SIZE: usize = 8;

// IMPLEMENTATION
// ================================================================================================

/// stores a key of N bytes
#[derive(PartialEq)]
pub struct Key<const N: usize>([u8; N]);

impl<const N: usize> Key<N> {
    /// Create a new key from a byte slice
    pub fn new(key: &[u8]) -> Result<Key<N>, KeyError> {
        let key = key
            .try_into()
            .map_err(|_| KeyError::IncorrectKeySize(N, key.len()))?;
        Ok(Key(key))
    }

    /// Create a zero key
    pub const fn zero() -> Self {
        Self([0u8; N])
    }

    /// Returns true if the key is the zero key, false otherwise
    pub fn is_zero(&self) -> bool {
        self == &Key::zero()
    }

    /// Returns the bit at the i'th index of the key
    pub fn bit(&self, i: usize) -> Result<bool, KeyError> {
        let byte_pos = i / BYTE_SIZE;
        if byte_pos >= N {
            return Err(KeyError::BitIndexOutOfBounds(i, N * 8));
        }

        let bit_pos = i % BYTE_SIZE;
        let bit = (self.0[byte_pos] >> (7 - bit_pos)) & 1;
        Ok(bit != 0)
    }

    /// Returns the key as a byte slice
    pub fn as_slice(&self) -> &[u8] {
        &self.0[..]
    }

    /// Returns an iterator over the key
    pub fn iter(&self) -> KeyIter<'_, N> {
        KeyIter {
            key: self,
            element: 0,
        }
    }

    /// Returns an iterator over the key from the i'th index
    pub fn iter_from(&self, i: usize) -> KeyIter<'_, N> {
        KeyIter {
            key: self,
            element: i,
        }
    }
}

/// Key iterator
pub struct KeyIter<'a, const N: usize> {
    key: &'a Key<N>,
    element: usize,
}

/// Key iterator implementation
impl<'a, const N: usize> Iterator for KeyIter<'a, N> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.element >= N * 8 {
            return None;
        }

        let result = self.key.bit(self.element).expect("element is checked");
        self.element += 1;

        Some(result)
    }
}

impl<const D: usize> AsRef<[u8]> for Key<D> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<const D: usize> TryFrom<&u64> for Key<D> {
    type Error = KeyError;

    fn try_from(value: &u64) -> Result<Self, Self::Error> {
        let max = 2u64.pow(D as u32 * 8);
        if value > &max {
            return Err(KeyError::LeafIndexOutOfBounds(*value, max));
        }

        let mut key = [0u8; D];
        key.copy_from_slice(&value.to_be_bytes()[..D]);
        Ok(Key(key))
    }
}
