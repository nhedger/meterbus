//! Node-API bindings for the wired M-Bus frame codecs.

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
use napi::{
    Env, Error, Result,
    bindgen_prelude::{JsObjectValue, ToNapiValue, Uint8Array, Unknown},
};
use napi_derive::napi;

/// A validated wired M-Bus frame.
#[derive(Clone)]
#[napi]
pub struct Frame {
    inner: CoreFrame,
}

/// A recovery performed while resynchronizing a byte stream.
#[derive(Clone)]
#[napi]
pub struct StreamRecovery {
    error: DecodeError,
    discarded: Vec<u8>,
}

#[napi]
impl StreamRecovery {
    /// Returns the decoding error that caused recovery.
    #[napi(getter)]
    pub fn error<'env>(&self, env: &'env Env) -> Result<Unknown<'env>> {
        create_error(env, decode_error_info(self.error))
    }

    /// Returns the bytes discarded while finding the next frame.
    #[napi(getter)]
    pub fn discarded(&self) -> Uint8Array {
        self.discarded.clone().into()
    }
}

/// Output produced by a streaming decoder push.
#[napi]
pub struct StreamPushResult {
    frames: Vec<Frame>,
    recoveries: Vec<StreamRecovery>,
}

#[napi]
impl StreamPushResult {
    /// Returns the complete frames decoded from the input.
    #[napi(getter)]
    pub fn frames(&self) -> Vec<Frame> {
        self.frames.clone()
    }

    /// Returns the recoveries performed while decoding.
    #[napi(getter)]
    pub fn recoveries(&self) -> Vec<StreamRecovery> {
        self.recoveries.clone()
    }
}

/// A stateful incremental frame decoder.
#[napi]
pub struct StreamDecoder {
    inner: CoreStreamDecoder,
}

#[napi]
impl StreamDecoder {
    /// Creates a strict stream decoder.
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: CoreStreamDecoder::new(),
        }
    }

    /// Creates a decoder that discards malformed bytes and resynchronizes.
    #[napi(factory)]
    pub fn resync() -> Self {
        Self {
            inner: CoreStreamDecoder::with_recovery(decoder::stream::Recovery::Resync),
        }
    }

    /// Returns the number of incomplete bytes retained between pushes.
    #[napi(getter)]
    pub fn buffered_bytes(&self) -> u32 {
        self.inner.buffered_bytes() as u32
    }

    /// Adds a byte chunk and returns decoded frames and recovery events.
    #[napi]
    pub fn push(&mut self, env: &Env, chunk: Uint8Array) -> Result<StreamPushResult> {
        let outcome = self
            .inner
            .push(&chunk)
            .map_err(|error| napi_error(env, decode_error_info(error.error)))?;
        Ok(StreamPushResult {
            frames: outcome.frames.into_iter().map(Frame::from).collect(),
            recoveries: outcome
                .recoveries
                .into_iter()
                .map(|recovery| StreamRecovery {
                    error: recovery.error,
                    discarded: recovery.discarded().to_vec(),
                })
                .collect(),
        })
    }

    /// Completes the stream and rejects incomplete trailing input.
    #[napi]
    pub fn finish(&mut self, env: &Env) -> Result<()> {
        self.inner
            .finish()
            .map_err(|error| napi_error(env, incomplete_frame_error_info(error)))
    }

    /// Discards incomplete buffered input.
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

impl Default for StreamDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl Frame {
    /// Creates an ACK frame.
    #[napi(factory)]
    pub fn ack() -> Self {
        Self::from(CoreFrame::from(AckFrame::new()))
    }

    /// Creates a NACK frame.
    #[napi(factory)]
    pub fn nack() -> Self {
        Self::from(CoreFrame::from(NackFrame::new()))
    }

    /// Creates a validated short frame.
    #[napi(factory)]
    pub fn short(env: &Env, control: f64, address: f64) -> Result<Self> {
        let frame = ShortFrame::new(
            Control::new(byte("control", control)?),
            Address::new(byte("address", address)?),
        )
        .map_err(|error| napi_error(env, short_error_info(error)))?;
        Ok(Self::from(CoreFrame::from(frame)))
    }

