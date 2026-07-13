//! Variable-format long frames with user data.
//!
//! Long frames carry [`Control`], [`Address`], a raw control-information byte,
//! and 1 to 252 bytes of user data.
//!
//! ```text
//! +------+---+---+------+---+---+----+-----------+----------+------+
//! | 0x68 | L | L | 0x68 | C | A | CI | user data | checksum | 0x16 |
//! +------+---+---+------+---+---+----+-----------+----------+------+
//! ```
//!
//! `L` counts `C`, `A`, `CI`, and the user-data bytes. It therefore ranges
//! from 4 to 255. The complete encoded frame ranges from
//! [`LongFrame::MIN_LEN`] to [`LongFrame::MAX_LEN`] bytes. The checksum is the
//! wrapping eight-bit sum from `C` through the final user-data byte.
//!
//! [`LongFrame::new`] checks the control value and payload length, then copies
//! the payload into fixed-capacity storage owned by the frame. This does not
//! require the `alloc` feature. [`LongFrame::decode`] also checks the repeated
//! length, start and stop bytes, exact total size, and checksum.
//!
//! [`LongFrame::encode_into`] is allocation-free. With the `alloc` feature,
//! [`LongFrame::encode`] returns an allocated vector. The control-information
//! byte and user data are not interpreted by this crate.
//!
//! # Example
//!
//! ```
//! use meterbus_wired_datalink::{Address, Control, LongFrame};
//!
//! # fn main() -> Result<(), meterbus_wired_datalink::LongFrameError> {
//! let frame = LongFrame::new(
//!     Control::snd_ud(false),
//!     Address::new(254),
//!     0x50,
//!     &[0x10],
//! )?;
//! let mut output = [0_u8; LongFrame::MIN_LEN];
//! assert_eq!(
//!     frame.encode_into(&mut output)?,
//!     [0x68, 0x04, 0x04, 0x68, 0x53, 0xfe, 0x50, 0x10, 0xb1, 0x16],
//! );
//! # Ok(())
//! # }
//! ```

use core::fmt;

#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

use super::field::{Address, Control, ControlError};

/// A variable-format frame containing user data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LongFrame {
    control: Control,
    address: Address,
    control_information: u8,
    user_data: [u8; Self::MAX_USER_DATA_LEN],
    user_data_len: u8,
}

impl LongFrame {
    /// Initial and repeated start byte.
    pub const START: u8 = 0x68;
    /// Number of checksum-covered bytes before user data.
    pub const FIXED_DATA_LEN: usize = 3;
    /// Minimum user-data length.
    pub const MIN_USER_DATA_LEN: usize = 1;
    /// Maximum user-data length permitted by the one-byte length field.
    pub const MAX_USER_DATA_LEN: usize = 252;
    /// Final frame byte.
    pub const STOP: u8 = 0x16;
    /// Minimum encoded frame length.
    pub const MIN_LEN: usize = 10;
    /// Maximum encoded frame length.
    pub const MAX_LEN: usize = 261;

