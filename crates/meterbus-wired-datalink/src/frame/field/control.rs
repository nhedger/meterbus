//! Wired M-Bus data-link control fields and communication types.
//!
//! The one-byte control field, commonly called the C-field, identifies the
//! data-link communication carried by a structured frame. It also carries two
//! direction-dependent flags. In messages from a master, those flags are the
//! frame count bit (FCB) and frame count valid bit (FCV). In messages from a
//! slave, the same bit positions are access demand (ACD) and data flow control
//! (DFC).
//!
//! [`Control`] preserves any `u8` received from the wire. Its methods explain
//! known values and expose their flags. Unknown values remain available for
//! error reporting. [`CommunicationType`] names the recognized message type, and
//! [`ControlError`] reports a control value used with the wrong frame format.
//!
//! # Bit layout
//!
//! The field is interpreted as follows for the communications represented by
//! this crate:
//!
//! ```text
//! bit:   7   6   5   4   3   2   1   0
//!      +---+---+---+---+---+---+---+---+
//!      | 0 | D | X | Y | function code |
//!      +---+---+---+---+---+---+---+---+
//! ```
//!
//! `D` distinguishes the message direction:
//!
//! | `D` | Direction | `X` (bit 5) | `Y` (bit 4) |
//! | --- | --- | --- | --- |
//! | `1` | Master to slave | FCB | FCV |
//! | `0` | Slave to master | ACD | DFC |
//!
//! [`Control::direction`] identifies `D`. Direction-specific flag
//! accessors return [`Some`] only when their interpretation applies and
//! [`None`] for the opposite direction. This avoids reporting, for example,
//! an ACD value from a master request merely because bit 5 is set.
//!
//! Bit 7 is clear for every communication supported here. The low four bits
//! carry the function code. Classification uses the complete byte rather than
//! only the function code because direction and required flag combinations
//! are part of the supported control values.
//!
//! # Supported control values
//!
//! | C-field | [`CommunicationType`] | Direction | Format | Flag state |
//! | --- | --- | --- | --- | --- |
//! | `0x40` | [`CommunicationType::SndNke`] | Master to slave | Short | FCB = 0, FCV = 0 |
//! | `0x5a`, `0x7a` | [`CommunicationType::ReqUd1`] | Master to slave | Short | FCV = 1; FCB = 0 or 1 |
//! | `0x5b`, `0x7b` | [`CommunicationType::ReqUd2`] | Master to slave | Short | FCV = 1; FCB = 0 or 1 |
//! | `0x53`, `0x73` | [`CommunicationType::SndUd`] | Master to slave | Variable | FCV = 1; bit 5 = 0 or 1 |
//! | `0x43` | [`CommunicationType::SndUd2`] | Master to slave | Variable | FCB = 0, FCV = 0 |
//! | `0x08`, `0x18`, `0x28`, `0x38` | [`CommunicationType::RspUd`] | Slave to master | Variable | ACD and DFC independently clear or set |
//!
//! Every other byte maps to [`CommunicationType::Unsupported`], while [`Control`]
//! still preserves the original value.
//!
//! # Frame compatibility
//!
//! Short frames support SND-NKE, REQ-UD1, and REQ-UD2. Variable frames support
//! SND-UD, SND-UD2, and RSP-UD. Frame constructors reject other combinations
//! with a [`ControlError`].
//!
//! [`ShortFrame::new`]: crate::ShortFrame::new
//! [`ControlFrame::new`]: crate::ControlFrame::new
//! [`LongFrame::new`]: crate::LongFrame::new
//!
//! For master messages, bit 5 is FCB and bit 4 is FCV. For slave messages, the
//! same bits are ACD and DFC. The accessors return [`None`] when a flag does not
//! apply to that direction.
//!
//! # Examples
//!
//! Inspect a REQ-UD2 control field sent by a master:
//!
//! ```
//! use meterbus_wired_datalink::{CommunicationType, Control, Direction};
//!
//! let control = Control::req_ud2(true);
//! assert_eq!(control.value(), 0x7b);
//! assert_eq!(control.communication_type(), CommunicationType::ReqUd2);
//! assert_eq!(control.direction(), Some(Direction::MasterToSlave));
//! assert_eq!(control.frame_count_bit(), Some(true));
//! assert_eq!(control.frame_count_valid(), Some(true));
//! assert_eq!(control.access_demand(), None);
//! ```
//!
//! Callers still maintain FCB state, act on ACD and DFC, and decide which reply
//! is valid for a request.

use core::fmt;

/// A data-link control field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Control(u8);

impl Control {
    /// Creates a control field from its wire value.
    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    /// Creates an SND-NKE control field.
    #[must_use]
    pub const fn snd_nke() -> Self {
        Self(0x40)
    }

    /// Creates a REQ-UD1 control field with the requested frame-count bit.
    #[must_use]
    pub const fn req_ud1(fcb: bool) -> Self {
        Self(0x5a | ((fcb as u8) << 5))
    }

