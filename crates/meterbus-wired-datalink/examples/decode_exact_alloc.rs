//! Decode exactly one frame and re-encode it into a vector.
//!
//! This is the allocating counterpart to `decode_exact_no_alloc`. It requires
//! the crate's `alloc` feature.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p meterbus-wired-datalink --example decode_exact_alloc --features alloc
//! ```

use meterbus_wired_datalink::{
    CommunicationType, Frame,
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

    assert_eq!(frame.encode(), [0x10, 0x5b, 0x01, 0x5c, 0x16]);
    Ok(())
}
