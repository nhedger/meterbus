// Build and encode a standard REQ-UD2 meter-readout request for slave 1.
import init, { Frame } from "@meterbus/wired-datalink-wasm";

// WebAssembly exports are available after the module is initialized.
await init();

const request = Frame.short(0x5b, 1);
const encoded = request.encode();

console.assert(
	encoded.every(
		(byte, index) => byte === [0x10, 0x5b, 0x01, 0x5c, 0x16][index],
	),
);
