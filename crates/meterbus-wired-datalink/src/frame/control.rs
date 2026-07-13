//! Variable-format control frames without user data.
//!
//! A control frame carries [`Control`], [`Address`], and a raw
//! control-information byte. It uses the variable M-Bus frame layout but fixes
//! both length bytes to `3` because no user data follows.
//!
//! ```text
//! +------+------+------+------+---+---+----+----------+------+
//! | 0x68 | 0x03 | 0x03 | 0x68 | C | A | CI | checksum | 0x16 |
//! +------+------+------+------+---+---+----+----------+------+
//! ```
//!
//! The checksum is the wrapping eight-bit sum of `C`, `A`, and `CI`. The
//! encoded frame is always [`ControlFrame::LEN`] bytes long.
//!
//! [`ControlFrame::new`] accepts control values used by variable frames.
//! [`ControlFrame::decode`] also checks both length bytes, both start bytes,
//! the stop byte, and the checksum. The crate stores `CI` as a `u8`; its
//! meaning belongs to the application protocol.
//!
//! # Example
//!
//! ```
//! use meterbus_wired_datalink::{Address, Control, ControlFrame};
//!
//! # fn main() -> Result<(), meterbus_wired_datalink::ControlFrameError> {
//! let frame = ControlFrame::new(Control::snd_ud(false), Address::new(254), 0xbd)?;
//! let mut output = [0_u8; ControlFrame::LEN];
//! assert_eq!(
//!     frame.encode_into(&mut output)?,
//!     [0x68, 0x03, 0x03, 0x68, 0x53, 0xfe, 0xbd, 0x0e, 0x16],
//! );
//! # Ok(())
//! # }
//! ```

use core::fmt;

#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

use super::field::{Address, Control, ControlError};

/// A variable-format frame without user data.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ControlFrame {
    control: Control,
    address: Address,
    control_information: u8,
}

impl ControlFrame {
    /// Initial and repeated start byte.
    pub const START: u8 = 0x68;
    /// Number of checksum-covered bytes.
    pub const DATA_LEN: u8 = 3;
    /// Final frame byte.
    pub const STOP: u8 = 0x16;
    /// Encoded frame length.
    pub const LEN: usize = 9;

    /// Creates a validated control frame.
    pub const fn new(
        control: Control,
        address: Address,
        control_information: u8,
    ) -> Result<Self, ControlFrameError> {
        if let Err(error) = control.validate_variable_frame() {
            return Err(ControlFrameError::Control(error));
        }
        Ok(Self {
            control,
            address,
            control_information,
        })
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

    /// Returns the control-information byte.
    #[must_use]
    pub const fn control_information(&self) -> u8 {
        self.control_information
    }

    /// Decodes exactly one control frame.
    pub fn decode(bytes: &[u8]) -> Result<Self, ControlFrameError> {
        if bytes.len() != Self::LEN {
            return Err(ControlFrameError::InvalidLength {
                actual: bytes.len(),
            });
        }
        if bytes[0] != Self::START {
            return Err(ControlFrameError::InvalidStart {
                index: 0,
                actual: bytes[0],
            });
        }
        if bytes[1] != Self::DATA_LEN {
            return Err(ControlFrameError::InvalidDataLength {
                index: 1,
                actual: bytes[1],
            });
        }
        if bytes[2] != Self::DATA_LEN {
            return Err(ControlFrameError::InvalidDataLength {
                index: 2,
                actual: bytes[2],
            });
        }
        if bytes[3] != Self::START {
            return Err(ControlFrameError::InvalidStart {
                index: 3,
                actual: bytes[3],
            });
        }
        if bytes[8] != Self::STOP {
            return Err(ControlFrameError::InvalidStop { actual: bytes[8] });
        }
        let expected = Self::checksum(bytes[4], bytes[5], bytes[6]);
        if bytes[7] != expected {
            return Err(ControlFrameError::InvalidChecksum {
                expected,
                actual: bytes[7],
            });
        }
        Self::new(Control::new(bytes[4]), Address::new(bytes[5]), bytes[6])
    }

    /// Encodes the frame into `output` and returns the encoded portion.
    pub fn encode_into<'a>(&self, output: &'a mut [u8]) -> Result<&'a [u8], ControlFrameError> {
        if output.len() < Self::LEN {
            return Err(ControlFrameError::OutputTooSmall {
                actual: output.len(),
            });
        }
        let control = self.control.value();
        let address = self.address.value();
        output[..Self::LEN].copy_from_slice(&[
            Self::START,
            Self::DATA_LEN,
            Self::DATA_LEN,
            Self::START,
            control,
            address,
            self.control_information,
            Self::checksum(control, address, self.control_information),
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
            Self::DATA_LEN,
            Self::DATA_LEN,
            Self::START,
            control,
            address,
            self.control_information,
            Self::checksum(control, address, self.control_information),
            Self::STOP,
        ]
    }

