//! Python bindings for the wired M-Bus frame codecs.

use meterbus_wired_datalink::{
    AckFrame, AckFrameError, Address, Control, ControlError, ControlFrame, ControlFrameError,
    Frame as CoreFrame, LongFrame, LongFrameError, NackFrame, NackFrameError, ShortFrame,
    ShortFrameError,
    decoder::{
        self,
        exact::DecodeError,
        stream::{
            IncompleteFrameError as CoreIncompleteFrameError, StreamDecoder as CoreStreamDecoder,
        },
    },
};
use pyo3::{
    exceptions::PyBaseException,
    prelude::*,
    pybacked::PyBackedBytes,
    types::{PyBytes, PyModule},
};

mod exceptions {
    #![allow(missing_docs)]

    use pyo3::{create_exception, exceptions::PyException};

    create_exception!(_native, DatalinkError, PyException);
    create_exception!(_native, EmptyInputError, DatalinkError);
    create_exception!(_native, IncompleteVariableHeaderError, DatalinkError);
    create_exception!(_native, UnknownStartByteError, DatalinkError);
    create_exception!(_native, InvalidAckLengthError, DatalinkError);
    create_exception!(_native, InvalidAckByteError, DatalinkError);
    create_exception!(_native, AckOutputTooSmallError, DatalinkError);
    create_exception!(_native, InvalidNackLengthError, DatalinkError);
    create_exception!(_native, InvalidNackByteError, DatalinkError);
    create_exception!(_native, NackOutputTooSmallError, DatalinkError);
    create_exception!(_native, InvalidShortFrameLengthError, DatalinkError);
    create_exception!(_native, InvalidShortFrameStartError, DatalinkError);
    create_exception!(_native, InvalidShortFrameStopError, DatalinkError);
    create_exception!(_native, InvalidShortFrameChecksumError, DatalinkError);
    create_exception!(_native, InvalidShortFrameControlError, DatalinkError);
    create_exception!(_native, ShortFrameOutputTooSmallError, DatalinkError);
    create_exception!(_native, InvalidControlFrameLengthError, DatalinkError);
    create_exception!(_native, InvalidControlFrameStartError, DatalinkError);
    create_exception!(_native, InvalidControlFrameDataLengthError, DatalinkError);
    create_exception!(_native, InvalidControlFrameStopError, DatalinkError);
    create_exception!(_native, InvalidControlFrameChecksumError, DatalinkError);
    create_exception!(_native, InvalidControlFrameControlError, DatalinkError);
    create_exception!(_native, ControlFrameOutputTooSmallError, DatalinkError);
    create_exception!(_native, IncompleteLongFrameHeaderError, DatalinkError);
    create_exception!(_native, InvalidLongFrameLengthError, DatalinkError);
    create_exception!(_native, InvalidLongFrameStartError, DatalinkError);
    create_exception!(_native, MismatchedLongFrameDataLengthsError, DatalinkError);
    create_exception!(_native, InvalidLongFrameDataLengthError, DatalinkError);
    create_exception!(_native, InvalidLongFrameStopError, DatalinkError);
    create_exception!(_native, InvalidLongFrameChecksumError, DatalinkError);
    create_exception!(_native, InvalidLongFrameControlError, DatalinkError);
    create_exception!(_native, InvalidLongFrameUserDataLengthError, DatalinkError);
    create_exception!(_native, LongFrameOutputTooSmallError, DatalinkError);
    create_exception!(_native, IncompleteFrameError, DatalinkError);
}

/// Error raised for invalid wired M-Bus data or decoder state.
pub use exceptions::*;

/// A validated wired M-Bus frame.
#[pyclass(
    frozen,
    module = "meterbus_wired_datalink._native",
    skip_from_py_object
)]
#[derive(Clone)]
pub struct Frame {
    inner: CoreFrame,
}

#[pymethods]
impl Frame {
    /// Creates an ACK frame.
    #[staticmethod]
    pub fn ack() -> Self {
        Self::from(CoreFrame::from(AckFrame::new()))
    }

    /// Creates a NACK frame.
    #[staticmethod]
    pub fn nack() -> Self {
        Self::from(CoreFrame::from(NackFrame::new()))
    }

