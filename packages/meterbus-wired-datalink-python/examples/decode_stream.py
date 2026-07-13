"""Decode split and adjacent frames, then recover from malformed input."""

from meterbus_wired_datalink import StreamDecoder

decoder = StreamDecoder()

# The decoder retains the incomplete short-frame prefix between pushes.
first = decoder.push(bytes.fromhex("105b"))
assert first.frames == []

second = decoder.push(bytes.fromhex("015c16e5"))
assert [frame.kind for frame in second.frames] == ["short", "ack"]
decoder.finish()

recovering = StreamDecoder.resync()
# Resync mode reports discarded noise and continues with the following ACK.
recovered = recovering.push(bytes.fromhex("ff00e5"))

assert [frame.kind for frame in recovered.frames] == ["ack"]
assert [event.discarded for event in recovered.recoveries] == [bytes.fromhex("ff00")]
