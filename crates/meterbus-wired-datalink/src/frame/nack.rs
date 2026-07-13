//! Negative acknowledgement (NACK) frames.
//!
//! A NACK is the single byte `0xa2`. In wired M-Bus it is used when a slave
//! does not support an SND-UD2 request. It contains no address, control field,
//! or checksum.
//!
//! ```text
//! +------+
//! | 0xa2 |
//! +------+
//! ```
//!
//! [`NackFrame::decode`] requires exactly one byte and rejects any other value.
//! [`NackFrame::encode_into`] requires at least one output byte and returns a
//! one-byte slice. With the `alloc` feature, [`NackFrame::encode`] returns an
//! allocated vector.
//!
//! # Example
//!
//! ```
//! use meterbus_wired_datalink::NackFrame;
//!
//! # fn main() -> Result<(), meterbus_wired_datalink::NackFrameError> {
//! let frame = NackFrame::new();
//! let mut output = [0_u8; NackFrame::LEN];
//! assert_eq!(frame.encode_into(&mut output)?, [0xa2]);
//! assert_eq!(NackFrame::decode(&output)?, frame);
//! # Ok(())
//! # }
//! ```

use core::fmt;

#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

/// A negative acknowledgement frame.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct NackFrame;

impl NackFrame {
    /// Wire byte identifying a negative acknowledgement.
    pub const BYTE: u8 = 0xa2;
    /// Encoded frame length.
    pub const LEN: usize = 1;

    /// Creates a negative acknowledgement frame.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Decodes exactly one negative acknowledgement frame.
    pub fn decode(bytes: &[u8]) -> Result<Self, NackFrameError> {
        if bytes.len() != Self::LEN {
            return Err(NackFrameError::InvalidLength {
                actual: bytes.len(),
            });
        }
        if bytes[0] != Self::BYTE {
            return Err(NackFrameError::InvalidByte { actual: bytes[0] });
        }
        Ok(Self)
    }

    /// Encodes the frame into `output` and returns the encoded portion.
    pub fn encode_into<'a>(&self, output: &'a mut [u8]) -> Result<&'a [u8], NackFrameError> {
        if output.len() < Self::LEN {
            return Err(NackFrameError::OutputTooSmall {
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

/// Error produced while encoding or decoding a negative acknowledgement frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum NackFrameError {
    /// The input was not exactly one byte.
    InvalidLength { actual: usize },
    /// The single input byte was not `0xa2`.
    InvalidByte { actual: u8 },
    /// The output buffer had no room for the frame.
    OutputTooSmall { actual: usize },
}

impl fmt::Display for NackFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { actual } => write!(
                formatter,
                "invalid NACK frame length: expected 1, got {actual}"
            ),
            Self::InvalidByte { actual } => write!(
                formatter,
                "invalid NACK frame byte: expected 0xa2, got 0x{actual:02x}"
            ),
            Self::OutputTooSmall { actual } => write!(
                formatter,
                "NACK output buffer too small: expected 1, got {actual}"
            ),
        }
    }
}

impl core::error::Error for NackFrameError {}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::string::ToString;

    #[test]
    fn encodes_and_decodes() {
        let mut output = [0; 1];
        assert_eq!(NackFrame::new().encode_into(&mut output).unwrap(), [0xa2]);
        assert_eq!(NackFrame::decode(&output), Ok(NackFrame));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn allocates_encoded_frame() {
        assert_eq!(NackFrame.encode(), [0xa2]);
    }

    #[test]
    fn rejects_invalid_inputs_and_output() {
        assert_eq!(
            NackFrame::decode(&[]),
            Err(NackFrameError::InvalidLength { actual: 0 })
        );
        assert_eq!(
            NackFrame::decode(&[0xa2, 0]),
            Err(NackFrameError::InvalidLength { actual: 2 })
        );
        assert_eq!(
            NackFrame::decode(&[0xe5]),
            Err(NackFrameError::InvalidByte { actual: 0xe5 })
        );
        assert_eq!(
            NackFrame.encode_into(&mut []),
            Err(NackFrameError::OutputTooSmall { actual: 0 })
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn formats_errors() {
        assert_eq!(
            NackFrameError::InvalidLength { actual: 0 }.to_string(),
            "invalid NACK frame length: expected 1, got 0"
        );
        assert_eq!(
            NackFrameError::InvalidByte { actual: 0 }.to_string(),
            "invalid NACK frame byte: expected 0xa2, got 0x00"
        );
        assert_eq!(
            NackFrameError::OutputTooSmall { actual: 0 }.to_string(),
            "NACK output buffer too small: expected 1, got 0"
        );
    }
}
