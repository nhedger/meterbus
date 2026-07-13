import { describe, expect, it } from "vitest";
import {
	asDatalinkError,
	DatalinkError,
	UnknownStartByteError,
} from "../src/errors.js";

describe("DatalinkError", () => {
	it("sets the concrete error name", () => {
		const cause = new Error("cause");
		const error = new UnknownStartByteError("message", { cause });

		expect(error.name).toBe("UnknownStartByteError");
		expect(error.message).toBe("message");
		expect(error.cause).toBe(cause);
		expect(error).toBeInstanceOf(DatalinkError);
	});

	it("restores known native prototypes", () => {
		const error = new Error("message");
		error.name = "UnknownStartByteError";

		expect(asDatalinkError(error)).toBe(error);
		expect(error).toBeInstanceOf(UnknownStartByteError);
	});

	it("leaves unknown values unchanged", () => {
		const error = new Error("message");

		expect(asDatalinkError(error)).toBe(error);
		expect(asDatalinkError("message")).toBe("message");
	});
});