    /// Creates a REQ-UD2 control field with the requested frame-count bit.
    #[must_use]
    pub const fn req_ud2(fcb: bool) -> Self {
        Self(0x5b | ((fcb as u8) << 5))
    }

    /// Creates an SND-UD control field with bit 5 clear or set.
    #[must_use]
    pub const fn snd_ud(bit5: bool) -> Self {
        Self(0x53 | ((bit5 as u8) << 5))
    }

    /// Creates an SND-UD2 control field.
    #[must_use]
    pub const fn snd_ud2() -> Self {
        Self(0x43)
    }

    /// Creates an RSP-UD control field with access-demand and data-flow-control flags.
    #[must_use]
    pub const fn rsp_ud(acd: bool, dfc: bool) -> Self {
        Self(0x08 | ((acd as u8) << 5) | ((dfc as u8) << 4))
    }

    /// Returns the wire value.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }

    /// Returns the supported communication type represented by this field.
    #[must_use]
    pub const fn communication_type(self) -> CommunicationType {
        match self.0 {
            0x40 => CommunicationType::SndNke,
            0x43 => CommunicationType::SndUd2,
            0x53 | 0x73 => CommunicationType::SndUd,
            0x5a | 0x7a => CommunicationType::ReqUd1,
            0x5b | 0x7b => CommunicationType::ReqUd2,
            0x08 | 0x18 | 0x28 | 0x38 => CommunicationType::RspUd,
            _ => CommunicationType::Unsupported,
        }
    }

    /// Returns the direction of a supported communication.
    #[must_use]
    pub const fn direction(self) -> Option<Direction> {
        match self.communication_type() {
            CommunicationType::SndNke
            | CommunicationType::ReqUd1
            | CommunicationType::ReqUd2
            | CommunicationType::SndUd
            | CommunicationType::SndUd2 => Some(Direction::MasterToSlave),
            CommunicationType::RspUd => Some(Direction::SlaveToMaster),
            CommunicationType::Unsupported => None,
        }
    }

    /// Returns the frame-count bit for a message sent by a master.
    #[must_use]
    pub const fn frame_count_bit(self) -> Option<bool> {
        if matches!(self.direction(), Some(Direction::MasterToSlave)) {
            Some(self.0 & 0x20 != 0)
        } else {
            None
        }
    }

    /// Returns the frame-count-valid bit for a message sent by a master.
    #[must_use]
    pub const fn frame_count_valid(self) -> Option<bool> {
        if matches!(self.direction(), Some(Direction::MasterToSlave)) {
            Some(self.0 & 0x10 != 0)
        } else {
            None
        }
    }

    /// Returns the access-demand bit for a message sent by a slave.
    #[must_use]
    pub const fn access_demand(self) -> Option<bool> {
        if matches!(self.direction(), Some(Direction::SlaveToMaster)) {
            Some(self.0 & 0x20 != 0)
        } else {
            None
        }
    }

    /// Returns the data-flow-control bit for a message sent by a slave.
    #[must_use]
    pub const fn data_flow_control(self) -> Option<bool> {
        if matches!(self.direction(), Some(Direction::SlaveToMaster)) {
            Some(self.0 & 0x10 != 0)
        } else {
            None
        }
    }

    pub(crate) const fn validate_short_frame(self) -> Result<(), ControlError> {
        if matches!(
            self.communication_type(),
            CommunicationType::SndNke | CommunicationType::ReqUd1 | CommunicationType::ReqUd2
        ) {
            Ok(())
        } else {
            Err(ControlError::InvalidForShortFrame { value: self.0 })
        }
    }

    pub(crate) const fn validate_variable_frame(self) -> Result<(), ControlError> {
        if matches!(
            self.communication_type(),
            CommunicationType::SndUd | CommunicationType::SndUd2 | CommunicationType::RspUd
        ) {
            Ok(())
        } else {
            Err(ControlError::InvalidForVariableFrame { value: self.0 })
        }
    }
}

/// Direction of a supported data-link communication.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Direction {
    /// A request or command sent by a master.
    MasterToSlave,
    /// A response sent by a slave.
    SlaveToMaster,
}

impl From<u8> for Control {
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

/// A supported wired M-Bus communication type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CommunicationType {
    /// Normalize the link layer.
    SndNke,
    /// Request time-critical data.
    ReqUd1,
    /// Request standard data.
    ReqUd2,
    /// Send user data.
    SndUd,
    /// Send user data and request a response.
    SndUd2,
    /// Respond with user data.
    RspUd,
    /// A control value unsupported by wired M-Bus.
    Unsupported,
}

/// Error produced when a control field is incompatible with a frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlError {
    /// The control value cannot be used in a short frame.
    InvalidForShortFrame {
        /// The rejected control-field value.
        value: u8,
    },
    /// The control value cannot be used in a variable-format frame.
    InvalidForVariableFrame {
        /// The rejected control-field value.
        value: u8,
    },
}