    /// Creates a validated long frame and copies `user_data` into inline storage.
    pub fn new(
        control: Control,
        address: Address,
        control_information: u8,
        user_data: &[u8],
    ) -> Result<Self, LongFrameError> {
        control
            .validate_variable_frame()
            .map_err(LongFrameError::Control)?;
        if !(Self::MIN_USER_DATA_LEN..=Self::MAX_USER_DATA_LEN).contains(&user_data.len()) {
            return Err(LongFrameError::InvalidUserDataLength {
                actual: user_data.len(),
            });
        }
        let mut stored = [0; Self::MAX_USER_DATA_LEN];
        stored[..user_data.len()].copy_from_slice(user_data);
        Ok(Self {
            control,
            address,
            control_information,
            user_data: stored,
            user_data_len: user_data.len() as u8,
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

    /// Returns the user-data bytes.
    #[must_use]
    pub fn user_data(&self) -> &[u8] {
        &self.user_data[..usize::from(self.user_data_len)]
    }

    /// Decodes exactly one long frame.
    pub fn decode(bytes: &[u8]) -> Result<Self, LongFrameError> {
        if bytes.len() < 4 {
            return Err(LongFrameError::IncompleteHeader {
                actual: bytes.len(),
            });
        }
        if bytes[0] != Self::START {
            return Err(LongFrameError::InvalidStart {
                index: 0,
                actual: bytes[0],
            });
        }
        if bytes[1] != bytes[2] {
            return Err(LongFrameError::MismatchedDataLengths {
                first: bytes[1],
                second: bytes[2],
            });
        }
        if usize::from(bytes[1]) < Self::FIXED_DATA_LEN + Self::MIN_USER_DATA_LEN {
            return Err(LongFrameError::InvalidDataLength { actual: bytes[1] });
        }
        let expected_len = usize::from(bytes[1]) + 6;
        if bytes.len() != expected_len {
            return Err(LongFrameError::InvalidLength {
                expected: expected_len,
                actual: bytes.len(),
            });
        }
        if bytes[3] != Self::START {
            return Err(LongFrameError::InvalidStart {
                index: 3,
                actual: bytes[3],
            });
        }
        if bytes[expected_len - 1] != Self::STOP {
            return Err(LongFrameError::InvalidStop {
                actual: bytes[expected_len - 1],
            });
        }
        let expected_checksum = Self::checksum(&bytes[4..expected_len - 2]);
        let actual_checksum = bytes[expected_len - 2];
        if actual_checksum != expected_checksum {
            return Err(LongFrameError::InvalidChecksum {
                expected: expected_checksum,
                actual: actual_checksum,
            });
        }
        Self::new(
            Control::new(bytes[4]),
            Address::new(bytes[5]),
            bytes[6],
            &bytes[7..expected_len - 2],
        )
    }

    /// Encodes the frame into `output` and returns the encoded portion.
    pub fn encode_into<'a>(&self, output: &'a mut [u8]) -> Result<&'a [u8], LongFrameError> {
        let required = self.required_len();
        if output.len() < required {
            return Err(LongFrameError::OutputTooSmall {
                required,
                actual: output.len(),
            });
        }
        let data_len = (Self::FIXED_DATA_LEN + self.user_data().len()) as u8;
        output[0] = Self::START;
        output[1] = data_len;
        output[2] = data_len;
        output[3] = Self::START;
        output[4] = self.control.value();
        output[5] = self.address.value();
        output[6] = self.control_information;
        output[7..required - 2].copy_from_slice(self.user_data());
        output[required - 2] = Self::checksum(&output[4..required - 2]);
        output[required - 1] = Self::STOP;
        Ok(&output[..required])
    }

    /// Encodes the frame into a newly allocated vector.
    #[cfg(feature = "alloc")]
    pub fn encode(&self) -> Vec<u8> {
        let mut output = vec![0; self.required_len()];
        let required = output.len();
        let data_len = (Self::FIXED_DATA_LEN + self.user_data().len()) as u8;
        output[..7].copy_from_slice(&[
            Self::START,
            data_len,
            data_len,
            Self::START,
            self.control.value(),
            self.address.value(),
            self.control_information,
        ]);
        output[7..required - 2].copy_from_slice(self.user_data());
        output[required - 2] = Self::checksum(&output[4..required - 2]);
        output[required - 1] = Self::STOP;
        output
    }

    fn required_len(&self) -> usize {
        self.user_data().len() + 9
    }

    fn checksum(bytes: &[u8]) -> u8 {
        bytes
            .iter()
            .fold(0, |checksum, byte| checksum.wrapping_add(*byte))
    }
}

/// Error produced while constructing, encoding, or decoding a long frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum LongFrameError {
    /// Fewer than four header bytes were supplied.
    IncompleteHeader { actual: usize },
    /// The encoded length differed from that declared by the frame.
    InvalidLength { expected: usize, actual: usize },
    /// A start byte was invalid.
    InvalidStart { index: usize, actual: u8 },
    /// The repeated length fields differed.
    MismatchedDataLengths { first: u8, second: u8 },
    /// The length field did not leave room for user data.
    InvalidDataLength { actual: u8 },
    /// The stop byte was invalid.
    InvalidStop { actual: u8 },
    /// The checksum did not match the frame contents.
    InvalidChecksum { expected: u8, actual: u8 },
    /// The control field is invalid for a variable-format frame.
    Control(ControlError),
    /// User data was empty or exceeded 252 bytes.
    InvalidUserDataLength { actual: usize },
    /// The output buffer was too small.
    OutputTooSmall { required: usize, actual: usize },
}

