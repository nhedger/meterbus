//! Incremental decoding of wired M-Bus frames.
//!
//! [`StreamDecoder`] accepts arbitrary byte chunks and retains only an
//! incomplete trailing frame. Complete frames are written immediately to
//! caller-provided output slots.
//!
//! [`StreamDecoder::push_into`] works without allocation. Its
//! [`PushIntoOutcome`] reports how many input bytes were consumed and how many
//! output slots were filled. If an output slice fills, pass the unconsumed
//! suffix to the next call.
//!
//! With the `alloc` feature, `StreamDecoder::push` collects all output into
//! vectors for convenience.
//!
//! # Allocation-free example
//!
//! ```
//! use meterbus_wired_datalink::{
//!     Frame,
//!     decoder::stream::{RecoveryEvent, StreamDecoder},
//! };
//!
//! # fn main() -> Result<(), meterbus_wired_datalink::decoder::stream::PushIntoError> {
//! let mut decoder = StreamDecoder::new();
//! let mut frames: [Option<Frame>; 2] = core::array::from_fn(|_| None);
//! let mut recoveries: [Option<RecoveryEvent>; 1] = [None];
//! let bytes = [0xe5, 0xa2];
//!
//! let outcome = decoder.push_into(&bytes, &mut frames, &mut recoveries)?;
//! assert_eq!(outcome.consumed, bytes.len());
//! assert_eq!(outcome.frames_written, 2);
//! assert!(matches!(frames[0], Some(Frame::Ack(_))));
//! assert!(matches!(frames[1], Some(Frame::Nack(_))));
//! # Ok(())
//! # }
//! ```
//!
//! # Recovery
//!
//! [`Recovery::Strict`] is the default. It clears buffered state and returns
//! the first decoding error. [`Recovery::Resync`] discards malformed bytes and
//! continues at the next complete valid frame.
//!
//! In resync mode, `push_into` writes a [`RecoveryEvent`] for each discarded
//! region. One event stores up to [`MAX_RECOVERY_BYTES`]; longer regions are
//! split across consecutive events. If the recovery output fills, retry the
//! unconsumed input with fresh output slots.
//!
//! Call [`StreamDecoder::finish`] when no more input will arrive. It reports an
//! incomplete trailing frame and clears the decoder.

use core::fmt;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{
    AckFrame, ControlFrame, ControlFrameError, Frame, LongFrame, LongFrameError, NackFrame,
    ShortFrame,
    decoder::exact::{self, DecodeError},
};

/// Maximum number of discarded bytes stored in one [`RecoveryEvent`].
///
/// Longer discarded regions are split across events, allowing recovery to
/// remain allocation-free.
pub const MAX_RECOVERY_BYTES: usize = 32;

/// Strategy used after malformed input.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Recovery {
    /// Clear buffered state and return the decoding error.
    #[default]
    Strict,
    /// Discard malformed bytes and continue at the next complete valid frame.
    Resync,
}

/// Progress made by [`StreamDecoder::push_into`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PushIntoOutcome {
    /// Bytes consumed from the supplied chunk.
    pub consumed: usize,
    /// Frame output slots filled from the beginning of the supplied slice.
    pub frames_written: usize,
    /// Recovery output slots filled from the beginning of the supplied slice.
    pub recoveries_written: usize,
}

/// Error from [`StreamDecoder::push_into`], including progress made before the
/// malformed frame was reached.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PushIntoError {
    /// The malformed frame error.
    pub error: DecodeError,
    /// Input and output progress completed before the error.
    pub outcome: PushIntoOutcome,
}

impl fmt::Display for PushIntoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(formatter)
    }
}

impl core::error::Error for PushIntoError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// A successful allocating call to [`StreamDecoder::push`].
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PushOutcome {
    /// Bytes consumed from the supplied chunk.
    pub consumed: usize,
    /// Complete frames decoded from the input.
    pub frames: Vec<Frame>,
    /// Recoveries performed while decoding the input.
    pub recoveries: Vec<RecoveryEvent>,
}

/// Error from [`StreamDecoder::push`], including all output produced before
/// the malformed frame was reached.
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushError {
    /// The malformed frame error.
    pub error: DecodeError,
    /// Progress and output completed before the error.
    pub outcome: PushOutcome,
}

#[cfg(feature = "alloc")]
impl fmt::Display for PushError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(formatter)
    }
}

