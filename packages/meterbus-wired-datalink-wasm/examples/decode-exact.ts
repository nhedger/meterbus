// Decode one complete frame, inspect its fields, and encode it again.
import init, { Frame } from "@meterbus/wired-datalink-wasm";

// WebAssembly exports are available after the module is initialized.
await init();

const frame = Frame.decode(Uint8Array.of(0x10, 0x5b, 0x01, 0x5c, 0x16));

console.assert(frame.kind === "short");
console.assert(frame.controlByte === 0x5b);
console.assert(frame.address === 1);
console.assert(
	frame
		.encode()
		.every((byte, index) => byte === [0x10, 0x5b, 0x01, 0x5c, 0x16][index]),
);
