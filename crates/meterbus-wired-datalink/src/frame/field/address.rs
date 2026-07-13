//! Wired M-Bus configured-slave and special-purpose data-link addresses.
//!
//! Every structured wired M-Bus frame contains a one-byte address field. Most
//! values identify one configured slave, while values near the ends of the
//! address space select protocol-defined modes such as unconfigured-slave
//! access, selected-slave addressing, diagnosis, and broadcast.
//!
//! [`Address`] preserves the byte read from the wire. [`AddressKind`] explains
//! how that value is used. Keeping both lets decoders and diagnostic tools
//! report reserved or unexpected addresses clearly.
//!
//! # Address space
//!
//! | Value | [`AddressKind`] | Meaning | Response generally expected |
//! | --- | --- | --- | --- |
//! | `0` | [`AddressKind::Unconfigured`] | All unconfigured slaves | Yes |
//! | `1..=250` | [`AddressKind::Primary`] | One configured slave | Yes |
//! | `251` | [`AddressKind::PrimaryMasterRepeater`] | Primary-master repeater management | Yes |
//! | `252` | [`AddressKind::Reserved`] | Reserved | No |
//! | `253` | [`AddressKind::Secondary`] | Previously selected slave or slaves | Yes |
//! | `254` | [`AddressKind::Test`] | Test and diagnosis | Yes |
//! | `255` | [`AddressKind::Broadcast`] | All slaves | No |
//!
//! [`Address::expects_response`] implements the final column: it returns
//! `false` only for the reserved and broadcast classes. It describes the
//! address-level rule, not a guarantee that a particular request has a reply.
//! The communication type, device capabilities, bus state, and transmission
//! success can all affect whether a response is actually observed.
//!
//! Address 253 refers to slaves selected by an earlier application-layer
//! operation; it does not contain their identity. Address 255 is broadcast, so
//! slaves do not reply. Reserved values remain representable for diagnostics.
//!
//! # Examples
//!
//! Classify an ordinary configured-slave address:
//!
//! ```
//! use meterbus_wired_datalink::{Address, AddressKind};
//!
//! let address = Address::new(42);
//! assert_eq!(address.value(), 42);
//! assert_eq!(address.kind(), AddressKind::Primary);
//! assert!(address.expects_response());
//! ```
//!
//! Detect a broadcast before waiting for a reply:
//!
//! ```
//! use meterbus_wired_datalink::{Address, AddressKind};
//!
//! let address = Address::from(0xff);
//! assert_eq!(address.kind(), AddressKind::Broadcast);
//! assert!(!address.expects_response());
//! ```
//!
//! Frame constructors do not reject reserved or special-purpose addresses. The
//! caller decides whether an address is valid for the current operation.

/// A data-link address.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Address(u8);

impl Address {
    /// Creates an address from its wire value.
    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    /// Returns the wire value.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }

    /// Classifies the address according to EN 13757-2.
    #[must_use]
    pub const fn kind(self) -> AddressKind {
        match self.0 {
            0 => AddressKind::Unconfigured,
            1..=250 => AddressKind::Primary,
            251 => AddressKind::PrimaryMasterRepeater,
            252 => AddressKind::Reserved,
            253 => AddressKind::Secondary,
            254 => AddressKind::Test,
            255 => AddressKind::Broadcast,
        }
    }

    /// Returns whether a slave response may be expected.
    #[must_use]
    pub const fn expects_response(self) -> bool {
        !matches!(self.kind(), AddressKind::Reserved | AddressKind::Broadcast)
    }
}

impl From<u8> for Address {
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

/// The protocol-defined use of a data-link address.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AddressKind {
    /// Address 0, used by unconfigured slaves.
    Unconfigured,
    /// Configured slave address in the range 1 through 250.
    Primary,
    /// Address 251, used for primary-master repeater management.
    PrimaryMasterRepeater,
    /// Reserved address 252.
    Reserved,
    /// Address 253, used to address a previously selected slave.
    Secondary,
    /// Address 254, used for testing and diagnosis.
    Test,
    /// Broadcast address 255.
    Broadcast,
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn classifies_every_address_range() {
        let cases = [
            (0, AddressKind::Unconfigured, true),
            (1, AddressKind::Primary, true),
            (250, AddressKind::Primary, true),
            (251, AddressKind::PrimaryMasterRepeater, true),
            (252, AddressKind::Reserved, false),
            (253, AddressKind::Secondary, true),
            (254, AddressKind::Test, true),
            (255, AddressKind::Broadcast, false),
        ];

        for (value, kind, expects_response) in cases {
            let address = Address::from(value);
            assert_eq!(address.value(), value);
            assert_eq!(address.kind(), kind);
            assert_eq!(address.expects_response(), expects_response);
        }
    }

    #[test]
    fn reserved_address_remains_representable() {
        assert_eq!(Address::new(252).value(), 252);
        assert_eq!(Address::new(252).kind(), AddressKind::Reserved);
    }
}