impl fmt::Display for ControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidForShortFrame { value } => write!(
                formatter,
                "control value 0x{value:02x} is invalid for a short frame"
            ),
            Self::InvalidForVariableFrame { value } => write!(
                formatter,
                "control value 0x{value:02x} is invalid for a variable-format frame"
            ),
        }
    }
}

impl core::error::Error for ControlError {}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::string::ToString;

    #[test]
    fn classifies_supported_communication_types() {
        let cases = [
            (0x40, CommunicationType::SndNke),
            (0x43, CommunicationType::SndUd2),
            (0x53, CommunicationType::SndUd),
            (0x73, CommunicationType::SndUd),
            (0x5a, CommunicationType::ReqUd1),
            (0x7a, CommunicationType::ReqUd1),
            (0x5b, CommunicationType::ReqUd2),
            (0x7b, CommunicationType::ReqUd2),
            (0x08, CommunicationType::RspUd),
            (0x18, CommunicationType::RspUd),
            (0x28, CommunicationType::RspUd),
            (0x38, CommunicationType::RspUd),
            (0xff, CommunicationType::Unsupported),
        ];

        for (value, communication_type) in cases {
            let control = Control::from(value);
            assert_eq!(control.value(), value);
            assert_eq!(control.communication_type(), communication_type);
        }
    }

    #[test]
    fn exposes_master_and_slave_flags() {
        let master = Control::new(0x7b);
        assert_eq!(master.direction(), Some(Direction::MasterToSlave));
        assert_eq!(master.frame_count_bit(), Some(true));
        assert_eq!(master.frame_count_valid(), Some(true));
        assert_eq!(master.access_demand(), None);
        assert_eq!(master.data_flow_control(), None);

        let slave = Control::new(0x38);
        assert_eq!(slave.direction(), Some(Direction::SlaveToMaster));
        assert_eq!(slave.frame_count_bit(), None);
        assert_eq!(slave.frame_count_valid(), None);
        assert_eq!(slave.access_demand(), Some(true));
        assert_eq!(slave.data_flow_control(), Some(true));
    }

    #[test]
    fn constructors_produce_named_control_values() {
        assert_eq!(Control::snd_nke().value(), 0x40);
        assert_eq!(Control::req_ud1(false).value(), 0x5a);
        assert_eq!(Control::req_ud1(true).value(), 0x7a);
        assert_eq!(Control::req_ud2(false).value(), 0x5b);
        assert_eq!(Control::req_ud2(true).value(), 0x7b);
        assert_eq!(Control::snd_ud(false).value(), 0x53);
        assert_eq!(Control::snd_ud(true).value(), 0x73);
        assert_eq!(Control::snd_ud2().value(), 0x43);
        assert_eq!(Control::rsp_ud(false, false).value(), 0x08);
        assert_eq!(Control::rsp_ud(false, true).value(), 0x18);
        assert_eq!(Control::rsp_ud(true, false).value(), 0x28);
        assert_eq!(Control::rsp_ud(true, true).value(), 0x38);
    }

    #[test]
    fn all_control_bytes_expose_flags_only_for_supported_direction() {
        for value in u8::MIN..=u8::MAX {
            let control = Control::new(value);
            match control.direction() {
                Some(Direction::MasterToSlave) => {
                    assert!(control.frame_count_bit().is_some());
                    assert!(control.frame_count_valid().is_some());
                    assert_eq!(control.access_demand(), None);
                    assert_eq!(control.data_flow_control(), None);
                }
                Some(Direction::SlaveToMaster) => {
                    assert_eq!(control.frame_count_bit(), None);
                    assert_eq!(control.frame_count_valid(), None);
                    assert!(control.access_demand().is_some());
                    assert!(control.data_flow_control().is_some());
                }
                None => {
                    assert_eq!(control.frame_count_bit(), None);
                    assert_eq!(control.frame_count_valid(), None);
                    assert_eq!(control.access_demand(), None);
                    assert_eq!(control.data_flow_control(), None);
                }
            }
        }
    }

    #[test]
    fn validates_frame_compatibility() {
        assert_eq!(Control::new(0x40).validate_short_frame(), Ok(()));
        assert_eq!(Control::new(0x53).validate_variable_frame(), Ok(()));

        let error = Control::new(0x53).validate_short_frame().unwrap_err();
        assert_eq!(error, ControlError::InvalidForShortFrame { value: 0x53 });
        #[cfg(feature = "alloc")]
        assert_eq!(
            error.to_string(),
            "control value 0x53 is invalid for a short frame"
        );
        assert_eq!(
            Control::new(0x40).validate_variable_frame(),
            Err(ControlError::InvalidForVariableFrame { value: 0x40 })
        );
    }
}
