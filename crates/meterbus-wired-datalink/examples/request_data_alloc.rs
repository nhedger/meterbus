//! Build and encode a standard meter-readout request into a vector.
//!
//! This is the allocating counterpart to `request_data_no_alloc`. It requires
//! the crate's `alloc` feature.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p meterbus-wired-datalink --example request_data_alloc --features alloc
//! ```

use meterbus_wired_datalink::{Address, Control, ShortFrame, ShortFrameError};

fn main() -> Result<(), ShortFrameError> {
    let request = ShortFrame::new(Control::req_ud2(false), Address::new(1))?;
    let encoded = request.encode();

    assert_eq!(encoded, [0x10, 0x5b, 0x01, 0x5c, 0x16]);
    Ok(())
}
