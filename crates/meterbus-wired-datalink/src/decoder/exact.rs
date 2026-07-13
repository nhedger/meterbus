//! Decode a slice containing exactly one wired M-Bus frame.
//!
//! [`decode`] identifies ACK, NACK, short, control, and long frames from their
//! leading bytes. The selected frame decoder then validates the complete slice,
//! including its length and checksum.
//!
//! ```
//! use meterbus_wired_datalink::{
//!     CommunicationType,
//!     Frame,
//!     decoder::exact::decode,
//! };
//!
//! # fn main() -> Result<(), meterbus_wired_datalink::decoder::exact::DecodeError> {
//! let frame = decode(&[0x10, 0x5b, 0x01, 0x5c, 0x16])?;
//! let Frame::Short(frame) = frame else {
//!     unreachable!();
//! };
//! assert_eq!(frame.control().communication_type(), CommunicationType::ReqUd2);
//! # Ok(())
//! # }
//! ```

use core::fmt;

use crate::{
    AckFrame, AckFrameError, ControlFrame, ControlFrameError, Frame, LongFrame, LongFrameError,
    NackFrame, NackFrameError, ShortFrame, ShortFrameError,
};

/// Decodes a slice containing exactly one frame.
///
/// Variable frames with a data length of three decode as [`Frame::Control`].
/// Larger data lengths decode as [`Frame::Long`]. The chosen decoder
/// rejects incomplete input and trailing bytes.
///
/// # Errors
///
/// Returns [`DecodeError::Empty`] for an empty slice,
/// [`DecodeError::IncompleteVariableHeader`] when a variable-frame start byte
/// is not followed by a length byte, or [`DecodeError::UnknownStart`] when the
/// first byte does not identify a supported frame. Other errors come from the
/// selected frame decoder.
pub fn decode(bytes: &[u8]) -> Result<Frame, DecodeError> {
    let Some(start) = bytes.first().copied() else {
        return Err(DecodeError::Empty);
    };

    match start {
        AckFrame::BYTE => AckFrame::decode(bytes)
            .map(Frame::Ack)
            .map_err(DecodeError::Ack),
        NackFrame::BYTE => NackFrame::decode(bytes)
            .map(Frame::Nack)
            .map_err(DecodeError::Nack),
        ShortFrame::START => ShortFrame::decode(bytes)
            .map(Frame::Short)
            .map_err(DecodeError::Short),
        ControlFrame::START => {
            let Some(data_len) = bytes.get(1).copied() else {
                return Err(DecodeError::IncompleteVariableHeader);
            };
            if data_len == ControlFrame::DATA_LEN {
                ControlFrame::decode(bytes)
                    .map(Frame::Control)
                    .map_err(DecodeError::Control)
            } else {
                LongFrame::decode(bytes)
                    .map(Frame::Long)
                    .map_err(DecodeError::Long)
            }
        }
        actual => Err(DecodeError::UnknownStart { actual }),
    }
}

/// Error returned by [`decode`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// No bytes were supplied.
    Empty,
    /// A variable-frame start byte was not followed by a length byte.
    IncompleteVariableHeader,
    /// The first byte does not identify a supported frame.
    UnknownStart {
        /// The unsupported first byte.
        actual: u8,
    },
    /// An ACK frame was invalid.
    Ack(AckFrameError),
    /// A NACK frame was invalid.
    Nack(NackFrameError),
    /// A short frame was invalid.
    Short(ShortFrameError),
    /// A control frame was invalid.
    Control(ControlFrameError),
    /// A long frame was invalid.
    Long(LongFrameError),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("cannot decode a frame from empty input"),
            Self::IncompleteVariableHeader => {
                formatter.write_str("incomplete variable frame header: expected a length byte")
            }
            Self::UnknownStart { actual } => {
                write!(formatter, "unknown frame start byte 0x{actual:02x}")
            }
            Self::Ack(error) => error.fmt(formatter),
            Self::Nack(error) => error.fmt(formatter),
            Self::Short(error) => error.fmt(formatter),
            Self::Control(error) => error.fmt(formatter),
            Self::Long(error) => error.fmt(formatter),
        }
    }
}

