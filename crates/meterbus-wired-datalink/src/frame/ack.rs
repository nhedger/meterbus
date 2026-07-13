//! Positive acknowledgement (ACK) frames.
//!
//! An ACK is the single byte `0xe5`. It confirms that a slave accepted a
//! request when the M-Bus exchange calls for an acknowledgement. It contains
//! no address, control field, or checksum.
//!
//! ```text
//! +------+
//! | 0xe5 |
//! +------+
//! ```
//!
//! [`AckFrame::decode`] requires exactly one byte and rejects any other value.
//! [`AckFrame::encode_into`] requires at least one output byte and returns a
//! one-byte slice. With the `alloc` feature, [`AckFrame::encode`] returns an
//! allocated vector.
//!
//! # Example
//!
//! ```
//! use meterbus_wired_datalink::AckFrame;
//!
//! # fn main() -> Result<(), meterbus_wired_datalink::AckFrameError> {
//! let frame = AckFrame::decode(&[0xe5])?;
//! let mut output = [0_u8; AckFrame::LEN];
//! assert_eq!(frame.encode_into(&mut output)?, [0xe5]);
//! # Ok(())
//! # }
//! ```

use core::fmt;

#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

/// A positive acknowledgement frame.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct AckFrame;

impl AckFrame {
    /// Wire byte identifying an acknowledgement.
    pub const BYTE: u8 = 0xe5;
    /// Encoded frame length.
    pub const LEN: usize = 1;

    /// Creates an acknowledgement frame.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Decodes exactly one acknowledgement frame.
    pub fn decode(bytes: &[u8]) -> Result<Self, AckFrameError> {
        if bytes.len() != Self::LEN {
            return Err(AckFrameError::InvalidLength {
                actual: bytes.len(),
            });
        }
        if bytes[0] != Self::BYTE {
            return Err(AckFrameError::InvalidByte { actual: bytes[0] });
        }
        Ok(Self)
    }

    /// Encodes the frame into `output` and returns the encoded portion.
    pub fn encode_into<'a>(&self, output: &'a mut [u8]) -> Result<&'a [u8], AckFrameError> {
        if output.len() < Self::LEN {
            return Err(AckFrameError::OutputTooSmall {
                actual: output.len(),
            });
        }
        output[0] = Self::BYTE;
        Ok(&output[..Self::LEN])
    }

    /// Encodes the frame into a newly allocated vector.
    #[cfg(feature = "alloc")]
    pub fn encode(&self) -> Vec<u8> {
        vec![Self::BYTE]
    }
}

/// Error produced while encoding or decoding an acknowledgement frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum AckFrameError {
    /// The input was not exactly one byte.
    InvalidLength { actual: usize },
    /// The single input byte was not `0xe5`.
    InvalidByte { actual: u8 },
    /// The output buffer had no room for the frame.
    OutputTooSmall { actual: usize },
}

impl fmt::Display for AckFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { actual } => write!(
                formatter,
                "invalid ACK frame length: expected 1, got {actual}"
            ),
            Self::InvalidByte { actual } => write!(
                formatter,
                "invalid ACK frame byte: expected 0xe5, got 0x{actual:02x}"
            ),
            Self::OutputTooSmall { actual } => write!(
                formatter,
                "ACK output buffer too small: expected 1, got {actual}"
            ),
        }
    }
}

impl core::error::Error for AckFrameError {}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::string::ToString;

    #[test]
    fn encodes_and_decodes() {
        let mut output = [0; 2];
        let encoded = AckFrame::new().encode_into(&mut output).unwrap();
        assert_eq!(encoded, [0xe5]);
        assert_eq!(AckFrame::decode(encoded), Ok(AckFrame));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn allocates_encoded_frame() {
        assert_eq!(AckFrame.encode(), [0xe5]);
    }

    #[test]
    fn rejects_invalid_inputs_and_output() {
        assert_eq!(
            AckFrame::decode(&[]),
            Err(AckFrameError::InvalidLength { actual: 0 })
        );
        assert_eq!(
            AckFrame::decode(&[0xe5, 0]),
            Err(AckFrameError::InvalidLength { actual: 2 })
        );
        assert_eq!(
            AckFrame::decode(&[0]),
            Err(AckFrameError::InvalidByte { actual: 0 })
        );
        assert_eq!(
            AckFrame.encode_into(&mut []),
            Err(AckFrameError::OutputTooSmall { actual: 0 })
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn formats_errors() {
        assert_eq!(
            AckFrameError::InvalidLength { actual: 0 }.to_string(),
            "invalid ACK frame length: expected 1, got 0"
        );
        assert_eq!(
            AckFrameError::InvalidByte { actual: 0 }.to_string(),
            "invalid ACK frame byte: expected 0xe5, got 0x00"
        );
        assert_eq!(
            AckFrameError::OutputTooSmall { actual: 0 }.to_string(),
            "ACK output buffer too small: expected 1, got 0"
        );
    }
}