#[cfg(feature = "alloc")]
impl core::error::Error for PushError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Malformed bytes discarded while resynchronizing.
///
/// Each event owns at most [`MAX_RECOVERY_BYTES`]. Consecutive events represent
/// longer discarded regions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryEvent {
    /// Error that triggered this recovery step.
    pub error: DecodeError,
    discarded: [u8; MAX_RECOVERY_BYTES],
    discarded_len: u8,
}

impl RecoveryEvent {
    /// Returns the bytes discarded by this event.
    #[must_use]
    pub fn discarded(&self) -> &[u8] {
        &self.discarded[..usize::from(self.discarded_len)]
    }
}

/// Error returned when a stream ends with trailing input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IncompleteFrameError {
    /// Number of bytes present when the stream ended.
    pub received_bytes: usize,
    /// Expected frame length, if enough header bytes were available.
    pub expected_length: Option<usize>,
}

impl fmt::Display for IncompleteFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.expected_length {
            Some(expected) => write!(
                formatter,
                "incomplete frame: expected {expected} bytes, received {}",
                self.received_bytes
            ),
            None => write!(
                formatter,
                "incomplete frame: received {} bytes before the stream ended",
                self.received_bytes
            ),
        }
    }
}

impl core::error::Error for IncompleteFrameError {}

/// Stateful incremental frame decoder with fixed internal storage.
#[derive(Clone, Debug)]
pub struct StreamDecoder {
    buffer: [u8; LongFrame::MAX_LEN],
    buffered: usize,
    recovery_discard: usize,
    recovery: Recovery,
}

