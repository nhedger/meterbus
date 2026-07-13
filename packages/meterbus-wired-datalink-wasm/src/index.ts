import init, * as native from "../.wasm/index.js";
import type { DatalinkError } from "./errors.js";
import { asDatalinkError } from "./errors.js";

export {
	AckOutputTooSmallError,
	ControlFrameOutputTooSmallError,
	DatalinkError,
	EmptyInputError,
	IncompleteFrameError,
	IncompleteLongFrameHeaderError,
	IncompleteVariableHeaderError,
	InvalidAckByteError,
	InvalidAckLengthError,
	InvalidControlFrameChecksumError,
	InvalidControlFrameControlError,
	InvalidControlFrameDataLengthError,
	InvalidControlFrameLengthError,
	InvalidControlFrameStartError,
	InvalidControlFrameStopError,
	InvalidLongFrameChecksumError,
	InvalidLongFrameControlError,
	InvalidLongFrameDataLengthError,
	InvalidLongFrameLengthError,
	InvalidLongFrameStartError,
	InvalidLongFrameStopError,
	InvalidLongFrameUserDataLengthError,
	InvalidNackByteError,
	InvalidNackLengthError,
	InvalidShortFrameChecksumError,
	InvalidShortFrameControlError,
	InvalidShortFrameLengthError,
	InvalidShortFrameStartError,
	InvalidShortFrameStopError,
	LongFrameOutputTooSmallError,
	MismatchedLongFrameDataLengthsError,
	NackOutputTooSmallError,
	ShortFrameOutputTooSmallError,
	UnknownStartByteError,
} from "./errors.js";

export default init;
export const Frame = native.Frame;
export type Frame = native.Frame;

export interface StreamRecovery {
	readonly error: DatalinkError;
	readonly discarded: Uint8Array;
}

export interface StreamPushResult {
	readonly frames: Frame[];
	readonly recoveries: StreamRecovery[];
}

export interface StreamDecoder extends Omit<native.StreamDecoder, "push"> {
	push(chunk: Uint8Array): StreamPushResult;
}

export const StreamDecoder = native.StreamDecoder as unknown as {
	new (): StreamDecoder;
	resync(): StreamDecoder;
};

type Method = (...arguments_: never[]) => unknown;

function wrapMethod(target: object, name: string): void {
	const method = Reflect.get(target, name) as Method;
	Reflect.set(target, name, function (this: unknown, ...arguments_: never[]) {
		try {
			return Reflect.apply(method, this, arguments_);
		} catch (error) {
			throw asDatalinkError(error);
		}
	});
}

for (const method of ["short", "control", "long", "decode"]) {
	wrapMethod(native.Frame, method);
}
wrapMethod(native.StreamDecoder.prototype, "finish");

const nativePush = native.StreamDecoder.prototype.push;
native.StreamDecoder.prototype.push = function (
	chunk: Uint8Array,
): native.StreamPushResult {
	try {
		const result = nativePush.call(this, chunk);
		for (const recovery of result.recoveries) {
			Reflect.set(recovery, "error", asDatalinkError(recovery.error));
		}
		return result;
	} catch (error) {
		throw asDatalinkError(error);
	}
};