    const fn checksum(control: u8, address: u8, control_information: u8) -> u8 {
        control
            .wrapping_add(address)
            .wrapping_add(control_information)
    }
}

/// Error produced while constructing, encoding, or decoding a control frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum ControlFrameError {
    /// The input length was not nine bytes.
    InvalidLength { actual: usize },
    /// A start byte was invalid.
    InvalidStart { index: usize, actual: u8 },
    /// A length byte did not contain three.
    InvalidDataLength { index: usize, actual: u8 },
    /// The stop byte was invalid.
    InvalidStop { actual: u8 },
    /// The checksum did not match the frame contents.
    InvalidChecksum { expected: u8, actual: u8 },
    /// The control field is invalid for a variable-format frame.
    Control(ControlError),
    /// The output buffer is shorter than nine bytes.
    OutputTooSmall { actual: usize },
}

impl fmt::Display for ControlFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { actual } => write!(
                formatter,
                "invalid control frame length: expected 9, got {actual}"
            ),
            Self::InvalidStart { index, actual } => write!(
                formatter,
                "invalid control frame start at {index}: expected 0x68, got 0x{actual:02x}"
            ),
            Self::InvalidDataLength { index, actual } => write!(
                formatter,
                "invalid control frame data length at {index}: expected 3, got {actual}"
            ),
            Self::InvalidStop { actual } => write!(
                formatter,
                "invalid control frame stop: expected 0x16, got 0x{actual:02x}"
            ),
            Self::InvalidChecksum { expected, actual } => write!(
                formatter,
                "invalid control frame checksum: expected 0x{expected:02x}, got 0x{actual:02x}"
            ),
            Self::Control(error) => error.fmt(formatter),
            Self::OutputTooSmall { actual } => write!(
                formatter,
                "control frame output buffer too small: expected 9, got {actual}"
            ),
        }
    }
}

impl core::error::Error for ControlFrameError {
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

    const FRAME: [u8; 9] = [0x68, 0x03, 0x03, 0x68, 0x53, 0xfe, 0xbd, 0x0e, 0x16];

