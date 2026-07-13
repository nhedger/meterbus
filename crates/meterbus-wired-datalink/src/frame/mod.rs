//! Wired M-Bus data-link frame formats.
//!
//! This module groups the frame representations used by the crate. The types
//! are re-exported from the crate root, so users normally import
//! [`AckFrame`], [`NackFrame`], [`ShortFrame`], [`ControlFrame`], and
//! [`LongFrame`] directly from `meterbus_wired_datalink`.
//!
//! EN 13757-2 uses the FT1.2 frame format for wired M-Bus. This crate
//! represents five forms: two single-byte acknowledgements, one fixed-length
//! short format, and two variants of the variable format.
//!
//! # Supported formats
//!
//! | Frame | First byte | Encoded length | Contents |
//! | --- | --- | --- | --- |
//! | [`AckFrame`] | `0xe5` | 1 byte | Positive acknowledgement |
//! | [`NackFrame`] | `0xa2` | 1 byte | Negative acknowledgement |
//! | [`ShortFrame`] | `0x10` | 5 bytes | Control and address fields |
//! | [`ControlFrame`] | `0x68` | 9 bytes | Control, address, and control-information fields |
//! | [`LongFrame`] | `0x68` | 10 to 261 bytes | Control, address, control information, and 1 to 252 user-data bytes |
//!
//! [`ControlFrame`] is the zero-user-data case of the variable wire format.
//! It has a distinct Rust type because [`LongFrame`] deliberately requires at
//! least one user-data byte.
//!
//! # Wire layouts
//!
//! The diagrams below use the field names customary for M-Bus:
//!
//! - `L` is the data-length field;
//! - `C` is the data-link [`Control`] field;
//! - `A` is the data-link [`Address`] field;
//! - `CI` is the control-information byte used by the application protocol;
//! - `UD` is user data; and
//! - `CS` is the wrapping eight-bit checksum.
//!
//! ## Acknowledgements
//!
//! ACK and NACK each consist of one fixed byte and contain no
//! address, control field, or checksum:
//!
//! ```text
//! ACK:  E5
//! NACK: A2
//! ```
//!
//! ## Short frame
//!
//! ```text
//! +-------+---+---+----+------+
//! | 0x10  | C | A | CS | 0x16 |
//! +-------+---+---+----+------+
//! ```
//!
//! `CS` is the wrapping sum of `C` and `A`. Short frames carry the SND-NKE,
//! REQ-UD1, and REQ-UD2 communication types supported by [`Control`].
//!
//! ## Variable-format control frame
//!
//! ```text
//! +------+------+------+------+---+---+----+----+------+
//! | 0x68 | 0x03 | 0x03 | 0x68 | C | A | CI | CS | 0x16 |
//! +------+------+------+------+---+---+----+----+------+
//! ```
//!
//! Both length bytes are `3` because the checksum-covered data region contains
//! `C`, `A`, and `CI`. `CS` is the wrapping sum of those three bytes.
//!
//! ## Variable-format long frame
//!
//! ```text
//! +------+------+------+-------+---+---+----+--------+----+------+
//! | 0x68 |  L   |  L   | 0x68  | C | A | CI | UD ... | CS | 0x16 |
//! +------+------+------+-------+---+---+----+--------+----+------+
//! ```
//!
//! `L` counts `C`, `A`, `CI`, and every user-data byte. Consequently, this
//! crate accepts `L` from 4 through 255 and complete long frames from 10
//! through 261 bytes. `CS` is the wrapping sum of every byte from `C` through
//! the final user-data byte.
//!
//! # Decoding and frame boundaries
//!
//! Each frame decoder expects exactly one complete frame. Fixed-size decoders
//! reject both short and trailing input. [`LongFrame::decode`] derives the
//! expected total size from the repeated `L` field and likewise rejects any
//! extra bytes. None of the decoders scans for a start byte or returns an
//! unconsumed remainder.
//!
//! A stream decoder can select an initial candidate from the first byte:
//!
//! | First byte | Candidate |
//! | --- | --- |
//! | `0xe5` | [`AckFrame`] |
//! | `0xa2` | [`NackFrame`] |
//! | `0x10` | [`ShortFrame`] |
//! | `0x68` | Variable format; inspect `L` to distinguish a control frame from a long frame |
//!
//! For a `0x68` prefix, repeated length bytes of `3` describe a
//! [`ControlFrame`]. Values from `4` through `255` describe a [`LongFrame`].
//! The surrounding transport remains responsible for buffering the declared
//! number of bytes before invoking the selected decoder.
//!
//! Decoders check lengths, separators, checksums, and control-field
//! compatibility. All frame types can encode into a caller-provided buffer.
//! With the `alloc` feature, they can also return an allocated vector.
//!
//! # Example
//!
//! Decode based on an already-established frame boundary and known format:
//!
//! ```
//! use meterbus_wired_datalink::{
//!     AckFrame, Address, CommunicationType, Control, ControlFrame, Frame, LongFrame,
//!     NackFrame, ShortFrame,
//! };
//!
//! # fn main() -> Result<(), meterbus_wired_datalink::ShortFrameError> {
//! let bytes = [0x10, 0x5b, 0x01, 0x5c, 0x16];
//! let frame = ShortFrame::decode(&bytes)?;
//!
//! assert_eq!(frame.control().communication_type(), CommunicationType::ReqUd2);
//! assert_eq!(frame.address().value(), 1);
//! # let _: Frame = AckFrame::new().into();
//! # let _: Frame = NackFrame::new().into();
//! # let _: Frame = frame.into();
//! # let _: Frame = ControlFrame::new(Control::snd_ud2(), Address::new(1), 0).expect("valid control frame").into();
//! # let _: Frame = LongFrame::new(Control::snd_ud(false), Address::new(1), 0, &[1]).expect("valid long frame").into();
//! # Ok(())
//! # }
//! ```
//!
//! The codecs validate individual frames. Message order, timing, retries, and
//! application data remain the caller's responsibility.

