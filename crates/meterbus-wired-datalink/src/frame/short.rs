//! Fixed-length short frames.
//!
//! Masters use short frames for SND-NKE, REQ-UD1, and REQ-UD2 messages. A
//! short frame carries a [`Control`] field and an [`Address`], but no
//! control-information byte or user data.
//!
//! ```text
//! +------+---+---+----------+------+
//! | 0x10 | C | A | checksum | 0x16 |
//! +------+---+---+----------+------+
//! ```
//!
//! The checksum is the wrapping eight-bit sum of `C` and `A`. The encoded frame
//! is always [`ShortFrame::LEN`] bytes long.
//!
//! [`ShortFrame::new`] rejects control values that do not belong in a short
//! frame. [`ShortFrame::decode`] additionally checks the exact length, start
//! and stop bytes, and checksum. Reserved and special addresses remain valid
//! values; the caller decides whether they are suitable for the request.
//!
//! # Example
//!
//! ```
//! use meterbus_wired_datalink::{Address, Control, ShortFrame};
//!
//! # fn main() -> Result<(), meterbus_wired_datalink::ShortFrameError> {
//! let frame = ShortFrame::new(Control::req_ud2(false), Address::new(1))?;
//! let mut output = [0_u8; ShortFrame::LEN];
//! assert_eq!(
//!     frame.encode_into(&mut output)?,
//!     [0x10, 0x5b, 0x01, 0x5c, 0x16],
//! );
//! # Ok(())
//! # }
//! ```

use core::fmt;

#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

use super::field::{Address, Control, ControlError};

/// A fixed-length short frame.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ShortFrame {
    control: Control,
    address: Address,
}

impl ShortFrame {
    /// Initial frame byte.
    pub const START: u8 = 0x10;
    /// Final frame byte.
    pub const STOP: u8 = 0x16;
    /// Encoded frame length.
    pub const LEN: usize = 5;

    /// Creates a validated short frame.
    pub const fn new(control: Control, address: Address) -> Result<Self, ShortFrameError> {
        if let Err(error) = control.validate_short_frame() {
            return Err(ShortFrameError::Control(error));
        }
        Ok(Self { control, address })
    }

    /// Returns the control field.
    #[must_use]
    pub const fn control(&self) -> Control {
        self.control
    }

    /// Returns the address field.
    #[must_use]
    pub const fn address(&self) -> Address {
        self.address
    }

    /// Decodes exactly one short frame.
    pub fn decode(bytes: &[u8]) -> Result<Self, ShortFrameError> {
        if bytes.len() != Self::LEN {
            return Err(ShortFrameError::InvalidLength {
                actual: bytes.len(),
            });
        }
        if bytes[0] != Self::START {
            return Err(ShortFrameError::InvalidStart { actual: bytes[0] });
        }
        if bytes[4] != Self::STOP {
            return Err(ShortFrameError::InvalidStop { actual: bytes[4] });
        }
        let expected = Self::checksum(bytes[1], bytes[2]);
        if bytes[3] != expected {
            return Err(ShortFrameError::InvalidChecksum {
                expected,
                actual: bytes[3],
            });
        }
        Self::new(Control::new(bytes[1]), Address::new(bytes[2]))
    }

    /// Encodes the frame into `output` and returns the encoded portion.
    pub fn encode_into<'a>(&self, output: &'a mut [u8]) -> Result<&'a [u8], ShortFrameError> {
        if output.len() < Self::LEN {
            return Err(ShortFrameError::OutputTooSmall {
                actual: output.len(),
            });
        }
        let control = self.control.value();
        let address = self.address.value();
        output[..Self::LEN].copy_from_slice(&[
            Self::START,
            control,
            address,
            Self::checksum(control, address),
            Self::STOP,
        ]);
        Ok(&output[..Self::LEN])
    }

    /// Encodes the frame into a newly allocated vector.
    #[cfg(feature = "alloc")]
    pub fn encode(&self) -> Vec<u8> {
        let control = self.control.value();
        let address = self.address.value();
        vec![
            Self::START,
            control,
            address,
            Self::checksum(control, address),
            Self::STOP,
        ]
    }

    const fn checksum(control: u8, address: u8) -> u8 {
        control.wrapping_add(address)
    }
}

/// Error produced while constructing, encoding, or decoding a short frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum ShortFrameError {
    /// The input length was not five bytes.
    InvalidLength { actual: usize },
    /// The initial start byte was invalid.
    InvalidStart { actual: u8 },
    /// The final stop byte was invalid.
    InvalidStop { actual: u8 },
    /// The checksum did not match the frame contents.
    InvalidChecksum { expected: u8, actual: u8 },
    /// The control field is invalid for a short frame.
    Control(ControlError),
    /// The output buffer is shorter than five bytes.
    OutputTooSmall { actual: usize },
}

