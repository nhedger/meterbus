"""Build and encode a standard REQ-UD2 meter-readout request for slave 1."""

from meterbus_wired_datalink import Frame

request = Frame.short(0x5B, 1)
encoded = request.encode()

assert encoded == bytes.fromhex("105b015c16")