    /// Creates a validated short frame.
    #[staticmethod]
    pub fn short(py: Python<'_>, control: u8, address: u8) -> PyResult<Self> {
        ShortFrame::new(Control::new(control), Address::new(address))
            .map(CoreFrame::from)
            .map(Self::from)
            .map_err(|error| py_short_error(py, error))
    }

    /// Creates a validated variable-format frame without user data.
    #[staticmethod]
    pub fn control(
        py: Python<'_>,
        control: u8,
        address: u8,
        control_information: u8,
    ) -> PyResult<Self> {
        ControlFrame::new(
            Control::new(control),
            Address::new(address),
            control_information,
        )
        .map(CoreFrame::from)
        .map(Self::from)
        .map_err(|error| py_control_frame_error(py, error))
    }

    /// Creates a validated variable-format frame containing user data.
    #[staticmethod]
    pub fn long(
        py: Python<'_>,
        control: u8,
        address: u8,
        control_information: u8,
        user_data: PyBackedBytes,
    ) -> PyResult<Self> {
        LongFrame::new(
            Control::new(control),
            Address::new(address),
            control_information,
            &user_data,
        )
        .map(CoreFrame::from)
        .map(Self::from)
        .map_err(|error| py_long_error(py, error))
    }

    /// Decodes exactly one wired M-Bus frame.
    #[staticmethod]
    pub fn decode(py: Python<'_>, bytes: PyBackedBytes) -> PyResult<Self> {
        decoder::exact::decode(&bytes)
            .map(Self::from)
            .map_err(|error| py_decode_error(py, error))
    }

    /// Returns the frame kind.
    #[getter]
    pub fn kind(&self) -> &'static str {
        match self.inner {
            CoreFrame::Ack(_) => "ack",
            CoreFrame::Nack(_) => "nack",
            CoreFrame::Short(_) => "short",
            CoreFrame::Control(_) => "control",
            CoreFrame::Long(_) => "long",
        }
    }

    /// Returns the control byte when present.
    #[getter]
    pub fn control_byte(&self) -> Option<u8> {
        match &self.inner {
            CoreFrame::Short(frame) => Some(frame.control().value()),
            CoreFrame::Control(frame) => Some(frame.control().value()),
            CoreFrame::Long(frame) => Some(frame.control().value()),
            CoreFrame::Ack(_) | CoreFrame::Nack(_) => None,
        }
    }

    /// Returns the address byte when present.
    #[getter]
    pub fn address(&self) -> Option<u8> {
        match &self.inner {
            CoreFrame::Short(frame) => Some(frame.address().value()),
            CoreFrame::Control(frame) => Some(frame.address().value()),
            CoreFrame::Long(frame) => Some(frame.address().value()),
            CoreFrame::Ack(_) | CoreFrame::Nack(_) => None,
        }
    }

    /// Returns the control-information byte when present.
    #[getter]
    pub fn control_information(&self) -> Option<u8> {
        match &self.inner {
            CoreFrame::Control(frame) => Some(frame.control_information()),
            CoreFrame::Long(frame) => Some(frame.control_information()),
            CoreFrame::Ack(_) | CoreFrame::Nack(_) | CoreFrame::Short(_) => None,
        }
    }

    /// Returns a copy of the user-data bytes when present.
    #[getter]
    pub fn user_data<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        match &self.inner {
            CoreFrame::Long(frame) => Some(PyBytes::new(py, frame.user_data())),
            CoreFrame::Ack(_)
            | CoreFrame::Nack(_)
            | CoreFrame::Short(_)
            | CoreFrame::Control(_) => None,
        }
    }

    /// Encodes the frame.
    pub fn encode<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.encode())
    }

    fn __repr__(&self) -> String {
        format!("Frame(kind='{}')", self.kind())
    }
}

impl From<CoreFrame> for Frame {
    fn from(inner: CoreFrame) -> Self {
        Self { inner }
    }
}

/// A recovery performed while resynchronizing a byte stream.
#[pyclass(frozen, module = "meterbus_wired_datalink._native")]
pub struct StreamRecovery {
    error: DecodeError,
    discarded: Vec<u8>,
}

