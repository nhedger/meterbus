from typing import Literal

FrameKind = Literal["ack", "nack", "short", "control", "long"]

class DatalinkError(Exception): ...

class AckOutputTooSmallError(DatalinkError):
    actual: int

class ControlFrameOutputTooSmallError(DatalinkError):
    actual: int

class EmptyInputError(DatalinkError): ...

class IncompleteFrameError(DatalinkError):
    received_bytes: int
    expected_length: int | None

class IncompleteLongFrameHeaderError(DatalinkError):
    actual: int

class IncompleteVariableHeaderError(DatalinkError): ...

class InvalidAckByteError(DatalinkError):
    actual: int

class InvalidAckLengthError(DatalinkError):
    actual: int

class InvalidControlFrameChecksumError(DatalinkError):
    expected: int
    actual: int

class InvalidControlFrameControlError(DatalinkError):
    value: int

class InvalidControlFrameDataLengthError(DatalinkError):
    index: int
    actual: int

class InvalidControlFrameLengthError(DatalinkError):
    actual: int

class InvalidControlFrameStartError(DatalinkError):
    index: int
    actual: int

class InvalidControlFrameStopError(DatalinkError):
    actual: int

class InvalidLongFrameChecksumError(DatalinkError):
    expected: int
    actual: int

class InvalidLongFrameControlError(DatalinkError):
    value: int

class InvalidLongFrameDataLengthError(DatalinkError):
    actual: int

class InvalidLongFrameLengthError(DatalinkError):
    expected: int
    actual: int

class InvalidLongFrameStartError(DatalinkError):
    index: int
    actual: int

class InvalidLongFrameStopError(DatalinkError):
    actual: int

class InvalidLongFrameUserDataLengthError(DatalinkError):
    actual: int

class InvalidNackByteError(DatalinkError):
    actual: int

class InvalidNackLengthError(DatalinkError):
    actual: int

class InvalidShortFrameChecksumError(DatalinkError):
    expected: int
    actual: int

class InvalidShortFrameControlError(DatalinkError):
    value: int

class InvalidShortFrameLengthError(DatalinkError):
    actual: int

class InvalidShortFrameStartError(DatalinkError):
    actual: int

class InvalidShortFrameStopError(DatalinkError):
    actual: int

class LongFrameOutputTooSmallError(DatalinkError):
    required: int
    actual: int

class MismatchedLongFrameDataLengthsError(DatalinkError):
    first: int
    second: int

class NackOutputTooSmallError(DatalinkError):
    actual: int

class ShortFrameOutputTooSmallError(DatalinkError):
    actual: int

class UnknownStartByteError(DatalinkError):
    actual: int

class Frame:
    @staticmethod
    def ack() -> Frame: ...
    @staticmethod
    def nack() -> Frame: ...
    @staticmethod
    def short(control: int, address: int) -> Frame: ...
    @staticmethod
    def control(
        control: int,
        address: int,
        control_information: int,
    ) -> Frame: ...
    @staticmethod
    def long(
        control: int,
        address: int,
        control_information: int,
        user_data: bytes | bytearray,
    ) -> Frame: ...
    @staticmethod
    def decode(data: bytes | bytearray) -> Frame: ...
    @property
    def kind(self) -> FrameKind: ...
    @property
    def control_byte(self) -> int | None: ...
    @property
    def address(self) -> int | None: ...
    @property
    def control_information(self) -> int | None: ...
    @property
    def user_data(self) -> bytes | None: ...
    def encode(self) -> bytes: ...

class StreamRecovery:
    @property
    def error(self) -> DatalinkError: ...
    @property
    def discarded(self) -> bytes: ...

class StreamPushResult:
    @property
    def frames(self) -> list[Frame]: ...
    @property
    def recoveries(self) -> list[StreamRecovery]: ...

class StreamDecoder:
    def __init__(self) -> None: ...
    @staticmethod
    def resync() -> StreamDecoder: ...
    @property
    def buffered_bytes(self) -> int: ...
    def push(self, chunk: bytes | bytearray) -> StreamPushResult: ...
    def finish(self) -> None: ...
    def reset(self) -> None: ...
