//! Data-link fields shared by structured wired M-Bus frames.
//!
//! Short and variable-format frames contain two common one-byte fields:
//! [`Control`] identifies the communication and its data-link flags, while
//! [`Address`] identifies the intended slave or a special addressing mode.
//! Variable-format frames additionally contain a control-information byte,
//! but that byte belongs to the boundary with the application protocol and is
//! therefore retained as a raw `u8` by [`ControlFrame`](super::ControlFrame)
//! and [`LongFrame`](super::LongFrame).
//!
//! The field types in this module are re-exported from the crate root. Users
//! normally import them directly from `meterbus_wired_datalink` rather than
//! through this module's internal path.
//!
//! # Wire position
//!
//! Both fields occupy one byte and appear at the beginning of the
//! checksum-covered region:
//!
//! ```text
//! Short frame:    10 C A CS 16
//! Variable frame: 68 L L 68 C A CI [user data ...] CS 16
//!                           ^ ^
//!                           | +-- Address
//!                           +---- Control
//! ```
//!
//! The frame codecs calculate and verify the checksum around these values.
//! Field values do not calculate checksums themselves and do not retain their
//! position within an encoded frame.
//!
//! # Keeping the original byte
//!
//! [`Address`] and [`Control`] each wrap a `u8`. Creating either type cannot
//! fail, and the original byte is always preserved. This lets a decoder report
//! a reserved or unsupported value instead of losing it.
//!
//! The types provide separate methods for interpreting that byte:
//!
//! - [`Address::kind`] reports what an address is used for.
//! - [`Address::expects_response`] reports the general response rule for that
//!   class.
//! - [`Control::communication_type`] maps recognized control values to a
//!   [`CommunicationType`] and reports all others as
//!   [`CommunicationType::Unsupported`].
//! - the control flag accessors expose the FCB, FCV, ACD, and DFC bits for the
//!   direction in which they apply.
//!
//! Because every byte can be represented, creating an [`Address`] or [`Control`]
//! alone does not assert that it can be sent. Frame constructors apply the
//! control-format compatibility checks required by the supported codecs.
//! The caller must still decide whether an address is right for the operation
//! and keep track of message order.
//!
//! # Example
//!
//! Inspect fields independently before constructing or processing a frame:
//!
//! ```
//! use meterbus_wired_datalink::{
//!     Address, AddressKind, CommunicationType, Control, Direction,
//! };
//!
//! let address = Address::new(1);
//! assert_eq!(address.kind(), AddressKind::Primary);
//! assert!(address.expects_response());
//!
//! let control = Control::req_ud2(false);
//! assert_eq!(control.communication_type(), CommunicationType::ReqUd2);
//! assert_eq!(control.direction(), Some(Direction::MasterToSlave));
//! assert_eq!(control.frame_count_bit(), Some(false));
//! assert_eq!(control.frame_count_valid(), Some(true));
//! assert_eq!(control.access_demand(), None);
//! ```
//!
//! These types describe individual bytes, not a complete exchange. Callers
//! still manage addressing rules, message order, and application data.

mod address;
mod control;

pub use address::{Address, AddressKind};
pub use control::{CommunicationType, Control, ControlError, Direction};