#[pymethods]
impl StreamRecovery {
    /// Returns the decoding error that caused recovery.
    #[getter]
    pub fn error(&self, py: Python<'_>) -> Py<PyBaseException> {
        py_decode_error(py, self.error).into_value(py)
    }

    /// Returns the bytes discarded while finding the next frame.
    #[getter]
    pub fn discarded<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.discarded)
    }
}

/// Output produced by a streaming decoder push.
#[pyclass(frozen, module = "meterbus_wired_datalink._native")]
pub struct StreamPushResult {
    frames: Vec<Py<Frame>>,
    recoveries: Vec<Py<StreamRecovery>>,
}

#[pymethods]
impl StreamPushResult {
    /// Returns the complete frames decoded from the input.
    #[getter]
    pub fn frames(&self, py: Python<'_>) -> Vec<Py<Frame>> {
        self.frames
            .iter()
            .map(|frame| frame.clone_ref(py))
            .collect()
    }

    /// Returns the recoveries performed while decoding.
    #[getter]
    pub fn recoveries(&self, py: Python<'_>) -> Vec<Py<StreamRecovery>> {
        self.recoveries
            .iter()
            .map(|recovery| recovery.clone_ref(py))
            .collect()
    }
}

/// A stateful incremental frame decoder.
#[pyclass(module = "meterbus_wired_datalink._native")]
pub struct StreamDecoder {
    inner: CoreStreamDecoder,
}

#[pymethods]
impl StreamDecoder {
    /// Creates a strict stream decoder.
    #[new]
    pub fn new() -> Self {
        Self {
            inner: CoreStreamDecoder::new(),
        }
    }

    /// Creates a decoder that discards malformed bytes and resynchronizes.
    #[staticmethod]
    pub fn resync() -> Self {
        Self {
            inner: CoreStreamDecoder::with_recovery(decoder::stream::Recovery::Resync),
        }
    }

    /// Returns the number of incomplete bytes retained between pushes.
    #[getter]
    pub fn buffered_bytes(&self) -> usize {
        self.inner.buffered_bytes()
    }

    /// Adds a byte chunk and returns decoded frames and recovery events.
    pub fn push(&mut self, py: Python<'_>, chunk: PyBackedBytes) -> PyResult<StreamPushResult> {
        let outcome = self
            .inner
            .push(&chunk)
            .map_err(|error| py_decode_error(py, error.error))?;
        let frames = outcome
            .frames
            .into_iter()
            .map(|frame| Py::new(py, Frame::from(frame)))
            .collect::<PyResult<_>>()?;
        let recoveries = outcome
            .recoveries
            .into_iter()
            .map(|recovery| {
                Py::new(
                    py,
                    StreamRecovery {
                        error: recovery.error,
                        discarded: recovery.discarded().to_vec(),
                    },
                )
            })
            .collect::<PyResult<_>>()?;
        Ok(StreamPushResult { frames, recoveries })
    }

    /// Completes the stream and rejects incomplete trailing input.
    pub fn finish(&mut self, py: Python<'_>) -> PyResult<()> {
        self.inner
            .finish()
            .map_err(|error| py_incomplete_frame_error(py, error))
    }

