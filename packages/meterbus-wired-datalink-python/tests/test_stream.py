from __future__ import annotations

import pytest

from meterbus_wired_datalink import (
    IncompleteFrameError,
    StreamDecoder,
    UnknownStartByteError,
)


def test_decodes_frames_split_across_chunks() -> None:
    decoder = StreamDecoder()

    first = decoder.push(bytes.fromhex("105b"))
    assert first.frames == []
    assert decoder.buffered_bytes == 2

    second = decoder.push(bytes.fromhex("015c16"))
    assert [frame.kind for frame in second.frames] == ["short"]
    assert second.frames[0].encode() == bytes.fromhex("105b015c16")
    assert decoder.buffered_bytes == 0
    decoder.finish()


def test_reports_and_clears_incomplete_input() -> None:
    decoder = StreamDecoder()
    decoder.push(bytes.fromhex("105b"))

    with pytest.raises(IncompleteFrameError, match="incomplete frame") as caught:
        decoder.finish()
    assert caught.value.received_bytes == 2
    assert caught.value.expected_length == 5

    assert decoder.buffered_bytes == 0


def test_resynchronizes_after_malformed_bytes() -> None:
    decoder = StreamDecoder.resync()
    result = decoder.push(bytes.fromhex("ff00e5"))

    assert [frame.kind for frame in result.frames] == ["ack"]
    assert len(result.recoveries) == 1
    assert isinstance(result.recoveries[0].error, UnknownStartByteError)
    assert result.recoveries[0].error.actual == 0xFF
    assert result.recoveries[0].discarded == bytes.fromhex("ff00")


def test_resets_buffered_input() -> None:
    decoder = StreamDecoder()
    decoder.push(bytes.fromhex("105b"))
    decoder.reset()

    assert decoder.buffered_bytes == 0
    decoder.finish()
