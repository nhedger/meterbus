//! WebAssembly bindings for the wired M-Bus frame codecs.

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
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const STREAM_TYPES: &str = r#"
export interface StreamRecovery {
  readonly error: Error;
  readonly discarded: Uint8Array;
}

export interface StreamPushResult {
  readonly frames: Frame[];
  readonly recoveries: StreamRecovery[];
}
"#;

#[wasm_bindgen]
extern "C" {
    /// Output produced by a streaming decoder push.
    #[wasm_bindgen(typescript_type = "StreamPushResult")]
    pub type StreamPushResult;
}

/// A validated wired M-Bus frame.
#[wasm_bindgen]
pub struct Frame {
    inner: CoreFrame,
}

/// A stateful incremental frame decoder.
#[wasm_bindgen]
pub struct StreamDecoder {
    inner: CoreStreamDecoder,
}

#[wasm_bindgen]
impl StreamDecoder {
    /// Creates a strict stream decoder.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: CoreStreamDecoder::new(),
        }
    }

    /// Creates a decoder that discards malformed bytes and resynchronizes.
    pub fn resync() -> Self {
        Self {
            inner: CoreStreamDecoder::with_recovery(decoder::stream::Recovery::Resync),
        }
    }

    /// Returns the number of incomplete bytes retained between pushes.
    #[wasm_bindgen(getter, js_name = bufferedBytes)]
    pub fn buffered_bytes(&self) -> u32 {
        self.inner.buffered_bytes() as u32
    }

    /// Adds a byte chunk and returns decoded frames and recovery events.
    pub fn push(&mut self, chunk: &[u8]) -> Result<StreamPushResult, JsValue> {
        let outcome = self
            .inner
            .push(chunk)
            .map_err(|error| js_error(decode_error_info(error.error)))?;
        let result = js_sys::Object::new();
        let frames = js_sys::Array::new();
        let recoveries = js_sys::Array::new();

        for frame in outcome.frames {
            frames.push(&JsValue::from(Frame::from(frame)));
        }
        for recovery in outcome.recoveries {
            let item = js_sys::Object::new();
            set(&item, "error", &js_error(decode_error_info(recovery.error)));
            set(
                &item,
                "discarded",
                &js_sys::Uint8Array::from(recovery.discarded()),
            );
            recoveries.push(&item);
        }

        set(&result, "frames", &frames);
        set(&result, "recoveries", &recoveries);
        Ok(result.unchecked_into())
    }

    /// Completes the stream and rejects incomplete trailing input.
    pub fn finish(&mut self) -> Result<(), JsValue> {
        self.inner
            .finish()
            .map_err(|error| js_error(incomplete_frame_error_info(error)))
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

#[wasm_bindgen]
impl Frame {
    /// Creates an ACK frame.
    pub fn ack() -> Self {
        Self::from(CoreFrame::from(AckFrame::new()))
    }

    /// Creates a NACK frame.
    pub fn nack() -> Self {
        Self::from(CoreFrame::from(NackFrame::new()))
    }

    /// Creates a validated short frame.
    pub fn short(control: f64, address: f64) -> Result<Self, JsValue> {
        let frame = ShortFrame::new(
            Control::new(byte("control", control)?),
            Address::new(byte("address", address)?),
        )
        .map_err(|error| js_error(short_error_info(error)))?;
        Ok(Self::from(CoreFrame::from(frame)))
    }

    /// Creates a validated variable-format frame without user data.
    pub fn control(control: f64, address: f64, control_information: f64) -> Result<Self, JsValue> {
        let frame = ControlFrame::new(
            Control::new(byte("control", control)?),
            Address::new(byte("address", address)?),
            byte("controlInformation", control_information)?,
        )
        .map_err(|error| js_error(control_frame_error_info(error)))?;
        Ok(Self::from(CoreFrame::from(frame)))
    }

    /// Creates a validated variable-format frame containing user data.
    pub fn long(
        control: f64,
        address: f64,
        control_information: f64,
        user_data: &[u8],
    ) -> Result<Self, JsValue> {
        let frame = LongFrame::new(
            Control::new(byte("control", control)?),
            Address::new(byte("address", address)?),
            byte("controlInformation", control_information)?,
            user_data,
        )
        .map_err(|error| js_error(long_error_info(error)))?;
        Ok(Self::from(CoreFrame::from(frame)))
    }

    /// Decodes exactly one wired M-Bus frame.
    pub fn decode(bytes: &[u8]) -> Result<Self, JsValue> {
        decoder::exact::decode(bytes)
            .map(Self::from)
            .map_err(|error| js_error(decode_error_info(error)))
    }

    /// Returns the frame kind.
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        match self.inner {
            CoreFrame::Ack(_) => "ack",
            CoreFrame::Nack(_) => "nack",
            CoreFrame::Short(_) => "short",
            CoreFrame::Control(_) => "control",
            CoreFrame::Long(_) => "long",
        }
        .into()
    }

    /// Returns the control byte when present.
    #[wasm_bindgen(getter, js_name = controlByte)]
    pub fn control_byte(&self) -> Option<u8> {
        match &self.inner {
            CoreFrame::Short(frame) => Some(frame.control().value()),
            CoreFrame::Control(frame) => Some(frame.control().value()),
            CoreFrame::Long(frame) => Some(frame.control().value()),
            CoreFrame::Ack(_) | CoreFrame::Nack(_) => None,
        }
    }

    /// Returns the address byte when present.
    #[wasm_bindgen(getter)]
    pub fn address(&self) -> Option<u8> {
        match &self.inner {
            CoreFrame::Short(frame) => Some(frame.address().value()),
            CoreFrame::Control(frame) => Some(frame.address().value()),
            CoreFrame::Long(frame) => Some(frame.address().value()),
            CoreFrame::Ack(_) | CoreFrame::Nack(_) => None,
        }
    }

    /// Returns the control-information byte when present.
    #[wasm_bindgen(getter, js_name = controlInformation)]
    pub fn control_information(&self) -> Option<u8> {
        match &self.inner {
            CoreFrame::Control(frame) => Some(frame.control_information()),
            CoreFrame::Long(frame) => Some(frame.control_information()),
            CoreFrame::Ack(_) | CoreFrame::Nack(_) | CoreFrame::Short(_) => None,
        }
    }

    /// Returns a copy of the user-data bytes when present.
    #[wasm_bindgen(getter, js_name = userData)]
    pub fn user_data(&self) -> Option<Vec<u8>> {
        match &self.inner {
            CoreFrame::Long(frame) => Some(frame.user_data().to_vec()),
            CoreFrame::Ack(_)
            | CoreFrame::Nack(_)
            | CoreFrame::Short(_)
            | CoreFrame::Control(_) => None,
        }
    }

    /// Encodes the frame.
    pub fn encode(&self) -> Vec<u8> {
        self.inner.encode()
    }
}

impl From<CoreFrame> for Frame {
    fn from(inner: CoreFrame) -> Self {
        Self { inner }
    }
}

fn byte(name: &str, value: f64) -> Result<u8, JsValue> {
    if value.is_finite() && value.fract() == 0.0 && (0.0..=255.0).contains(&value) {
        Ok(value as u8)
    } else {
        Err(js_sys::RangeError::new(&format!("{name} must be an integer between 0 and 255")).into())
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

fn js_error(info: DatalinkErrorInfo) -> JsValue {
    let error = js_sys::Error::new(&info.message);
    error.set_name(info.name);
    for (name, value) in info.fields {
        set(
            error.as_ref(),
            name,
            &value.map_or(JsValue::NULL, JsValue::from),
        );
    }
    error.into()
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

fn set(object: &js_sys::Object, name: &str, value: &JsValue) {
    js_sys::Reflect::set(object, &JsValue::from(name), value)
        .expect("setting a property on a fresh object cannot fail");
}