impl fmt::Display for LongFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompleteHeader { actual } => write!(
                formatter,
                "incomplete long frame header: expected at least 4 bytes, got {actual}"
            ),
            Self::InvalidLength { expected, actual } => write!(
                formatter,
                "invalid long frame length: expected {expected}, got {actual}"
            ),
            Self::InvalidStart { index, actual } => write!(
                formatter,
                "invalid long frame start at {index}: expected 0x68, got 0x{actual:02x}"
            ),
            Self::MismatchedDataLengths { first, second } => write!(
                formatter,
                "mismatched long frame data lengths: {first} and {second}"
            ),
            Self::InvalidDataLength { actual } => write!(
                formatter,
                "invalid long frame data length: expected at least 4, got {actual}"
            ),
            Self::InvalidStop { actual } => write!(
                formatter,
                "invalid long frame stop: expected 0x16, got 0x{actual:02x}"
            ),
            Self::InvalidChecksum { expected, actual } => write!(
                formatter,
                "invalid long frame checksum: expected 0x{expected:02x}, got 0x{actual:02x}"
            ),
            Self::Control(error) => error.fmt(formatter),
            Self::InvalidUserDataLength { actual } => write!(
                formatter,
                "invalid long frame user-data length: expected 1 through 252, got {actual}"
            ),
            Self::OutputTooSmall { required, actual } => write!(
                formatter,
                "long frame output buffer too small: expected {required}, got {actual}"
            ),
        }
    }
}

impl core::error::Error for LongFrameError {
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

    const FRAME: [u8; 10] = [0x68, 0x04, 0x04, 0x68, 0x53, 0xfe, 0x50, 0x10, 0xb1, 0x16];

