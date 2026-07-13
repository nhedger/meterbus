//! Frame decoders.
//!
//! Use [`exact`] when a slice contains exactly one frame. Use [`stream`] for
//! arbitrary chunks, split frames, or several frames in one chunk. Both return
//! the crate-level [`Frame`](crate::Frame) type.

pub mod exact;
pub mod stream;