    /// Creates a validated variable-format frame without user data.
    #[napi(factory)]
    pub fn control(
        env: &Env,
        control: f64,
        address: f64,
        control_information: f64,
    ) -> Result<Self> {
        let frame = ControlFrame::new(
            Control::new(byte("control", control)?),
            Address::new(byte("address", address)?),
            byte("controlInformation", control_information)?,
        )
        .map_err(|error| napi_error(env, control_frame_error_info(error)))?;
        Ok(Self::from(CoreFrame::from(frame)))
    }

    /// Creates a validated variable-format frame containing user data.
    #[napi(factory)]
    pub fn long(
        env: &Env,
        control: f64,
        address: f64,
        control_information: f64,
        user_data: Uint8Array,
    ) -> Result<Self> {
        let frame = LongFrame::new(
            Control::new(byte("control", control)?),
            Address::new(byte("address", address)?),
            byte("controlInformation", control_information)?,
            &user_data,
        )
        .map_err(|error| napi_error(env, long_error_info(error)))?;
        Ok(Self::from(CoreFrame::from(frame)))
    }

    /// Decodes exactly one wired M-Bus frame.
    #[napi(factory)]
    pub fn decode(env: &Env, bytes: Uint8Array) -> Result<Self> {
        decoder::exact::decode(&bytes)
            .map(Self::from)
            .map_err(|error| napi_error(env, decode_error_info(error)))
    }

    /// Returns the frame kind.
    #[napi(getter)]
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
    #[napi(getter, js_name = "controlByte")]
    pub fn control_byte(&self) -> Option<u8> {
        match &self.inner {
            CoreFrame::Short(frame) => Some(frame.control().value()),
            CoreFrame::Control(frame) => Some(frame.control().value()),
            CoreFrame::Long(frame) => Some(frame.control().value()),
            CoreFrame::Ack(_) | CoreFrame::Nack(_) => None,
        }
    }

    /// Returns the address byte when present.
    #[napi(getter)]
    pub fn address(&self) -> Option<u8> {
        match &self.inner {
            CoreFrame::Short(frame) => Some(frame.address().value()),
            CoreFrame::Control(frame) => Some(frame.address().value()),
            CoreFrame::Long(frame) => Some(frame.address().value()),
            CoreFrame::Ack(_) | CoreFrame::Nack(_) => None,
        }
    }

    /// Returns the control-information byte when present.
    #[napi(getter, js_name = "controlInformation")]
    pub fn control_information(&self) -> Option<u8> {
        match &self.inner {
            CoreFrame::Control(frame) => Some(frame.control_information()),
            CoreFrame::Long(frame) => Some(frame.control_information()),
            CoreFrame::Ack(_) | CoreFrame::Nack(_) | CoreFrame::Short(_) => None,
        }
    }

    /// Returns a copy of the user-data bytes when present.
    #[napi(getter, js_name = "userData")]
    pub fn user_data(&self) -> Option<Uint8Array> {
        match &self.inner {
            CoreFrame::Long(frame) => Some(frame.user_data().to_vec().into()),
            CoreFrame::Ack(_)
            | CoreFrame::Nack(_)
            | CoreFrame::Short(_)
            | CoreFrame::Control(_) => None,
        }
    }

    /// Encodes the frame.
    #[napi]
    pub fn encode(&self) -> Uint8Array {
        self.inner.encode().into()
    }
}

impl From<CoreFrame> for Frame {
    fn from(inner: CoreFrame) -> Self {
        Self { inner }
    }
}

fn byte(name: &str, value: f64) -> Result<u8> {
    if value.is_finite() && value.fract() == 0.0 && (0.0..=255.0).contains(&value) {
        Ok(value as u8)
    } else {
        Err(Error::from_reason(format!(
            "{name} must be an integer between 0 and 255"
        )))
    }
}