impl core::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Ack(error) => Some(error),
            Self::Nack(error) => Some(error),
            Self::Short(error) => Some(error),
            Self::Control(error) => Some(error),
            Self::Long(error) => Some(error),
            Self::Empty | Self::IncompleteVariableHeader | Self::UnknownStart { .. } => None,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::string::ToString;

    #[test]
    fn decodes_every_frame_type() {
        let cases = [
            (&[0xe5][..], Frame::Ack(AckFrame::new())),
            (&[0xa2][..], Frame::Nack(NackFrame::new())),
            (
                &[0x10, 0x5b, 0x01, 0x5c, 0x16][..],
                Frame::Short(
                    ShortFrame::new(crate::Control::new(0x5b), crate::Address::new(1)).unwrap(),
                ),
            ),
        ];

        for (bytes, expected) in cases {
            assert_eq!(decode(bytes), Ok(expected));
        }

        assert_eq!(
            decode(&[0x68, 3, 3, 0x68, 0x53, 0xfe, 0xbd, 0x0e, 0x16]),
            Ok(Frame::Control(
                ControlFrame::new(crate::Control::new(0x53), crate::Address::new(254), 0xbd)
                    .unwrap()
            ))
        );
        assert_eq!(
            decode(&[0x68, 4, 4, 0x68, 0x53, 0xfe, 0x50, 0x10, 0xb1, 0x16]),
            Ok(Frame::Long(
                LongFrame::new(
                    crate::Control::new(0x53),
                    crate::Address::new(254),
                    0x50,
                    &[0x10],
                )
                .unwrap()
            ))
        );
    }

    #[test]
    fn rejects_unknown_or_incomplete_input() {
        assert_eq!(decode(&[]), Err(DecodeError::Empty));
        assert_eq!(decode(&[0x68]), Err(DecodeError::IncompleteVariableHeader));
        assert_eq!(
            decode(&[0xff]),
            Err(DecodeError::UnknownStart { actual: 0xff })
        );
    }

    #[test]
    fn preserves_selected_decoder_errors() {
        assert_eq!(
            decode(&[0xe5, 0]),
            Err(DecodeError::Ack(AckFrameError::InvalidLength { actual: 2 }))
        );
        assert_eq!(
            decode(&[0x10, 0x5b, 0x01, 0x00, 0x16]),
            Err(DecodeError::Short(ShortFrameError::InvalidChecksum {
                expected: 0x5c,
                actual: 0,
            }))
        );
        assert_eq!(
            decode(&[0x68, 3, 4, 0x68, 0x53, 0xfe, 0xbd, 0x0e, 0x16]),
            Err(DecodeError::Control(ControlFrameError::InvalidDataLength {
                index: 2,
                actual: 4,
            }))
        );
        assert_eq!(
            decode(&[0x68, 2, 2, 0x68, 0, 0, 0, 0]),
            Err(DecodeError::Long(LongFrameError::InvalidDataLength {
                actual: 2
            }))
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn formats_and_exposes_errors() {
        let errors = [
            (DecodeError::Empty, "cannot decode a frame from empty input"),
            (
                DecodeError::IncompleteVariableHeader,
                "incomplete variable frame header: expected a length byte",
            ),
            (
                DecodeError::UnknownStart { actual: 0xff },
                "unknown frame start byte 0xff",
            ),
            (
                DecodeError::Ack(AckFrameError::InvalidByte { actual: 0 }),
                "invalid ACK frame byte: expected 0xe5, got 0x00",
            ),
            (
                DecodeError::Nack(NackFrameError::InvalidByte { actual: 0 }),
                "invalid NACK frame byte: expected 0xa2, got 0x00",
            ),
            (
                DecodeError::Short(ShortFrameError::InvalidLength { actual: 0 }),
                "invalid short frame length: expected 5, got 0",
            ),
            (
                DecodeError::Control(ControlFrameError::InvalidLength { actual: 0 }),
                "invalid control frame length: expected 9, got 0",
            ),
            (
                DecodeError::Long(LongFrameError::IncompleteHeader { actual: 0 }),
                "incomplete long frame header: expected at least 4 bytes, got 0",
            ),
        ];

        for (error, expected) in &errors {
            assert_eq!(error.to_string(), *expected);
        }

        assert!(core::error::Error::source(&DecodeError::Empty).is_none());
        assert!(core::error::Error::source(&DecodeError::IncompleteVariableHeader).is_none());
        assert!(core::error::Error::source(&DecodeError::UnknownStart { actual: 0 }).is_none());
        for (error, _) in &errors[3..] {
            assert!(core::error::Error::source(error).is_some());
        }
    }
}
