//! Canonical WebeID value and representation conversions.

use std::fmt;
use std::str::FromStr;

use crate::components::WebeIdComponents;
use crate::error::ParseWebeIdError;

/// Number of bits used by the millisecond time component.
pub const TIME_BITS: u8 = 40;
/// Number of bits used by the node identifier component.
pub const NODE_BITS: u8 = 8;
/// Number of bits used by the sequence component.
pub const SEQUENCE_BITS: u8 = 16;

/// Bit shift for the millisecond time component.
pub const TIME_SHIFT: u8 = NODE_BITS + SEQUENCE_BITS;
/// Bit shift for the node component.
pub const NODE_SHIFT: u8 = SEQUENCE_BITS;

/// Maximum millisecond duration encodable in the 40-bit time component.
pub const MAX_TIME_MILLISECONDS: u64 = (1_u64 << TIME_BITS) - 1;
/// Maximum node value encodable in the 8-bit node component.
pub const MAX_NODE_ID: u8 = u8::MAX;
/// Maximum sequence value encodable in the 16-bit sequence component.
pub const MAX_SEQUENCE: u16 = u16::MAX;

const NODE_MASK: u64 = 0xff;
const SEQUENCE_MASK: u64 = 0xffff;
const BYTE_LENGTH: usize = 8;

/// Canonical compact 64-bit WebeID value.
///
/// The value sorts numerically by its high-order 40-bit millisecond component.
///
/// # Examples
///
/// ```
/// use webe_id::{NodeId, WebeIdComponents};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let id = WebeIdComponents::new(10, NodeId::from_u8(2), 5)?.to_id();
/// assert_eq!(id.to_hex_string(), "000000000a020005");
/// assert_eq!(webe_id::WebeId::parse_hex("000000000a020005")?, id);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WebeId(u64);

impl WebeId {
    /// Creates a WebeID from its raw canonical `u64` representation.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw canonical `u64` representation.
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns the decomposed 40-bit time, 8-bit node, and 16-bit sequence fields.
    pub const fn components(self) -> WebeIdComponents {
        let time_millis = self.0 >> TIME_SHIFT;
        let node_id = ((self.0 >> NODE_SHIFT) & NODE_MASK) as u8;
        let sequence = (self.0 & SEQUENCE_MASK) as u16;
        WebeIdComponents::from_encoded_parts(time_millis, node_id, sequence)
    }

    /// Creates a WebeID from exactly eight big-endian bytes.
    pub const fn from_be_bytes(bytes: [u8; BYTE_LENGTH]) -> Self {
        Self(u64::from_be_bytes(bytes))
    }

    /// Converts this WebeID to exactly eight big-endian bytes.
    pub const fn to_be_bytes(self) -> [u8; BYTE_LENGTH] {
        self.0.to_be_bytes()
    }

    /// Parses a WebeID from a big-endian byte slice.
    pub fn parse_be_bytes(bytes: &[u8]) -> Result<Self, ParseWebeIdError> {
        if bytes.len() != BYTE_LENGTH {
            return Err(ParseWebeIdError::InvalidByteLength {
                expected: BYTE_LENGTH,
                actual: bytes.len(),
            });
        }

        let mut raw_bytes = [0_u8; BYTE_LENGTH];
        raw_bytes.copy_from_slice(bytes);
        Ok(Self::from_be_bytes(raw_bytes))
    }

    /// Formats this WebeID as unsigned decimal text.
    pub fn to_decimal_string(self) -> String {
        self.0.to_string()
    }

    /// Parses a WebeID from unsigned decimal text.
    pub fn parse_decimal(input: &str) -> Result<Self, ParseWebeIdError> {
        input
            .parse::<u64>()
            .map(Self)
            .map_err(|_| ParseWebeIdError::InvalidDecimal {
                input: input.to_owned(),
            })
    }

    /// Formats this WebeID as fixed-width lowercase hexadecimal text.
    pub fn to_hex_string(self) -> String {
        format!("{:016x}", self.0)
    }

    /// Parses a WebeID from hexadecimal text with an optional `0x` prefix.
    pub fn parse_hex(input: &str) -> Result<Self, ParseWebeIdError> {
        let trimmed = input
            .strip_prefix("0x")
            .or_else(|| input.strip_prefix("0X"))
            .unwrap_or(input);

        if trimmed.is_empty() {
            return Err(ParseWebeIdError::InvalidHex {
                input: input.to_owned(),
            });
        }

        u64::from_str_radix(trimmed, 16)
            .map(Self)
            .map_err(|_| ParseWebeIdError::InvalidHex {
                input: input.to_owned(),
            })
    }
}

impl From<u64> for WebeId {
    fn from(raw: u64) -> Self {
        Self::from_raw(raw)
    }
}

impl From<WebeId> for u64 {
    fn from(id: WebeId) -> Self {
        id.as_u64()
    }
}

impl fmt::Display for WebeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl FromStr for WebeId {
    type Err = ParseWebeIdError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse_decimal(input)
    }
}
