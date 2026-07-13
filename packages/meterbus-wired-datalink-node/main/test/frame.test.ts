import { describe, expect, it } from "vitest";
import {
	DatalinkError,
	Frame,
	IncompleteFrameError,
	InvalidControlFrameControlError,
	InvalidLongFrameUserDataLengthError,
	InvalidShortFrameChecksumError,
	InvalidShortFrameControlError,
	StreamDecoder,
	UnknownStartByteError,
} from "../src/index.js";

type FrameCase = readonly [frame: Frame, kind: string, hex: string];

const longFrame = Frame.long(0x53, 0xfe, 0x50, Uint8Array.of(0x10));

const cases: readonly FrameCase[] = [
	[Frame.ack(), "ack", "e5"],
	[Frame.nack(), "nack", "a2"],
	[Frame.short(0x5b, 1), "short", "105b015c16"],
	[Frame.control(0x53, 0xfe, 0xbd), "control", "6803036853febd0e16"],
	[longFrame, "long", "6804046853fe5010b116"],
];

describe("Frame", () => {
	it("constructs, encodes, and decodes every frame kind", () => {
		for (const [frame, kind, hex] of cases) {
			expect(frame.kind).toBe(kind);
			expect(Buffer.from(frame.encode()).toString("hex")).toBe(hex);

			const decoded = Frame.decode(frame.encode());
			expect(decoded.kind).toBe(kind);
			expect(Buffer.from(decoded.encode()).toString("hex")).toBe(hex);
		}
	});

	it("returns copies of user data", () => {
		const userData = longFrame.userData;
		if (userData === null) {
			throw new Error("long frame did not expose user data");
		}
		userData[0] = 0xff;

		expect(Buffer.from(longFrame.encode()).toString("hex")).toBe(
			"6804046853fe5010b116",
		);
	});

	it("rejects invalid inputs", () => {
		expect(() => Frame.short(256, 1)).toThrow(/integer between 0 and 255/);
		expect(() => Frame.short(0x53, 1)).toThrow(InvalidShortFrameControlError);
		expect(() => Frame.control(0x40, 1, 0)).toThrow(
			InvalidControlFrameControlError,
		);
		expect(() => Frame.long(0x53, 1, 0, new Uint8Array())).toThrow(
			InvalidLongFrameUserDataLengthError,
		);
		try {
			Frame.decode(Uint8Array.of(0xff));
			expect.unreachable();
		} catch (error) {
			expect(error).toBeInstanceOf(UnknownStartByteError);
			expect(error).toBeInstanceOf(DatalinkError);
			expect((error as UnknownStartByteError).actual).toBe(0xff);
		}

		try {
			Frame.decode(Uint8Array.of(0x10, 0x5b, 0x01, 0, 0x16));
			expect.unreachable();
		} catch (error) {
			expect(error).toBeInstanceOf(InvalidShortFrameChecksumError);
			expect((error as InvalidShortFrameChecksumError).expected).toBe(0x5c);
			expect((error as InvalidShortFrameChecksumError).actual).toBe(0);
		}
	});
});

describe("StreamDecoder", () => {
	it("decodes frames split across chunks", () => {
		const decoder = new StreamDecoder();
		const first = decoder.push(Uint8Array.of(0x10, 0x5b));

		expect(first.frames).toHaveLength(0);
		expect(decoder.bufferedBytes).toBe(2);

		const second = decoder.push(Uint8Array.of(0x01, 0x5c, 0x16));
		expect(second.frames).toHaveLength(1);
		expect(Buffer.from(second.frames[0].encode()).toString("hex")).toBe(
			"105b015c16",
		);
		expect(decoder.bufferedBytes).toBe(0);
		expect(() => decoder.finish()).not.toThrow();
	});

	it("reports and clears incomplete trailing input", () => {
		const decoder = new StreamDecoder();
		decoder.push(Uint8Array.of(0x10, 0x5b));

		try {
			decoder.finish();
			expect.unreachable();
		} catch (error) {
			expect(error).toBeInstanceOf(IncompleteFrameError);
			expect((error as IncompleteFrameError).receivedBytes).toBe(2);
			expect((error as IncompleteFrameError).expectedLength).toBe(5);
		}
		expect(decoder.bufferedBytes).toBe(0);
	});

	it("reports malformed strict input", () => {
		const decoder = new StreamDecoder();

		expect(() => decoder.push(Uint8Array.of(0xff))).toThrow(
			UnknownStartByteError,
		);
	});

	it("resynchronizes after malformed bytes", () => {
		const decoder = StreamDecoder.resync();
		const result = decoder.push(Uint8Array.of(0xff, 0xe5));

		expect(result.frames).toHaveLength(1);
		expect(result.frames[0].kind).toBe("ack");
		expect(result.recoveries).toHaveLength(1);
		expect(result.recoveries[0].error).toBeInstanceOf(UnknownStartByteError);
		expect((result.recoveries[0].error as UnknownStartByteError).actual).toBe(
			0xff,
		);
		expect(result.recoveries[0].discarded).toEqual(Uint8Array.of(0xff));
	});

	it("resets buffered input", () => {
		const decoder = new StreamDecoder();
		decoder.push(Uint8Array.of(0x10, 0x5b));
		decoder.reset();

		expect(decoder.bufferedBytes).toBe(0);
		expect(() => decoder.finish()).not.toThrow();
	});
});