impl Default for StreamDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamDecoder {
    /// Creates a strict stream decoder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: [0; LongFrame::MAX_LEN],
            buffered: 0,
            recovery_discard: 0,
            recovery: Recovery::Strict,
        }
    }

    /// Creates a stream decoder using `recovery`.
    #[must_use]
    pub const fn with_recovery(recovery: Recovery) -> Self {
        Self {
            buffer: [0; LongFrame::MAX_LEN],
            buffered: 0,
            recovery_discard: 0,
            recovery,
        }
    }

    /// Returns the number of incomplete bytes retained between pushes.
    #[must_use]
    pub const fn buffered_bytes(&self) -> usize {
        self.buffered
    }

    /// Decodes `chunk` into caller-provided output slots.
    ///
    /// The result reports how much input and output was consumed. If a needed
    /// output slice has no free slot, decoding stops before consuming the bytes
    /// requiring that output; call again with `&chunk[outcome.consumed..]`.
    /// A zero-length frame slice still permits buffering incomplete input and
    /// recovery, while a zero-length recovery slice only blocks resynchronizing
    /// malformed input. Complete frames are never retained internally.
    ///
    /// Existing values in output slots may be replaced; filled slots always
    /// form a prefix of each output slice.
    ///
    /// # Errors
    ///
    /// In strict mode, returns the first malformed frame error together with
    /// progress made before that frame, then clears all buffered state.
    /// Resynchronizing mode writes errors as recovery events.
    pub fn push_into(
        &mut self,
        chunk: &[u8],
        frames: &mut [Option<Frame>],
        recoveries: &mut [Option<RecoveryEvent>],
    ) -> Result<PushIntoOutcome, PushIntoError> {
        let mut outcome = PushIntoOutcome::default();

        loop {
            let input = &chunk[outcome.consumed..];
            let total = self.buffered + input.len();
            if total == 0 {
                return Ok(outcome);
            }

            let length = match combined_frame_length(&self.buffer[..self.buffered], input, 0) {
                Ok(length) => length,
                Err(error) => {
                    if self.recovery == Recovery::Strict {
                        self.reset();
                        return Err(PushIntoError { error, outcome });
                    }
                    if outcome.recoveries_written == recoveries.len() {
                        return Ok(outcome);
                    }
                    let discarded = self.recovery_length(input);
                    let event = self.recovery_event(input, discarded, error);
                    recoveries[outcome.recoveries_written] = Some(event);
                    outcome.recoveries_written += 1;
                    outcome.consumed += self.discard_prefix(discarded);
                    continue;
                }
            };

            let Some(length) = length else {
                self.buffer_input(input);
                outcome.consumed = chunk.len();
                return Ok(outcome);
            };
            if total < length {
                self.buffer_input(input);
                outcome.consumed = chunk.len();
                return Ok(outcome);
            }
            if outcome.frames_written == frames.len() {
                return Ok(outcome);
            }

            let buffered = self.buffered;
            let decoded = if buffered == 0 {
                exact::decode(&input[..length])
            } else if buffered >= length {
                exact::decode(&self.buffer[..length])
            } else {
                let needed = length - buffered;
                self.buffer[buffered..length].copy_from_slice(&input[..needed]);
                exact::decode(&self.buffer[..length])
            };
            match decoded {
                Ok(frame) => {
                    self.recovery_discard = 0;
                    frames[outcome.frames_written] = Some(frame);
                    outcome.frames_written += 1;
                    outcome.consumed += self.discard_prefix(length);
                }
                Err(error) => {
                    if self.recovery == Recovery::Strict {
                        self.reset();
                        return Err(PushIntoError { error, outcome });
                    }
                    if outcome.recoveries_written == recoveries.len() {
                        return Ok(outcome);
                    }
                    let discarded = self.recovery_length(input);
                    let event = self.recovery_event(input, discarded, error);
                    recoveries[outcome.recoveries_written] = Some(event);
                    outcome.recoveries_written += 1;
                    outcome.consumed += self.discard_prefix(discarded);
                }
            }
        }
    }

    /// Adds a chunk and allocates vectors for all decoded output.
    ///
    /// # Errors
    ///
    /// In strict mode, returns the first malformed frame error together with
    /// every frame, recovery, and consumed byte completed before that frame,
    /// then clears all buffered state.
    #[cfg(feature = "alloc")]
    pub fn push(&mut self, chunk: &[u8]) -> Result<PushOutcome, PushError> {
        const BATCH: usize = 8;
        let mut outcome = PushOutcome::default();
        let mut consumed = 0;
        loop {
            let mut frames: [Option<Frame>; BATCH] = core::array::from_fn(|_| None);
            let mut recoveries = [None; BATCH];
            let progress = match self.push_into(&chunk[consumed..], &mut frames, &mut recoveries) {
                Ok(progress) => progress,
                Err(error) => {
                    outcome.frames.extend(
                        frames
                            .into_iter()
                            .take(error.outcome.frames_written)
                            .flatten(),
                    );
                    outcome.recoveries.extend(
                        recoveries
                            .into_iter()
                            .take(error.outcome.recoveries_written)
                            .flatten(),
                    );
                    outcome.consumed = consumed + error.outcome.consumed;
                    return Err(PushError {
                        error: error.error,
                        outcome,
                    });
                }
            };
            outcome
                .frames
                .extend(frames.into_iter().take(progress.frames_written).flatten());
            outcome.recoveries.extend(
                recoveries
                    .into_iter()
                    .take(progress.recoveries_written)
                    .flatten(),
            );
            consumed += progress.consumed;
            outcome.consumed = consumed;
            if consumed == chunk.len() {
                return Ok(outcome);
            }
        }
    }

    /// Completes the stream, rejecting and clearing trailing input.
    ///
    /// # Errors
    ///
    /// Returns [`IncompleteFrameError`] when any buffered bytes remain.
    pub fn finish(&mut self) -> Result<(), IncompleteFrameError> {
        if self.buffered == 0 {
            return Ok(());
        }
        let error = IncompleteFrameError {
            received_bytes: self.buffered,
            expected_length: combined_frame_length(&self.buffer[..self.buffered], &[], 0)
                .ok()
                .flatten(),
        };
        self.reset();
        Err(error)
    }

    /// Discards all buffered input.
    pub fn reset(&mut self) {
        self.buffered = 0;
        self.recovery_discard = 0;
    }

    fn buffer_input(&mut self, input: &[u8]) {
        self.buffer[self.buffered..self.buffered + input.len()].copy_from_slice(input);
        self.buffered += input.len();
    }

    fn discard_prefix(&mut self, count: usize) -> usize {
        if count < self.buffered {
            self.buffer.copy_within(count..self.buffered, 0);
            self.buffered -= count;
            0
        } else {
            let consumed = count - self.buffered;
            self.buffered = 0;
            consumed
        }
    }

    fn recovery_length(&mut self, input: &[u8]) -> usize {
        if self.recovery_discard != 0 {
            let count = self.recovery_discard.min(MAX_RECOVERY_BYTES);
            self.recovery_discard -= count;
            return count;
        }
        let total = self.buffered + input.len();
        let mut incomplete = None;
        let discard = (1..total)
            .find(
                |&offset| match candidate_at(&self.buffer[..self.buffered], input, offset) {
                    Candidate::Complete => true,
                    Candidate::Incomplete => {
                        incomplete.get_or_insert(offset);
                        false
                    }
                    Candidate::Invalid => false,
                },
            )
            .or(incomplete)
            .unwrap_or(total);
        let count = discard.min(MAX_RECOVERY_BYTES);
        self.recovery_discard = discard - count;
        count
    }

    fn recovery_event(&self, input: &[u8], count: usize, error: DecodeError) -> RecoveryEvent {
        let mut discarded = [0; MAX_RECOVERY_BYTES];
        copy_combined(
            &self.buffer[..self.buffered],
            input,
            0,
            &mut discarded[..count],
        );
        RecoveryEvent {
            error,
            discarded,
            discarded_len: count as u8,
        }
    }
}