    /// Discards incomplete buffered input.
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

impl Default for StreamDecoder {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! py_error {
    ($py:expr, $type:ty, $error:expr $(, $name:literal => $value:expr)*) => {{
        let exception = <$type>::new_err($error.to_string());
        $(
            if let Err(attribute_error) = exception.value($py).setattr($name, $value) {
                return attribute_error;
            }
        )*
        exception
    }};
}

fn py_decode_error(py: Python<'_>, error: DecodeError) -> PyErr {
    match error {
        DecodeError::Empty => py_error!(py, EmptyInputError, error),
        DecodeError::IncompleteVariableHeader => {
            py_error!(py, IncompleteVariableHeaderError, error)
        }
        DecodeError::UnknownStart { actual } => {
            py_error!(py, UnknownStartByteError, error, "actual" => actual)
        }
        DecodeError::Ack(error) => py_ack_error(py, error),
        DecodeError::Nack(error) => py_nack_error(py, error),
        DecodeError::Short(error) => py_short_error(py, error),
        DecodeError::Control(error) => py_control_frame_error(py, error),
        DecodeError::Long(error) => py_long_error(py, error),
    }
}

fn py_ack_error(py: Python<'_>, error: AckFrameError) -> PyErr {
    match error {
        AckFrameError::InvalidLength { actual } => {
            py_error!(py, InvalidAckLengthError, error, "actual" => actual)
        }
        AckFrameError::InvalidByte { actual } => {
            py_error!(py, InvalidAckByteError, error, "actual" => actual)
        }
        AckFrameError::OutputTooSmall { actual } => {
            py_error!(py, AckOutputTooSmallError, error, "actual" => actual)
        }
    }
}

fn py_nack_error(py: Python<'_>, error: NackFrameError) -> PyErr {
    match error {
        NackFrameError::InvalidLength { actual } => {
            py_error!(py, InvalidNackLengthError, error, "actual" => actual)
        }
        NackFrameError::InvalidByte { actual } => {
            py_error!(py, InvalidNackByteError, error, "actual" => actual)
        }
        NackFrameError::OutputTooSmall { actual } => {
            py_error!(py, NackOutputTooSmallError, error, "actual" => actual)
        }
    }
}

fn py_short_error(py: Python<'_>, error: ShortFrameError) -> PyErr {
    match error {
        ShortFrameError::InvalidLength { actual } => {
            py_error!(py, InvalidShortFrameLengthError, error, "actual" => actual)
        }
        ShortFrameError::InvalidStart { actual } => {
            py_error!(py, InvalidShortFrameStartError, error, "actual" => actual)
        }
        ShortFrameError::InvalidStop { actual } => {
            py_error!(py, InvalidShortFrameStopError, error, "actual" => actual)
        }
        ShortFrameError::InvalidChecksum { expected, actual } => py_error!(
            py,
            InvalidShortFrameChecksumError,
            error,
            "expected" => expected,
            "actual" => actual
        ),
        ShortFrameError::Control(ControlError::InvalidForShortFrame { value }) => {
            py_error!(py, InvalidShortFrameControlError, error, "value" => value)
        }
        ShortFrameError::Control(ControlError::InvalidForVariableFrame { value }) => {
            py_error!(py, InvalidShortFrameControlError, error, "value" => value)
        }
        ShortFrameError::OutputTooSmall { actual } => {
            py_error!(py, ShortFrameOutputTooSmallError, error, "actual" => actual)
        }
    }
}

fn py_control_frame_error(py: Python<'_>, error: ControlFrameError) -> PyErr {
    match error {
        ControlFrameError::InvalidLength { actual } => {
            py_error!(py, InvalidControlFrameLengthError, error, "actual" => actual)
        }
        ControlFrameError::InvalidStart { index, actual } => py_error!(
            py,
            InvalidControlFrameStartError,
            error,
            "index" => index,
            "actual" => actual
        ),
        ControlFrameError::InvalidDataLength { index, actual } => py_error!(
            py,
            InvalidControlFrameDataLengthError,
            error,
            "index" => index,
            "actual" => actual
        ),
        ControlFrameError::InvalidStop { actual } => {
            py_error!(py, InvalidControlFrameStopError, error, "actual" => actual)
        }
        ControlFrameError::InvalidChecksum { expected, actual } => py_error!(
            py,
            InvalidControlFrameChecksumError,
            error,
            "expected" => expected,
            "actual" => actual
        ),
        ControlFrameError::Control(ControlError::InvalidForVariableFrame { value })
        | ControlFrameError::Control(ControlError::InvalidForShortFrame { value }) => py_error!(
            py,
            InvalidControlFrameControlError,
            error,
            "value" => value
        ),
        ControlFrameError::OutputTooSmall { actual } => py_error!(
            py,
            ControlFrameOutputTooSmallError,
            error,
            "actual" => actual
        ),
    }
}

fn py_long_error(py: Python<'_>, error: LongFrameError) -> PyErr {
    match error {
        LongFrameError::IncompleteHeader { actual } => {
            py_error!(py, IncompleteLongFrameHeaderError, error, "actual" => actual)
        }
        LongFrameError::InvalidLength { expected, actual } => py_error!(
            py,
            InvalidLongFrameLengthError,
            error,
            "expected" => expected,
            "actual" => actual
        ),
        LongFrameError::InvalidStart { index, actual } => py_error!(
            py,
            InvalidLongFrameStartError,
            error,
            "index" => index,
            "actual" => actual
        ),
        LongFrameError::MismatchedDataLengths { first, second } => py_error!(
            py,
            MismatchedLongFrameDataLengthsError,
            error,
            "first" => first,
            "second" => second
        ),
        LongFrameError::InvalidDataLength { actual } => {
            py_error!(py, InvalidLongFrameDataLengthError, error, "actual" => actual)
        }
        LongFrameError::InvalidStop { actual } => {
            py_error!(py, InvalidLongFrameStopError, error, "actual" => actual)
        }
        LongFrameError::InvalidChecksum { expected, actual } => py_error!(
            py,
            InvalidLongFrameChecksumError,
            error,
            "expected" => expected,
            "actual" => actual
        ),
        LongFrameError::Control(ControlError::InvalidForVariableFrame { value })
        | LongFrameError::Control(ControlError::InvalidForShortFrame { value }) => {
            py_error!(py, InvalidLongFrameControlError, error, "value" => value)
        }
        LongFrameError::InvalidUserDataLength { actual } => py_error!(
            py,
            InvalidLongFrameUserDataLengthError,
            error,
            "actual" => actual
        ),
        LongFrameError::OutputTooSmall { required, actual } => py_error!(
            py,
            LongFrameOutputTooSmallError,
            error,
            "required" => required,
            "actual" => actual
        ),
    }
}

fn py_incomplete_frame_error(py: Python<'_>, error: CoreIncompleteFrameError) -> PyErr {
    py_error!(
        py,
        IncompleteFrameError,
        error,
        "received_bytes" => error.received_bytes,
        "expected_length" => error.expected_length
    )
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    macro_rules! add_exception {
        ($type:ty) => {
            module.add(stringify!($type), module.py().get_type::<$type>())?;
        };
    }

    add_exception!(DatalinkError);
    add_exception!(EmptyInputError);
    add_exception!(IncompleteVariableHeaderError);
    add_exception!(UnknownStartByteError);
    add_exception!(InvalidAckLengthError);
    add_exception!(InvalidAckByteError);
    add_exception!(AckOutputTooSmallError);
    add_exception!(InvalidNackLengthError);
    add_exception!(InvalidNackByteError);
    add_exception!(NackOutputTooSmallError);
    add_exception!(InvalidShortFrameLengthError);
    add_exception!(InvalidShortFrameStartError);
    add_exception!(InvalidShortFrameStopError);
    add_exception!(InvalidShortFrameChecksumError);
    add_exception!(InvalidShortFrameControlError);
    add_exception!(ShortFrameOutputTooSmallError);
    add_exception!(InvalidControlFrameLengthError);
    add_exception!(InvalidControlFrameStartError);
    add_exception!(InvalidControlFrameDataLengthError);
    add_exception!(InvalidControlFrameStopError);
    add_exception!(InvalidControlFrameChecksumError);
    add_exception!(InvalidControlFrameControlError);
    add_exception!(ControlFrameOutputTooSmallError);
    add_exception!(IncompleteLongFrameHeaderError);
    add_exception!(InvalidLongFrameLengthError);
    add_exception!(InvalidLongFrameStartError);
    add_exception!(MismatchedLongFrameDataLengthsError);
    add_exception!(InvalidLongFrameDataLengthError);
    add_exception!(InvalidLongFrameStopError);
    add_exception!(InvalidLongFrameChecksumError);
    add_exception!(InvalidLongFrameControlError);
    add_exception!(InvalidLongFrameUserDataLengthError);
    add_exception!(LongFrameOutputTooSmallError);
    add_exception!(IncompleteFrameError);
    module.add_class::<Frame>()?;
    module.add_class::<StreamDecoder>()?;
    module.add_class::<StreamPushResult>()?;
    module.add_class::<StreamRecovery>()?;
    Ok(())
}
