#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Wired M-Bus data-link fields and frame codecs.
//!
//! This crate represents and encodes the wired M-Bus frames exchanged by
//! masters and slaves. Its wire formats and message types follow the
//! link-layer portion of EN 13757-2:2018+A1:2023, which uses the FT1.2 format
//! class defined by the EN 60870-5 series.
//!
//! The API is suitable for M-Bus masters, meters, bridges, and diagnostic
//! tools. It is [`no_std`](https://doc.rust-lang.org/reference/names/preludes.html#the-no_std-attribute)
//! and supports encoding without dynamic allocation.
//!
//! # Scope
//!
//! The crate provides:
//!
//! - typed control and address fields;
//! - ACK and NACK acknowledgement frames;
//! - fixed-length short frames;
//! - variable-format control frames without user data; and
//! - variable-format long frames containing user data.
//!
//! Constructors and decoders validate each frame's structure, checksum, and
//! supported control-field use. Address values remain representable even when
//! they are reserved or inappropriate for a particular exchange; callers can
//! inspect [`Address::kind`] and [`Address::expects_response`] when applying
//! protocol rules.
//!
//! This crate does **not** implement:
//!
//! - the wired electrical interface or UART configuration;
//! - byte timing, collision handling, retries, or response timeouts;
//! - master or slave state machines;
//! - application-layer interpretation of the control-information byte or user
//!   data.
//!
//! Those concerns belong to the surrounding physical, transport, and
//! application layers. In particular, application data carried by a
//! [`LongFrame`] is specified separately by EN 13757-3.
//!
//! # Choosing a frame
//!
//! | Type | Wire form | Typical role |
//! | --- | --- | --- |
//! | [`AckFrame`] | One byte, `0xe5` | Positively acknowledge an accepted request |
//! | [`NackFrame`] | One byte, `0xa2` | Reject an unsupported SND-UD2 request |
//! | [`ShortFrame`] | Five-byte fixed format | Send SND-NKE, REQ-UD1, or REQ-UD2 |
//! | [`ControlFrame`] | Nine-byte variable format with no user data | Carry control and control-information fields without an application payload |
//! | [`LongFrame`] | Variable format with 1 to 252 user-data bytes | Send or respond with application data |
//!
//! A frame constructor rejects a [`Control`] value that is incompatible with
//! that frame format. [`CommunicationType`] identifies the supported
//! communication type represented by a raw control field.
//!
//! # Encoding and decoding
//!
//! Every `decode` function consumes a slice containing exactly one complete
//! frame. It rejects truncated input, trailing bytes, invalid delimiters,
//! invalid lengths, checksum mismatches, and incompatible control fields as
//! applicable to that frame type. [`decoder::stream`] identifies boundaries
//! across arbitrary chunks and can recover from noise without allocation.
//!
//! Every frame provides `encode_into`, which writes into a caller-owned buffer
//! and returns only the initialized portion. When the `alloc` feature is
//! enabled, frames also provide `encode`, which returns an allocated
//! `Vec`.
//!
//! # Example
//!
//! The following constructs the standard five-byte shape of a REQ-UD2 request
//! for slave address 1, encodes it without allocation, and decodes it again:
//!
//! ```
//! use meterbus_wired_datalink::{Address, Control, ShortFrame};
//!
//! # fn main() -> Result<(), meterbus_wired_datalink::ShortFrameError> {
//! let request = ShortFrame::new(Control::req_ud2(false), Address::new(1))?;
//! let mut storage = [0_u8; ShortFrame::LEN];
//! let encoded = request.encode_into(&mut storage)?;
//!
//! assert_eq!(encoded, [0x10, 0x5b, 0x01, 0x5c, 0x16]);
//! assert_eq!(ShortFrame::decode(encoded)?, request);
//! # Ok(())
//! # }
//! ```
//!
//! # Allocation and crate features
//!
//! The crate itself always uses `#![no_std]`.
//!
//! - the default feature set is allocation-free;
//! - `alloc` adds allocating `encode` methods and the collecting
//!   `decoder::stream::StreamDecoder::push` convenience method.
//!
//! Without `alloc`, encode with `encode_into` and stream with `push_into`.
//!
//! A valid frame is not necessarily the right frame for the current exchange.
//! Callers must still manage message order, timing, retries, and application
//! data according to EN 13757-2 and EN 13757-3.

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod decoder;
mod frame;

pub use frame::{AckFrame, AckFrameError};
pub use frame::{Address, AddressKind, CommunicationType, Control, ControlError, Direction};
pub use frame::{ControlFrame, ControlFrameError};
pub use frame::{Frame, FrameEncodeError, FrameKind};
pub use frame::{LongFrame, LongFrameError};
pub use frame::{NackFrame, NackFrameError};
pub use frame::{ShortFrame, ShortFrameError};