struct DatalinkErrorInfo {
    name: &'static str,
    message: String,
    fields: Vec<(&'static str, Option<u32>)>,
}

macro_rules! error_info {
    ($name:literal, $error:expr $(, $field:literal => $value:expr)*) => {
        DatalinkErrorInfo {
            name: $name,
            message: $error.to_string(),
            fields: vec![$(($field, Some(u32::from($value)))),*],
        }
    };
}

macro_rules! usize_error_info {
    ($name:literal, $error:expr $(, $field:literal => $value:expr)*) => {
        DatalinkErrorInfo {
            name: $name,
            message: $error.to_string(),
            fields: vec![$(($field, Some(to_u32($value)))),*],
        }
    };
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn create_error<'env>(env: &'env Env, info: DatalinkErrorInfo) -> Result<Unknown<'env>> {
    let mut object = env.create_error(Error::from_reason(info.message))?;
    object.set_named_property("name", info.name)?;
    for (name, value) in info.fields {
        object.set_named_property(name, value)?;
    }
    object.into_unknown(env)
}

fn napi_error(env: &Env, info: DatalinkErrorInfo) -> Error {
    match create_error(env, info) {
        Ok(error) => Error::from(error),
        Err(error) => error,
    }
}

fn decode_error_info(error: DecodeError) -> DatalinkErrorInfo {
    match error {
        DecodeError::Empty => error_info!("EmptyInputError", error),
        DecodeError::IncompleteVariableHeader => {
            error_info!("IncompleteVariableHeaderError", error)
        }
        DecodeError::UnknownStart { actual } => {
            error_info!("UnknownStartByteError", error, "actual" => actual)
        }
        DecodeError::Ack(error) => ack_error_info(error),
        DecodeError::Nack(error) => nack_error_info(error),
        DecodeError::Short(error) => short_error_info(error),
        DecodeError::Control(error) => control_frame_error_info(error),
        DecodeError::Long(error) => long_error_info(error),
    }
}

fn ack_error_info(error: AckFrameError) -> DatalinkErrorInfo {
    match error {
        AckFrameError::InvalidLength { actual } => {
            usize_error_info!("InvalidAckLengthError", error, "actual" => actual)
        }
        AckFrameError::InvalidByte { actual } => {
            error_info!("InvalidAckByteError", error, "actual" => actual)
        }
        AckFrameError::OutputTooSmall { actual } => {
            usize_error_info!("AckOutputTooSmallError", error, "actual" => actual)
        }
    }
}

fn nack_error_info(error: NackFrameError) -> DatalinkErrorInfo {
    match error {
        NackFrameError::InvalidLength { actual } => {
            usize_error_info!("InvalidNackLengthError", error, "actual" => actual)
        }
        NackFrameError::InvalidByte { actual } => {
            error_info!("InvalidNackByteError", error, "actual" => actual)
        }
        NackFrameError::OutputTooSmall { actual } => {
            usize_error_info!("NackOutputTooSmallError", error, "actual" => actual)
        }
    }
}

fn short_error_info(error: ShortFrameError) -> DatalinkErrorInfo {
    match error {
        ShortFrameError::InvalidLength { actual } => {
            usize_error_info!("InvalidShortFrameLengthError", error, "actual" => actual)
        }
        ShortFrameError::InvalidStart { actual } => {
            error_info!("InvalidShortFrameStartError", error, "actual" => actual)
        }
        ShortFrameError::InvalidStop { actual } => {
            error_info!("InvalidShortFrameStopError", error, "actual" => actual)
        }
        ShortFrameError::InvalidChecksum { expected, actual } => error_info!(
            "InvalidShortFrameChecksumError",
            error,
            "expected" => expected,
            "actual" => actual
        ),
        ShortFrameError::Control(ControlError::InvalidForShortFrame { value })
        | ShortFrameError::Control(ControlError::InvalidForVariableFrame { value }) => {
            error_info!("InvalidShortFrameControlError", error, "value" => value)
        }
        ShortFrameError::OutputTooSmall { actual } => {
            usize_error_info!("ShortFrameOutputTooSmallError", error, "actual" => actual)
        }
    }
}