fn combined_frame_length(
    prefix: &[u8],
    input: &[u8],
    offset: usize,
) -> Result<Option<usize>, DecodeError> {
    let available = prefix.len() + input.len() - offset;
    let byte = |index: usize| {
        let index = offset + index;
        (index < prefix.len())
            .then(|| prefix[index])
            .or_else(|| input.get(index - prefix.len()).copied())
    };
    let Some(start) = byte(0) else {
        return Ok(None);
    };
    match start {
        AckFrame::BYTE | NackFrame::BYTE => Ok(Some(1)),
        ShortFrame::START => Ok(Some(ShortFrame::LEN)),
        ControlFrame::START if available < 3 => Ok(None),
        ControlFrame::START => {
            let first = byte(1).expect("available header byte");
            let second = byte(2).expect("available header byte");
            if first != second {
                return Err(variable_error(
                    first,
                    ControlFrameError::InvalidDataLength {
                        index: 2,
                        actual: second,
                    },
                    LongFrameError::MismatchedDataLengths { first, second },
                ));
            }
            if first < ControlFrame::DATA_LEN {
                return Err(DecodeError::Long(LongFrameError::InvalidDataLength {
                    actual: first,
                }));
            }
            if let Some(actual) = byte(3) {
                if actual != ControlFrame::START {
                    return Err(variable_error(
                        first,
                        ControlFrameError::InvalidStart { index: 3, actual },
                        LongFrameError::InvalidStart { index: 3, actual },
                    ));
                }
            }
            Ok(Some(usize::from(first) + 6))
        }
        actual => Err(DecodeError::UnknownStart { actual }),
    }
}

fn variable_error(data_len: u8, control: ControlFrameError, long: LongFrameError) -> DecodeError {
    if data_len == ControlFrame::DATA_LEN {
        DecodeError::Control(control)
    } else {
        DecodeError::Long(long)
    }
}

enum Candidate {
    Invalid,
    Incomplete,
    Complete,
}

fn candidate_at(prefix: &[u8], input: &[u8], offset: usize) -> Candidate {
    let available = prefix.len() + input.len() - offset;
    let Ok(length) = combined_frame_length(prefix, input, offset) else {
        return Candidate::Invalid;
    };
    let Some(length) = length else {
        return Candidate::Incomplete;
    };
    if available < length {
        return Candidate::Incomplete;
    }
    let end = offset + length;
    let bytes = if end <= prefix.len() {
        &prefix[offset..end]
    } else if offset >= prefix.len() {
        &input[offset - prefix.len()..end - prefix.len()]
    } else {
        // A crossing candidate will become contiguous in the decoder buffer on
        // the next pass. Treat it as plausible so its start is preserved.
        return Candidate::Incomplete;
    };
    if exact::decode(bytes).is_ok() {
        Candidate::Complete
    } else {
        Candidate::Invalid
    }
}

