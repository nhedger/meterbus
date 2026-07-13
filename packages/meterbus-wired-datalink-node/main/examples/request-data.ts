// Build and encode a standard REQ-UD2 meter-readout request for slave 1.
import assert from "node:assert/strict";
import { Frame } from "@meterbus/wired-datalink-node";

const request = Frame.short(0x5b, 1);
const encoded = request.encode();

assert.deepEqual(encoded, Uint8Array.of(0x10, 0x5b, 0x01, 0x5c, 0x16));