fn control_frame_error_info(error: ControlFrameError) -> DatalinkErrorInfo {
    match error {
        ControlFrameError::InvalidLength { actual } => {
            usize_error_info!("InvalidControlFrameLengthError", error, "actual" => actual)
        }
        ControlFrameError::InvalidStart { index, actual } => DatalinkErrorInfo {
            name: "InvalidControlFrameStartError",
            message: error.to_string(),
            fields: vec![
                ("index", Some(to_u32(index))),
                ("actual", Some(u32::from(actual))),
            ],
        },
        ControlFrameError::InvalidDataLength { index, actual } => DatalinkErrorInfo {
            name: "InvalidControlFrameDataLengthError",
            message: error.to_string(),
            fields: vec![
                ("index", Some(to_u32(index))),
                ("actual", Some(u32::from(actual))),
            ],
        },
        ControlFrameError::InvalidStop { actual } => {
            error_info!("InvalidControlFrameStopError", error, "actual" => actual)
        }
        ControlFrameError::InvalidChecksum { expected, actual } => error_info!(
            "InvalidControlFrameChecksumError",
            error,
            "expected" => expected,
            "actual" => actual
        ),
        ControlFrameError::Control(ControlError::InvalidForVariableFrame { value })
        | ControlFrameError::Control(ControlError::InvalidForShortFrame { value }) => {
            error_info!("InvalidControlFrameControlError", error, "value" => value)
        }
        ControlFrameError::OutputTooSmall { actual } => usize_error_info!(
            "ControlFrameOutputTooSmallError",
            error,
            "actual" => actual
        ),
    }
}

fn long_error_info(error: LongFrameError) -> DatalinkErrorInfo {
    match error {
        LongFrameError::IncompleteHeader { actual } => usize_error_info!(
            "IncompleteLongFrameHeaderError",
            error,
            "actual" => actual
        ),
        LongFrameError::InvalidLength { expected, actual } => DatalinkErrorInfo {
            name: "InvalidLongFrameLengthError",
            message: error.to_string(),
            fields: vec![
                ("expected", Some(to_u32(expected))),
                ("actual", Some(to_u32(actual))),
            ],
        },
        LongFrameError::InvalidStart { index, actual } => DatalinkErrorInfo {
            name: "InvalidLongFrameStartError",
            message: error.to_string(),
            fields: vec![
                ("index", Some(to_u32(index))),
                ("actual", Some(u32::from(actual))),
            ],
        },
        LongFrameError::MismatchedDataLengths { first, second } => error_info!(
            "MismatchedLongFrameDataLengthsError",
            error,
            "first" => first,
            "second" => second
        ),
        LongFrameError::InvalidDataLength { actual } => {
            error_info!("InvalidLongFrameDataLengthError", error, "actual" => actual)
        }
        LongFrameError::InvalidStop { actual } => {
            error_info!("InvalidLongFrameStopError", error, "actual" => actual)
        }
        LongFrameError::InvalidChecksum { expected, actual } => error_info!(
            "InvalidLongFrameChecksumError",
            error,
            "expected" => expected,
            "actual" => actual
        ),
        LongFrameError::Control(ControlError::InvalidForVariableFrame { value })
        | LongFrameError::Control(ControlError::InvalidForShortFrame { value }) => {
            error_info!("InvalidLongFrameControlError", error, "value" => value)
        }
        LongFrameError::InvalidUserDataLength { actual } => usize_error_info!(
            "InvalidLongFrameUserDataLengthError",
            error,
            "actual" => actual
        ),
        LongFrameError::OutputTooSmall { required, actual } => DatalinkErrorInfo {
            name: "LongFrameOutputTooSmallError",
            message: error.to_string(),
            fields: vec![
                ("required", Some(to_u32(required))),
                ("actual", Some(to_u32(actual))),
            ],
        },
    }
}

fn incomplete_frame_error_info(error: CoreIncompleteFrameError) -> DatalinkErrorInfo {
    DatalinkErrorInfo {
        name: "IncompleteFrameError",
        message: error.to_string(),
        fields: vec![
            ("receivedBytes", Some(to_u32(error.received_bytes))),
            ("expectedLength", error.expected_length.map(to_u32)),
        ],
    }
}
