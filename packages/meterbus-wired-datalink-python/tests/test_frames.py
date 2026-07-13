from __future__ import annotations

import pytest

from meterbus_wired_datalink import (
    DatalinkError,
    Frame,
    InvalidShortFrameChecksumError,
    InvalidShortFrameControlError,
    UnknownStartByteError,
)


@pytest.mark.parametrize(
    ("frame", "kind", "encoded"),
    [
        (Frame.ack(), "ack", bytes.fromhex("e5")),
        (Frame.nack(), "nack", bytes.fromhex("a2")),
        (Frame.short(0x5B, 1), "short", bytes.fromhex("105b015c16")),
        (
            Frame.control(0x53, 0xFE, 0xBD),
            "control",
            bytes.fromhex("6803036853febd0e16"),
        ),
        (
            Frame.long(0x53, 0xFE, 0x50, b"\x10"),
            "long",
            bytes.fromhex("6804046853fe5010b116"),
        ),
    ],
)
def test_constructs_encodes_and_decodes_frames(
    frame: Frame,
    kind: str,
    encoded: bytes,
) -> None:
    assert frame.kind == kind
    assert frame.encode() == encoded

    decoded = Frame.decode(encoded)
    assert decoded.kind == kind
    assert decoded.encode() == encoded


def test_exposes_frame_fields() -> None:
    frame = Frame.long(0x53, 0xFE, 0x50, b"\x10")

    assert frame.control_byte == 0x53
    assert frame.address == 0xFE
    assert frame.control_information == 0x50
    assert frame.user_data == b"\x10"


def test_rejects_invalid_inputs() -> None:
    with pytest.raises(OverflowError):
        Frame.short(256, 1)

    with pytest.raises(
        InvalidShortFrameControlError, match="invalid for a short frame"
    ) as control_error:
        Frame.short(0x53, 1)
    assert isinstance(control_error.value, DatalinkError)
    assert control_error.value.value == 0x53

    with pytest.raises(
        UnknownStartByteError, match="unknown frame start byte"
    ) as start_error:
        Frame.decode(b"\xff")
    assert start_error.value.actual == 0xFF

    with pytest.raises(InvalidShortFrameChecksumError) as checksum_error:
        Frame.decode(bytes.fromhex("105b010016"))
    assert checksum_error.value.expected == 0x5C
    assert checksum_error.value.actual == 0
