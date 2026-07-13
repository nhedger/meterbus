//! Decode and re-encode exactly one frame without allocation.
//!
//! The exact decoder requires a slice containing one complete frame. The
//! decoded frame is written back into a fixed-size caller-owned buffer.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p meterbus-wired-datalink --example decode_exact_no_alloc
//! ```

use meterbus_wired_datalink::{
    CommunicationType, Frame, LongFrame,
    decoder::exact::{DecodeError, decode},
};

fn main() -> Result<(), DecodeError> {
    let frame = decode(&[0x10, 0x5b, 0x01, 0x5c, 0x16])?;

    if let Frame::Short(short) = &frame {
        assert_eq!(
            short.control().communication_type(),
            CommunicationType::ReqUd2
        );
        assert_eq!(short.address().value(), 1);
    }

    let mut output = [0_u8; LongFrame::MAX_LEN];
    assert_eq!(
        frame
            .encode_into(&mut output)
            .expect("maximum frame buffer is large enough"),
        [0x10, 0x5b, 0x01, 0x5c, 0x16]
    );

    Ok(())
}