fn copy_combined(prefix: &[u8], input: &[u8], offset: usize, output: &mut [u8]) {
    for (index, byte) in output.iter_mut().enumerate() {
        let source = offset + index;
        *byte = if source < prefix.len() {
            prefix[source]
        } else {
            input[source - prefix.len()]
        };
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::{Address, Control};
    #[cfg(feature = "alloc")]
    use alloc::string::ToString;

    const SHORT: &[u8] = &[0x10, 0x40, 0x01, 0x41, 0x16];

    fn outputs() -> ([Option<Frame>; 4], [Option<RecoveryEvent>; 4]) {
        (core::array::from_fn(|_| None), [None; 4])
    }

    #[test]
    fn push_into_buffers_split_frames() {
        let mut decoder = StreamDecoder::new();
        let (mut frames, mut recoveries) = outputs();
        let first = decoder
            .push_into(&SHORT[..2], &mut frames, &mut recoveries)
            .unwrap();
        assert_eq!(first.consumed, 2);
        assert_eq!(first.frames_written, 0);
        assert_eq!(decoder.buffered_bytes(), 2);
        let second = decoder
            .push_into(&SHORT[2..], &mut frames, &mut recoveries)
            .unwrap();
        assert_eq!(second.consumed, 3);
        assert!(matches!(frames[0], Some(Frame::Short(_))));
        assert_eq!(decoder.buffered_bytes(), 0);
    }

    #[test]
    fn every_frame_kind_decodes_at_every_split_boundary() {
        let cases: &[&[u8]] = &[
            &[0xe5],
            &[0xa2],
            SHORT,
            &[0x68, 3, 3, 0x68, 0x53, 0xfe, 0xbd, 0x0e, 0x16],
            &[0x68, 4, 4, 0x68, 0x53, 0xfe, 0x50, 0x10, 0xb1, 0x16],
        ];

        for bytes in cases {
            for split in 0..=bytes.len() {
                let mut decoder = StreamDecoder::new();
                let (mut frames, mut recoveries) = outputs();
                let first = decoder
                    .push_into(&bytes[..split], &mut frames, &mut recoveries)
                    .unwrap();
                let second = decoder
                    .push_into(
                        &bytes[split..],
                        &mut frames[first.frames_written..],
                        &mut recoveries[first.recoveries_written..],
                    )
                    .unwrap();
                assert_eq!(first.frames_written + second.frames_written, 1);
                assert_eq!(decoder.buffered_bytes(), 0);
            }
        }
    }

    #[test]
    fn push_into_stops_at_frame_capacity_and_retries_suffix() {
        let mut decoder = StreamDecoder::new();
        let bytes = [0xe5, 0xa2, 0xe5];
        let mut first_frames = [None];
        let mut no_recoveries = [];
        let first = decoder
            .push_into(&bytes, &mut first_frames, &mut no_recoveries)
            .unwrap();
        assert_eq!(
            first,
            PushIntoOutcome {
                consumed: 1,
                frames_written: 1,
                recoveries_written: 0
            }
        );
        assert_eq!(decoder.buffered_bytes(), 0);

        let mut remaining = [None, None];
        let second = decoder
            .push_into(&bytes[first.consumed..], &mut remaining, &mut no_recoveries)
            .unwrap();
        assert_eq!(second.consumed, 2);
        assert!(matches!(
            remaining,
            [Some(Frame::Nack(_)), Some(Frame::Ack(_))]
        ));
    }

    #[test]
    fn zero_frame_capacity_only_buffers_incomplete_input() {
        let mut decoder = StreamDecoder::new();
        let mut no_frames = [];
        let mut no_recoveries = [];
        let incomplete = decoder
            .push_into(&SHORT[..2], &mut no_frames, &mut no_recoveries)
            .unwrap();
        assert_eq!(incomplete.consumed, 2);
        let complete = decoder
            .push_into(&SHORT[2..], &mut no_frames, &mut no_recoveries)
            .unwrap();
        assert_eq!(complete.consumed, 0);
        assert_eq!(decoder.buffered_bytes(), 2);
    }

    #[test]
    fn recovery_capacity_applies_backpressure() {
        let mut decoder = StreamDecoder::with_recovery(Recovery::Resync);
        let mut frames = [None];
        let mut no_recoveries = [];
        let blocked = decoder
            .push_into(&[0xff, 0xe5], &mut frames, &mut no_recoveries)
            .unwrap();
        assert_eq!(blocked.consumed, 0);

        let mut recoveries = [None];
        let progress = decoder
            .push_into(&[0xff, 0xe5], &mut frames, &mut recoveries)
            .unwrap();
        assert_eq!(
            progress,
            PushIntoOutcome {
                consumed: 2,
                frames_written: 1,
                recoveries_written: 1
            }
        );
        assert_eq!(recoveries[0].as_ref().unwrap().discarded(), [0xff]);
    }

    #[test]
    fn resyncs_noise_corruption_and_incomplete_false_candidates() {
        let mut decoder = StreamDecoder::with_recovery(Recovery::Resync);
        let (mut frames, mut recoveries) = outputs();
        let bytes = [0xff, 0x68, 0xe5];
        let progress = decoder
            .push_into(&bytes, &mut frames, &mut recoveries)
            .unwrap();
        assert_eq!(progress.consumed, bytes.len());
        assert!(matches!(frames[0], Some(Frame::Ack(_))));
        assert_eq!(recoveries[0].as_ref().unwrap().discarded(), [0xff, 0x68]);

        let corrupted = [0x10, 0x40, 1, 0x42, 0x16, 0xe5];
        let progress = decoder
            .push_into(&corrupted, &mut frames, &mut recoveries)
            .unwrap();
        assert_eq!(progress.frames_written, 1);
        assert!(matches!(frames[0], Some(Frame::Ack(_))));
    }

    #[test]
    fn resyncs_false_long_header_and_keeps_surrounding_frames() {
        let mut decoder = StreamDecoder::with_recovery(Recovery::Resync);
        let (mut frames, mut recoveries) = outputs();
        let false_long = [0x68, 0xff, 0xff, 0, 0xe5];
        let progress = decoder
            .push_into(&false_long, &mut frames, &mut recoveries)
            .unwrap();
        assert_eq!(progress.frames_written, 1);
        assert_eq!(
            recoveries[0].as_ref().unwrap().discarded(),
            &false_long[..4]
        );

        let around_noise = [0xe5, 0, 0x10, 0x40, 1, 0x41, 0x16];
        let progress = decoder
            .push_into(&around_noise, &mut frames, &mut recoveries)
            .unwrap();
        assert_eq!(progress.frames_written, 2);
        assert!(matches!(frames[0], Some(Frame::Ack(_))));
        assert!(matches!(frames[1], Some(Frame::Short(_))));
    }

    #[test]
    fn incomplete_long_payload_is_not_scanned_for_embedded_frames() {
        let payload = [0xe5, 0xa2, 0x10, 0x40, 1, 0x41, 0x16];
        let frame = LongFrame::new(Control::new(0x53), Address::new(1), 0, &payload).unwrap();
        let mut encoded = [0; LongFrame::MAX_LEN];
        let encoded = frame.encode_into(&mut encoded).unwrap();
        let mut decoder = StreamDecoder::with_recovery(Recovery::Resync);
        let (mut frames, mut recoveries) = outputs();

        let split = encoded.len() - 1;
        let first = decoder
            .push_into(&encoded[..split], &mut frames, &mut recoveries)
            .unwrap();
        assert_eq!(first.frames_written, 0);
        assert_eq!(first.recoveries_written, 0);
        let second = decoder
            .push_into(&encoded[split..], &mut frames, &mut recoveries)
            .unwrap();
        assert!(matches!(frames[0], Some(Frame::Long(_))));
        assert_eq!(second.recoveries_written, 0);
    }

    #[test]
    fn recovery_splits_long_noise_into_bounded_events() {
        let mut decoder = StreamDecoder::with_recovery(Recovery::Resync);
        let noise = [0xff; MAX_RECOVERY_BYTES * 2 + 1];
        let mut frames = [];
        let mut recoveries = [None; 3];
        let progress = decoder
            .push_into(&noise, &mut frames, &mut recoveries)
            .unwrap();
        assert_eq!(progress.consumed, noise.len());
        assert_eq!(progress.recoveries_written, 3);
        assert_eq!(recoveries[0].as_ref().unwrap().discarded().len(), 32);
        assert_eq!(recoveries[1].as_ref().unwrap().discarded().len(), 32);
        assert_eq!(recoveries[2].as_ref().unwrap().discarded(), [0xff]);
    }

    #[test]
    fn buffers_maximum_long_frame() {
        let frame = LongFrame::new(Control::new(0x53), Address::new(1), 0, &[0; 252]).unwrap();
        let mut encoded = [0; LongFrame::MAX_LEN];
        frame.encode_into(&mut encoded).unwrap();
        let mut decoder = StreamDecoder::new();
        let (mut frames, mut recoveries) = outputs();
        let first = decoder
            .push_into(
                &encoded[..LongFrame::MAX_LEN - 1],
                &mut frames,
                &mut recoveries,
            )
            .unwrap();
        assert_eq!(first.consumed, LongFrame::MAX_LEN - 1);
        assert_eq!(decoder.buffered_bytes(), LongFrame::MAX_LEN - 1);
        let second = decoder
            .push_into(
                &encoded[LongFrame::MAX_LEN - 1..],
                &mut frames,
                &mut recoveries,
            )
            .unwrap();
        assert_eq!(second.frames_written, 1);
        assert!(matches!(frames[0], Some(Frame::Long(_))));
    }

    #[test]
    fn strict_errors_reset_buffered_state() {
        let mut decoder = StreamDecoder::new();
        let (mut frames, mut recoveries) = outputs();
        decoder
            .push_into(&SHORT[..2], &mut frames, &mut recoveries)
            .unwrap();
        let error = decoder
            .push_into(&[1, 0x42, 0x16], &mut frames, &mut recoveries)
            .unwrap_err();
        assert!(matches!(error.error, DecodeError::Short(_)));
        assert_eq!(decoder.buffered_bytes(), 0);
    }

    #[test]
    fn strict_error_reports_frames_and_input_consumed_before_it() {
        let mut decoder = StreamDecoder::new();
        let (mut frames, mut recoveries) = outputs();
        let error = decoder
            .push_into(&[AckFrame::BYTE, 0xff], &mut frames, &mut recoveries)
            .unwrap_err();

        assert!(matches!(
            error.error,
            DecodeError::UnknownStart { actual: 0xff }
        ));
        assert_eq!(
            error.outcome,
            PushIntoOutcome {
                consumed: 1,
                frames_written: 1,
                recoveries_written: 0,
            }
        );
        assert!(matches!(frames[0], Some(Frame::Ack(_))));
    }

    #[test]
    fn stream_state_sizes_remain_bounded() {
        // One maximum frame plus three machine words and alignment.
        assert!(core::mem::size_of::<StreamDecoder>() <= LongFrame::MAX_LEN + 32);
        // Discard storage plus the decode error, length byte, and alignment.
        assert!(core::mem::size_of::<RecoveryEvent>() <= MAX_RECOVERY_BYTES + 32);
    }

    #[test]
    fn rejects_malformed_variable_headers() {
        let mut decoder = StreamDecoder::new();
        let (mut frames, mut recoveries) = outputs();
        let error = decoder
            .push_into(&[0x68, 3, 4], &mut frames, &mut recoveries)
            .unwrap_err();
        assert!(matches!(error.error, DecodeError::Control(_)));
        let error = decoder
            .push_into(&[0x68, 2, 2], &mut frames, &mut recoveries)
            .unwrap_err();
        assert!(matches!(error.error, DecodeError::Long(_)));
        assert!(
            decoder
                .push_into(&[0x68, 4, 4, 0], &mut frames, &mut recoveries)
                .is_err()
        );
        assert_eq!(decoder.buffered_bytes(), 0);
    }

    #[test]
    fn reset_and_finish_are_allocation_free() {
        let mut decoder = StreamDecoder::default();
        let mut frames = [];
        let mut recoveries = [];
        assert_eq!(decoder.finish(), Ok(()));
        decoder
            .push_into(&SHORT[..2], &mut frames, &mut recoveries)
            .unwrap();
        assert_eq!(
            decoder.finish(),
            Err(IncompleteFrameError {
                received_bytes: 2,
                expected_length: Some(5)
            })
        );
        decoder
            .push_into(&[0x68], &mut frames, &mut recoveries)
            .unwrap();
        assert_eq!(
            decoder.finish(),
            Err(IncompleteFrameError {
                received_bytes: 1,
                expected_length: None
            })
        );
        decoder
            .push_into(&SHORT[..2], &mut frames, &mut recoveries)
            .unwrap();
        decoder.reset();
        assert_eq!(decoder.buffered_bytes(), 0);
    }

    #[test]
    fn recovery_backpressure_preserves_corrupted_frame() {
        let mut decoder = StreamDecoder::with_recovery(Recovery::Resync);
        let mut frames: [Option<Frame>; 1] = [None];
        let mut recoveries = [];
        let corrupted = [0x10, 0x40, 1, 0x42, 0x16];
        let outcome = decoder
            .push_into(&corrupted, &mut frames, &mut recoveries)
            .unwrap();
        assert_eq!(outcome.consumed, 0);
        assert_eq!(outcome.frames_written, 0);
    }

    #[test]
    fn resyncs_from_corruption_already_in_the_internal_buffer() {
        let mut decoder = StreamDecoder::with_recovery(Recovery::Resync);
        let bytes = [0x68, 4, 4, 0x68, 0x53, 0xfe, 0x50, 0xe5, 0, 0x16];
        let (mut frames, mut recoveries) = outputs();
        decoder
            .push_into(&bytes[..9], &mut frames, &mut recoveries)
            .unwrap();
        let outcome = decoder
            .push_into(&bytes[9..], &mut frames, &mut recoveries)
            .unwrap();
        assert!(outcome.recoveries_written > 0);
        assert!(
            frames[..outcome.frames_written]
                .iter()
                .any(|frame| matches!(frame, Some(Frame::Ack(_))))
        );
    }

    #[test]
    fn resync_skips_a_recognized_but_incomplete_candidate() {
        let mut decoder = StreamDecoder::with_recovery(Recovery::Resync);
        let (mut frames, mut recoveries) = outputs();
        let outcome = decoder
            .push_into(&[0xff, 0x10, 0xe5], &mut frames, &mut recoveries)
            .unwrap();
        assert!(matches!(frames[0], Some(Frame::Ack(_))));
        assert_eq!(outcome.recoveries_written, 1);
        assert_eq!(recoveries[0].as_ref().unwrap().discarded(), [0xff, 0x10]);
    }

    #[test]
    fn empty_input_has_no_frame_length() {
        assert_eq!(combined_frame_length(&[], &[], 0), Ok(None));
    }

    #[test]
    fn candidate_classification_covers_crossing_and_invalid_frames() {
        assert!(matches!(
            candidate_at(&[0xff, 0x10, 0x40], &[1, 0x41, 0x16], 1),
            Candidate::Incomplete
        ));
        assert!(matches!(
            candidate_at(&[], &[0x10, 0x40, 1, 0x42, 0x16], 0),
            Candidate::Invalid
        ));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn formats_incomplete_frame_errors() {
        assert_eq!(
            IncompleteFrameError {
                received_bytes: 2,
                expected_length: Some(5),
            }
            .to_string(),
            "incomplete frame: expected 5 bytes, received 2"
        );
        assert_eq!(
            IncompleteFrameError {
                received_bytes: 1,
                expected_length: None,
            }
            .to_string(),
            "incomplete frame: received 1 bytes before the stream ended"
        );

        let push_into = PushIntoError {
            error: DecodeError::UnknownStart { actual: 0xff },
            outcome: PushIntoOutcome::default(),
        };
        assert_eq!(push_into.to_string(), "unknown frame start byte 0xff");
        assert!(core::error::Error::source(&push_into).is_some());

        let push = PushError {
            error: DecodeError::UnknownStart { actual: 0xff },
            outcome: PushOutcome::default(),
        };
        assert_eq!(push.to_string(), "unknown frame start byte 0xff");
        assert!(core::error::Error::source(&push).is_some());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn allocating_push_collects_all_output() {
        let mut decoder = StreamDecoder::with_recovery(Recovery::Resync);
        let mut bytes = alloc::vec![0xff];
        bytes.extend(core::iter::repeat_n(0xe5, 20));
        let outcome = decoder.push(&bytes).unwrap();
        assert_eq!(outcome.frames.len(), 20);
        assert_eq!(outcome.recoveries.len(), 1);
        assert_eq!(outcome.recoveries[0].discarded(), [0xff]);

        let mut decoder = StreamDecoder::new();
        let error = decoder.push(&[AckFrame::BYTE, 0xff]).unwrap_err();
        assert_eq!(error.outcome.consumed, 1);
        assert_eq!(error.outcome.frames.len(), 1);
        assert!(matches!(error.outcome.frames[0], Frame::Ack(_)));
        assert!(matches!(
            error.error,
            DecodeError::UnknownStart { actual: 0xff }
        ));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn push_matches_push_into() {
        let bytes = [0xff, 0xe5, 0xa2, 0x10, 0x40, 1, 0x41, 0x16];
        let mut allocating = StreamDecoder::with_recovery(Recovery::Resync);
        let allocated = allocating.push(&bytes).unwrap();

        let mut fixed = StreamDecoder::with_recovery(Recovery::Resync);
        let (mut frames, mut recoveries) = outputs();
        let progress = fixed
            .push_into(&bytes, &mut frames, &mut recoveries)
            .unwrap();
        assert_eq!(allocated.consumed, progress.consumed);
        assert_eq!(
            allocated.frames,
            frames
                .into_iter()
                .take(progress.frames_written)
                .flatten()
                .collect::<Vec<_>>()
        );
        assert_eq!(
            allocated.recoveries,
            recoveries
                .into_iter()
                .take(progress.recoveries_written)
                .flatten()
                .collect::<Vec<_>>()
        );
    }
}