mod ack;
mod control;
pub mod field;
mod long;
mod nack;
mod short;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::fmt;

pub use ack::{AckFrame, AckFrameError};
pub use control::{ControlFrame, ControlFrameError};
pub use field::{Address, AddressKind, CommunicationType, Control, ControlError, Direction};
pub use long::{LongFrame, LongFrameError};
pub use nack::{NackFrame, NackFrameError};
pub use short::{ShortFrame, ShortFrameError};

/// Any supported wired M-Bus frame.
///
/// The long variant remains inline so this type works without an allocator.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Frame {
    /// A positive acknowledgement.
    Ack(AckFrame),
    /// A negative acknowledgement.
    Nack(NackFrame),
    /// A fixed-length short frame.
    Short(ShortFrame),
    /// A variable-format frame without user data.
    Control(ControlFrame),
    /// A variable-format frame with user data.
    Long(LongFrame),
}

/// The wire format represented by a [`Frame`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum FrameKind {
    /// Positive acknowledgement.
    Ack,
    /// Negative acknowledgement.
    Nack,
    /// Fixed-length short frame.
    Short,
    /// Variable-format frame without user data.
    Control,
    /// Variable-format frame with user data.
    Long,
}

impl Frame {
    /// Returns the frame's wire format.
    #[must_use]
    pub const fn kind(&self) -> FrameKind {
        match self {
            Self::Ack(_) => FrameKind::Ack,
            Self::Nack(_) => FrameKind::Nack,
            Self::Short(_) => FrameKind::Short,
            Self::Control(_) => FrameKind::Control,
            Self::Long(_) => FrameKind::Long,
        }
    }

    /// Returns the encoded frame length.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Ack(_) => AckFrame::LEN,
            Self::Nack(_) => NackFrame::LEN,
            Self::Short(_) => ShortFrame::LEN,
            Self::Control(_) => ControlFrame::LEN,
            Self::Long(frame) => frame.user_data().len() + 9,
        }
    }

    /// Returns whether the encoded frame is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }

    /// Encodes the frame into `output` and returns the encoded portion.
    ///
    /// # Errors
    ///
    /// Returns [`FrameEncodeError`] when `output` is shorter than [`Self::len`].
    pub fn encode_into<'a>(&self, output: &'a mut [u8]) -> Result<&'a [u8], FrameEncodeError> {
        let required = self.len();
        if output.len() < required {
            return Err(FrameEncodeError {
                required,
                actual: output.len(),
            });
        }
        let encoded = match self {
            Self::Ack(frame) => frame
                .encode_into(output)
                .expect("prechecked ACK output length"),
            Self::Nack(frame) => frame
                .encode_into(output)
                .expect("prechecked NACK output length"),
            Self::Short(frame) => frame
                .encode_into(output)
                .expect("prechecked short-frame output length"),
            Self::Control(frame) => frame
                .encode_into(output)
                .expect("prechecked control-frame output length"),
            Self::Long(frame) => frame
                .encode_into(output)
                .expect("prechecked long-frame output length"),
        };
        Ok(encoded)
    }

    /// Encodes the frame into a newly allocated vector.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::Ack(frame) => frame.encode(),
            Self::Nack(frame) => frame.encode(),
            Self::Short(frame) => frame.encode(),
            Self::Control(frame) => frame.encode(),
            Self::Long(frame) => frame.encode(),
        }
    }
}

