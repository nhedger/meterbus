//! Build and encode a standard meter-readout request without allocation.
//!
//! The example creates a `REQ-UD2` short frame for slave address 1 and writes
//! it into a fixed-size caller-owned buffer.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p meterbus-wired-datalink --example request_data_no_alloc
//! ```

use meterbus_wired_datalink::{Address, Control, ShortFrame, ShortFrameError};

fn main() -> Result<(), ShortFrameError> {
    let request = ShortFrame::new(Control::req_ud2(false), Address::new(1))?;

    let mut output = [0_u8; ShortFrame::LEN];
    let encoded = request.encode_into(&mut output)?;

    assert_eq!(encoded, [0x10, 0x5b, 0x01, 0x5c, 0x16]);
    Ok(())
}
