"""Decode one complete frame, inspect its fields, and encode it again."""

from meterbus_wired_datalink import Frame

frame = Frame.decode(bytes.fromhex("105b015c16"))

assert frame.kind == "short"
assert frame.control_byte == 0x5B
assert frame.address == 1
assert frame.encode() == bytes.fromhex("105b015c16")