    #[test]
    fn encodes_and_decodes_known_frame() {
        let frame = LongFrame::new(Control::new(0x53), Address::new(254), 0x50, &[0x10]).unwrap();
        let mut output = [0; LongFrame::MAX_LEN];
        assert_eq!(frame.encode_into(&mut output).unwrap(), FRAME);
        assert_eq!(LongFrame::decode(&FRAME), Ok(frame.clone()));
        assert_eq!(frame.control(), Control::new(0x53));
        assert_eq!(frame.address(), Address::new(254));
        assert_eq!(frame.control_information(), 0x50);
        assert_eq!(frame.user_data(), [0x10]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn allocates_encoded_frame() {
        assert_eq!(LongFrame::decode(&FRAME).unwrap().encode(), FRAME);
    }

    #[test]
    fn owns_payload_and_validates_boundaries() {
        let mut source = [1, 2];
        let frame = LongFrame::new(Control::new(0x53), Address::new(252), 0, &source).unwrap();
        source[0] = 9;
        assert_eq!(source[0], 9);
        assert_eq!(frame.user_data(), [1, 2]);
        assert_eq!(
            LongFrame::new(Control::new(0x53), Address::new(1), 0, &[]),
            Err(LongFrameError::InvalidUserDataLength { actual: 0 })
        );
        let error = LongFrameError::Control(ControlError::InvalidForVariableFrame { value: 0x40 });
        assert!(core::error::Error::source(&error).is_some());
        assert!(
            core::error::Error::source(&LongFrameError::IncompleteHeader { actual: 0 }).is_none()
        );
        assert_eq!(
            LongFrame::new(Control::new(0x53), Address::new(1), 0, &[0; 253]),
            Err(LongFrameError::InvalidUserDataLength { actual: 253 })
        );
        assert_eq!(
            LongFrame::new(Control::new(0x40), Address::new(1), 0, &[1]),
            Err(LongFrameError::Control(
                ControlError::InvalidForVariableFrame { value: 0x40 }
            ))
        );
    }

    #[test]
    fn supports_maximum_frame() {
        let frame =
            LongFrame::new(Control::new(0x53), Address::new(255), 255, &[0xff; 252]).unwrap();
        let mut output = [0; LongFrame::MAX_LEN];
        let encoded = frame.encode_into(&mut output).unwrap();
        assert_eq!(encoded.len(), LongFrame::MAX_LEN);
        assert_eq!(encoded[1], 0xff);
        assert_eq!(LongFrame::decode(encoded), Ok(frame));
    }

    #[test]
    fn rejects_header_and_length_errors() {
        assert_eq!(
            LongFrame::decode(&[0; 3]),
            Err(LongFrameError::IncompleteHeader { actual: 3 })
        );
        let mut bytes = FRAME;
        bytes[0] = 0;
        assert_eq!(
            LongFrame::decode(&bytes),
            Err(LongFrameError::InvalidStart {
                index: 0,
                actual: 0
            })
        );
        bytes = FRAME;
        bytes[2] = 5;
        assert_eq!(
            LongFrame::decode(&bytes),
            Err(LongFrameError::MismatchedDataLengths {
                first: 4,
                second: 5
            })
        );
        bytes = FRAME;
        bytes[1] = 3;
        bytes[2] = 3;
        assert_eq!(
            LongFrame::decode(&bytes),
            Err(LongFrameError::InvalidDataLength { actual: 3 })
        );
        assert_eq!(
            LongFrame::decode(&FRAME[..9]),
            Err(LongFrameError::InvalidLength {
                expected: 10,
                actual: 9
            })
        );
        let mut trailing = [0; 11];
        trailing[..10].copy_from_slice(&FRAME);
        assert_eq!(
            LongFrame::decode(&trailing),
            Err(LongFrameError::InvalidLength {
                expected: 10,
                actual: 11
            })
        );
    }

    #[test]
    fn rejects_body_errors() {
        let mut bytes = FRAME;
        bytes[3] = 0;
        assert_eq!(
            LongFrame::decode(&bytes),
            Err(LongFrameError::InvalidStart {
                index: 3,
                actual: 0
            })
        );
        bytes = FRAME;
        bytes[9] = 0;
        assert_eq!(
            LongFrame::decode(&bytes),
            Err(LongFrameError::InvalidStop { actual: 0 })
        );
        bytes = FRAME;
        bytes[8] = 0;
        assert_eq!(
            LongFrame::decode(&bytes),
            Err(LongFrameError::InvalidChecksum {
                expected: 0xb1,
                actual: 0
            })
        );
        bytes = [0x68, 4, 4, 0x68, 0x40, 1, 0, 1, 0x42, 0x16];
        assert_eq!(
            LongFrame::decode(&bytes),
            Err(LongFrameError::Control(
                ControlError::InvalidForVariableFrame { value: 0x40 }
            ))
        );
    }

    #[test]
    fn rejects_small_output() {
        let frame = LongFrame::decode(&FRAME).unwrap();
        assert_eq!(
            frame.encode_into(&mut [0; 9]),
            Err(LongFrameError::OutputTooSmall {
                required: 10,
                actual: 9
            })
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn formats_errors() {
        let errors = [
            (
                LongFrameError::IncompleteHeader { actual: 3 }.to_string(),
                "incomplete long frame header: expected at least 4 bytes, got 3",
            ),
            (
                LongFrameError::InvalidLength {
                    expected: 10,
                    actual: 9,
                }
                .to_string(),
                "invalid long frame length: expected 10, got 9",
            ),
            (
                LongFrameError::InvalidStart {
                    index: 3,
                    actual: 0,
                }
                .to_string(),
                "invalid long frame start at 3: expected 0x68, got 0x00",
            ),
            (
                LongFrameError::MismatchedDataLengths {
                    first: 4,
                    second: 5,
                }
                .to_string(),
                "mismatched long frame data lengths: 4 and 5",
            ),
            (
                LongFrameError::InvalidDataLength { actual: 3 }.to_string(),
                "invalid long frame data length: expected at least 4, got 3",
            ),
            (
                LongFrameError::InvalidStop { actual: 0 }.to_string(),
                "invalid long frame stop: expected 0x16, got 0x00",
            ),
            (
                LongFrameError::InvalidChecksum {
                    expected: 1,
                    actual: 2,
                }
                .to_string(),
                "invalid long frame checksum: expected 0x01, got 0x02",
            ),
            (
                LongFrameError::Control(ControlError::InvalidForVariableFrame { value: 0x40 })
                    .to_string(),
                "control value 0x40 is invalid for a variable-format frame",
            ),
            (
                LongFrameError::InvalidUserDataLength { actual: 0 }.to_string(),
                "invalid long frame user-data length: expected 1 through 252, got 0",
            ),
            (
                LongFrameError::OutputTooSmall {
                    required: 10,
                    actual: 9,
                }
                .to_string(),
                "long frame output buffer too small: expected 10, got 9",
            ),
        ];
        for (actual, expected) in errors {
            assert_eq!(actual, expected);
        }
    }
}