    #[test]
    fn encodes_and_decodes_known_frame() {
        let frame = ControlFrame::new(Control::new(0x53), Address::new(254), 0xbd).unwrap();
        let mut output = [0; 10];
        assert_eq!(frame.encode_into(&mut output).unwrap(), FRAME);
        assert_eq!(ControlFrame::decode(&FRAME), Ok(frame));
        assert_eq!(frame.control(), Control::new(0x53));
        assert_eq!(frame.address(), Address::new(254));
        assert_eq!(frame.control_information(), 0xbd);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn allocates_encoded_frame() {
        assert_eq!(ControlFrame::decode(&FRAME).unwrap().encode(), FRAME);
    }

    #[test]
    fn validates_control_but_tolerates_reserved_address() {
        assert_eq!(
            ControlFrame::new(Control::new(0x40), Address::new(1), 0),
            Err(ControlFrameError::Control(
                ControlError::InvalidForVariableFrame { value: 0x40 }
            ))
        );
        assert!(ControlFrame::new(Control::new(0x53), Address::new(252), 0).is_ok());
        let error =
            ControlFrameError::Control(ControlError::InvalidForVariableFrame { value: 0x40 });
        assert!(core::error::Error::source(&error).is_some());
        assert!(
            core::error::Error::source(&ControlFrameError::InvalidLength { actual: 0 }).is_none()
        );
    }

    #[test]
    fn rejects_structural_errors() {
        assert_eq!(
            ControlFrame::decode(&FRAME[..8]),
            Err(ControlFrameError::InvalidLength { actual: 8 })
        );
        let mut bytes = FRAME;
        bytes[0] = 0;
        assert_eq!(
            ControlFrame::decode(&bytes),
            Err(ControlFrameError::InvalidStart {
                index: 0,
                actual: 0
            })
        );
        bytes = FRAME;
        bytes[1] = 4;
        assert_eq!(
            ControlFrame::decode(&bytes),
            Err(ControlFrameError::InvalidDataLength {
                index: 1,
                actual: 4
            })
        );
        bytes = FRAME;
        bytes[2] = 4;
        assert_eq!(
            ControlFrame::decode(&bytes),
            Err(ControlFrameError::InvalidDataLength {
                index: 2,
                actual: 4
            })
        );
        bytes = FRAME;
        bytes[3] = 0;
        assert_eq!(
            ControlFrame::decode(&bytes),
            Err(ControlFrameError::InvalidStart {
                index: 3,
                actual: 0
            })
        );
        bytes = FRAME;
        bytes[8] = 0;
        assert_eq!(
            ControlFrame::decode(&bytes),
            Err(ControlFrameError::InvalidStop { actual: 0 })
        );
        bytes = FRAME;
        bytes[7] = 0;
        assert_eq!(
            ControlFrame::decode(&bytes),
            Err(ControlFrameError::InvalidChecksum {
                expected: 0x0e,
                actual: 0
            })
        );
    }

    #[test]
    fn rejects_invalid_decoded_control_and_small_output() {
        assert_eq!(
            ControlFrame::decode(&[0x68, 3, 3, 0x68, 0x40, 1, 0, 0x41, 0x16]),
            Err(ControlFrameError::Control(
                ControlError::InvalidForVariableFrame { value: 0x40 }
            ))
        );
        let frame = ControlFrame::decode(&FRAME).unwrap();
        assert_eq!(
            frame.encode_into(&mut [0; 8]),
            Err(ControlFrameError::OutputTooSmall { actual: 8 })
        );
    }

    #[test]
    fn wraps_checksum() {
        let frame = ControlFrame::new(Control::new(0x53), Address::new(255), 255).unwrap();
        let mut output = [0; 9];
        assert_eq!(frame.encode_into(&mut output).unwrap()[7], 0x51);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn formats_errors() {
        let cases = [
            (
                ControlFrameError::InvalidLength { actual: 0 }.to_string(),
                "invalid control frame length: expected 9, got 0",
            ),
            (
                ControlFrameError::InvalidStart {
                    index: 3,
                    actual: 0,
                }
                .to_string(),
                "invalid control frame start at 3: expected 0x68, got 0x00",
            ),
            (
                ControlFrameError::InvalidDataLength {
                    index: 1,
                    actual: 4,
                }
                .to_string(),
                "invalid control frame data length at 1: expected 3, got 4",
            ),
            (
                ControlFrameError::InvalidStop { actual: 0 }.to_string(),
                "invalid control frame stop: expected 0x16, got 0x00",
            ),
            (
                ControlFrameError::InvalidChecksum {
                    expected: 1,
                    actual: 2,
                }
                .to_string(),
                "invalid control frame checksum: expected 0x01, got 0x02",
            ),
            (
                ControlFrameError::Control(ControlError::InvalidForVariableFrame { value: 0x40 })
                    .to_string(),
                "control value 0x40 is invalid for a variable-format frame",
            ),
            (
                ControlFrameError::OutputTooSmall { actual: 8 }.to_string(),
                "control frame output buffer too small: expected 9, got 8",
            ),
        ];
        for (actual, expected) in cases {
            assert_eq!(actual, expected);
        }
    }
}