/// Error returned when a buffer cannot hold an encoded [`Frame`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct FrameEncodeError {
    /// Number of bytes required by the frame.
    pub required: usize,
    /// Number of bytes available in the output buffer.
    pub actual: usize,
}

impl fmt::Display for FrameEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "frame output buffer too small: expected {}, got {}",
            self.required, self.actual
        )
    }
}

impl core::error::Error for FrameEncodeError {}

impl From<AckFrame> for Frame {
    fn from(frame: AckFrame) -> Self {
        Self::Ack(frame)
    }
}

impl From<NackFrame> for Frame {
    fn from(frame: NackFrame) -> Self {
        Self::Nack(frame)
    }
}

impl From<ShortFrame> for Frame {
    fn from(frame: ShortFrame) -> Self {
        Self::Short(frame)
    }
}

impl From<ControlFrame> for Frame {
    fn from(frame: ControlFrame) -> Self {
        Self::Control(frame)
    }
}

impl From<LongFrame> for Frame {
    fn from(frame: LongFrame) -> Self {
        Self::Long(frame)
    }
}

#[cfg(test)]
mod conversion_coverage {
    use super::*;

    #[test]
    fn converts_concrete_frames() {
        let _: Frame = AckFrame::new().into();
        let _: Frame = NackFrame::new().into();
        let _: Frame = ShortFrame::new(Control::snd_nke(), Address::new(1))
            .expect("valid short frame")
            .into();
        let _: Frame = ControlFrame::new(Control::snd_ud2(), Address::new(1), 0)
            .expect("valid control frame")
            .into();
        let _: Frame = LongFrame::new(Control::snd_ud(false), Address::new(1), 0, &[1])
            .expect("valid long frame")
            .into();
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn generic_frame_api_covers_every_kind() {
        let frames = [
            Frame::from(AckFrame::new()),
            Frame::from(NackFrame::new()),
            Frame::from(ShortFrame::new(Control::snd_nke(), Address::new(1)).unwrap()),
            Frame::from(ControlFrame::new(Control::snd_ud2(), Address::new(1), 0).unwrap()),
            Frame::from(LongFrame::new(Control::snd_ud(false), Address::new(1), 0, &[1]).unwrap()),
        ];
        let kinds = [
            FrameKind::Ack,
            FrameKind::Nack,
            FrameKind::Short,
            FrameKind::Control,
            FrameKind::Long,
        ];
        for (frame, kind) in frames.iter().zip(kinds) {
            assert_eq!(frame.kind(), kind);
            assert!(!frame.is_empty());
            let mut output = [0; LongFrame::MAX_LEN];
            assert_eq!(frame.encode_into(&mut output).unwrap().len(), frame.len());
            assert_eq!(
                frame.encode_into(&mut output[..frame.len() - 1]),
                Err(FrameEncodeError {
                    required: frame.len(),
                    actual: frame.len() - 1
                })
            );
            #[cfg(feature = "alloc")]
            assert_eq!(frame.encode().len(), frame.len());
        }
    }

    #[test]
    fn large_frame_types_remain_inline_and_bounded() {
        assert!(core::mem::size_of::<LongFrame>() <= LongFrame::MAX_LEN);
        assert!(
            core::mem::size_of::<Frame>() <= LongFrame::MAX_LEN + core::mem::size_of::<usize>()
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn formats_frame_encode_errors() {
        use alloc::string::ToString;

        let error = FrameEncodeError {
            required: 5,
            actual: 4,
        };
        assert_eq!(
            error.to_string(),
            "frame output buffer too small: expected 5, got 4"
        );
        assert!(core::error::Error::source(&error).is_none());
    }
}
