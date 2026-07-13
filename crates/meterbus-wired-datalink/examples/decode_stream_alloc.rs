//! Decode arbitrary byte chunks and collect the output into vectors.
//!
//! The example covers split and adjacent frames, then enables resynchronizing
//! recovery to continue after malformed bytes. It requires the `alloc` feature.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p meterbus-wired-datalink --example decode_stream_alloc --features alloc
//! ```

use meterbus_wired_datalink::{
    Frame,
    decoder::stream::{Recovery, StreamDecoder},
};

fn main() {
    let mut decoder = StreamDecoder::new();

    let first = decoder
        .push(&[0x10, 0x5b])
        .expect("the first chunk is valid");
    assert!(first.frames.is_empty());

    let second = decoder
        .push(&[0x01, 0x5c, 0x16, 0xe5])
        .expect("the second chunk is valid");
    assert!(matches!(
        second.frames.as_slice(),
        [Frame::Short(_), Frame::Ack(_)]
    ));
    decoder.finish().expect("no incomplete trailing frame");

    let mut decoder = StreamDecoder::with_recovery(Recovery::Resync);
    let outcome = decoder
        .push(&[0xff, 0x00, 0xe5])
        .expect("resync reports malformed bytes as recovery events");
    assert!(matches!(outcome.frames.as_slice(), [Frame::Ack(_)]));
    assert_eq!(outcome.recoveries[0].discarded(), [0xff, 0x00]);
}
