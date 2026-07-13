// Decode split and adjacent frames, then recover from malformed input.
import assert from "node:assert/strict";
import { StreamDecoder } from "@meterbus/wired-datalink-node";

const decoder = new StreamDecoder();

// The decoder retains the incomplete short-frame prefix between pushes.
const first = decoder.push(Uint8Array.of(0x10, 0x5b));
assert.equal(first.frames.length, 0);

const second = decoder.push(Uint8Array.of(0x01, 0x5c, 0x16, 0xe5));
assert.deepEqual(
	second.frames.map((frame) => frame.kind),
	["short", "ack"],
);
decoder.finish();

const recovering = StreamDecoder.resync();
// Resync mode reports discarded noise and continues with the following ACK.
const recovered = recovering.push(Uint8Array.of(0xff, 0x00, 0xe5));

assert.deepEqual(
	recovered.frames.map((frame) => frame.kind),
	["ack"],
);
assert.deepEqual(
	recovered.recoveries.map((event) => event.discarded),
	[Uint8Array.of(0xff, 0x00)],
);
