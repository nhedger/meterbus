// Decode one complete frame, inspect its fields, and encode it again.
import assert from "node:assert/strict";
import { Frame } from "@meterbus/wired-datalink-node";

const frame = Frame.decode(Uint8Array.of(0x10, 0x5b, 0x01, 0x5c, 0x16));

assert.equal(frame.kind, "short");
assert.equal(frame.controlByte, 0x5b);
assert.equal(frame.address, 1);
assert.deepEqual(frame.encode(), Uint8Array.of(0x10, 0x5b, 0x01, 0x5c, 0x16));
