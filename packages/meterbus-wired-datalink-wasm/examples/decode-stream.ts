// Decode split and adjacent frames, then recover from malformed input.
import init, { StreamDecoder } from "@meterbus/wired-datalink-wasm";

// WebAssembly exports are available after the module is initialized.
await init();

const decoder = new StreamDecoder();

// The decoder retains the incomplete short-frame prefix between pushes.
const first = decoder.push(Uint8Array.of(0x10, 0x5b));
console.assert(first.frames.length === 0);

const second = decoder.push(Uint8Array.of(0x01, 0x5c, 0x16, 0xe5));
console.assert(second.frames.map((frame) => frame.kind).join() === "short,ack");
decoder.finish();

const recovering = StreamDecoder.resync();
// Resync mode reports discarded noise and continues with the following ACK.
const recovered = recovering.push(Uint8Array.of(0xff, 0x00, 0xe5));

console.assert(recovered.frames[0].kind === "ack");
console.assert(
	recovered.recoveries[0].discarded.every(
		(byte, index) => byte === [0xff, 0x00][index],
	),
);
