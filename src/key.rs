// CONSTANTS
// ================================================================================================

/// The number of bits in a byte
const BYTE_SIZE: u8 = 8;

// IMPLEMENTATION
// ================================================================================================

/// stores a key of N bytes
#[derive(PartialEq)]
pub struct Key<const N: usize>([u8; N]);

impl<const N: usize> Key<N> {
    /// Create a new key from a byte slice
    pub fn new(key: &[u8; N]) -> Key<N> {
        Key(*key)
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
    pub fn bit(&self, i: u8) -> bool {
        let byte_pos = i / BYTE_SIZE;
        let bit_pos = i % BYTE_SIZE;
        let bit = self.0[byte_pos as usize] >> (7 - bit_pos) & 1;
        bit != 0
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
    pub fn iter_from(&self, i: u8) -> KeyIter<'_, N> {
        KeyIter {
            key: self,
            element: i,
        }
    }
}

/// Key iterator
pub struct KeyIter<'a, const N: usize> {
    key: &'a Key<N>,
    element: u8,
}

/// Key iterator implementation
impl<'a, const N: usize> Iterator for KeyIter<'a, N> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.element >= N as u8 * 8 {
            return None;
        }

        let result = self.key.bit(self.element);
        self.element += 1;

        Some(result)
    }
}
