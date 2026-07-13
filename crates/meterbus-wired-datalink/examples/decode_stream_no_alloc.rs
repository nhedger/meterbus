//! Decode arbitrary byte chunks into caller-provided output slots.
//!
//! `StreamDecoder::push_into` retains incomplete trailing input and writes
//! complete frames and recovery events without using an allocator.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p meterbus-wired-datalink --example decode_stream_no_alloc
//! ```

use meterbus_wired_datalink::{
    Frame,
    decoder::stream::{PushIntoError, RecoveryEvent, StreamDecoder},
};

fn main() -> Result<(), PushIntoError> {
    let mut decoder = StreamDecoder::new();
    let mut frames: [Option<Frame>; 2] = core::array::from_fn(|_| None);
    let mut recoveries: [Option<RecoveryEvent>; 1] = [None];
    let input = [0xe5, 0xa2];

    let outcome = decoder.push_into(&input, &mut frames, &mut recoveries)?;

    assert_eq!(outcome.consumed, input.len());
    assert_eq!(outcome.frames_written, 2);
    assert!(matches!(frames[0], Some(Frame::Ack(_))));
    assert!(matches!(frames[1], Some(Frame::Nack(_))));
    Ok(())
}