impl fmt::Display for ShortFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { actual } => write!(
                formatter,
                "invalid short frame length: expected 5, got {actual}"
            ),
            Self::InvalidStart { actual } => write!(
                formatter,
                "invalid short frame start: expected 0x10, got 0x{actual:02x}"
            ),
            Self::InvalidStop { actual } => write!(
                formatter,
                "invalid short frame stop: expected 0x16, got 0x{actual:02x}"
            ),
            Self::InvalidChecksum { expected, actual } => write!(
                formatter,
                "invalid short frame checksum: expected 0x{expected:02x}, got 0x{actual:02x}"
            ),
            Self::Control(error) => error.fmt(formatter),
            Self::OutputTooSmall { actual } => write!(
                formatter,
                "short frame output buffer too small: expected 5, got {actual}"
            ),
        }
    }
}

impl core::error::Error for ShortFrameError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Control(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::string::ToString;

    const FRAME: [u8; 5] = [0x10, 0x40, 0x01, 0x41, 0x16];

    #[test]
    fn encodes_and_decodes_specification_frame() {
        let frame = ShortFrame::new(Control::new(0x40), Address::new(1)).unwrap();
        let mut output = [0; 6];
        assert_eq!(frame.encode_into(&mut output).unwrap(), FRAME);
        assert_eq!(ShortFrame::decode(&FRAME), Ok(frame));
        assert_eq!(frame.control(), Control::new(0x40));
        assert_eq!(frame.address(), Address::new(1));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn allocates_encoded_frame() {
        let frame = ShortFrame::decode(&FRAME).unwrap();
        assert_eq!(frame.encode(), FRAME);
    }

    #[test]
    fn validates_constructor_fields() {
        assert_eq!(
            ShortFrame::new(Control::new(0x53), Address::new(1)),
            Err(ShortFrameError::Control(
                ControlError::InvalidForShortFrame { value: 0x53 }
            ))
        );
        assert!(ShortFrame::new(Control::new(0x40), Address::new(252)).is_ok());
        let error = ShortFrameError::Control(ControlError::InvalidForShortFrame { value: 0x53 });
        assert!(core::error::Error::source(&error).is_some());
        assert!(
            core::error::Error::source(&ShortFrameError::InvalidLength { actual: 0 }).is_none()
        );
    }

    #[test]
    fn rejects_each_structural_error() {
        assert_eq!(
            ShortFrame::decode(&FRAME[..4]),
            Err(ShortFrameError::InvalidLength { actual: 4 })
        );
        assert_eq!(
            ShortFrame::decode(&[FRAME.as_slice(), &[0]].concat()),
            Err(ShortFrameError::InvalidLength { actual: 6 })
        );
        let mut bytes = FRAME;
        bytes[0] = 0;
        assert_eq!(
            ShortFrame::decode(&bytes),
            Err(ShortFrameError::InvalidStart { actual: 0 })
        );
        bytes = FRAME;
        bytes[4] = 0;
        assert_eq!(
            ShortFrame::decode(&bytes),
            Err(ShortFrameError::InvalidStop { actual: 0 })
        );
        bytes = FRAME;
        bytes[3] = 0;
        assert_eq!(
            ShortFrame::decode(&bytes),
            Err(ShortFrameError::InvalidChecksum {
                expected: 0x41,
                actual: 0
            })
        );
    }

    #[test]
    fn rejects_semantically_invalid_decoded_fields() {
        assert_eq!(
            ShortFrame::decode(&[0x10, 0x53, 1, 0x54, 0x16]),
            Err(ShortFrameError::Control(
                ControlError::InvalidForShortFrame { value: 0x53 }
            ))
        );
        assert_eq!(
            ShortFrame::decode(&[0x10, 0x40, 252, 0x3c, 0x16])
                .unwrap()
                .address(),
            Address::new(252)
        );
    }

    #[test]
    fn wraps_checksum_and_rejects_small_output() {
        let frame = ShortFrame::new(Control::new(0x7b), Address::new(255)).unwrap();
        let mut output = [0; 5];
        assert_eq!(
            frame.encode_into(&mut output).unwrap(),
            [0x10, 0x7b, 0xff, 0x7a, 0x16]
        );
        assert_eq!(
            frame.encode_into(&mut [0; 4]),
            Err(ShortFrameError::OutputTooSmall { actual: 4 })
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn formats_errors() {
        assert_eq!(
            ShortFrameError::InvalidLength { actual: 0 }.to_string(),
            "invalid short frame length: expected 5, got 0"
        );
        assert_eq!(
            ShortFrameError::InvalidStart { actual: 0 }.to_string(),
            "invalid short frame start: expected 0x10, got 0x00"
        );
        assert_eq!(
            ShortFrameError::InvalidStop { actual: 0 }.to_string(),
            "invalid short frame stop: expected 0x16, got 0x00"
        );
        assert_eq!(
            ShortFrameError::InvalidChecksum {
                expected: 1,
                actual: 2
            }
            .to_string(),
            "invalid short frame checksum: expected 0x01, got 0x02"
        );
        assert_eq!(
            ShortFrameError::Control(ControlError::InvalidForShortFrame { value: 0x53 })
                .to_string(),
            "control value 0x53 is invalid for a short frame"
        );
        assert_eq!(
            ShortFrameError::OutputTooSmall { actual: 4 }.to_string(),
            "short frame output buffer too small: expected 5, got 4"
        );
    }
}
