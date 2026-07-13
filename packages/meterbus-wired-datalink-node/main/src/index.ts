import * as native from "../.napi/index.js";
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
export type Frame = native.Frame;
export const Frame = {
	ack(): Frame {
		return native.Frame.ack();
	},
	nack(): Frame {
		return native.Frame.nack();
	},
	short(control: number, address: number): Frame {
		try {
			return native.Frame.short(control, address);
		} catch (error) {
			throw asDatalinkError(error);
		}
	},
	control(control: number, address: number, controlInformation: number): Frame {
		try {
			return native.Frame.control(control, address, controlInformation);
		} catch (error) {
			throw asDatalinkError(error);
		}
	},
	long(
		control: number,
		address: number,
		controlInformation: number,
		userData: Uint8Array,
	): Frame {
		try {
			return native.Frame.long(control, address, controlInformation, userData);
		} catch (error) {
			throw asDatalinkError(error);
		}
	},
	decode(bytes: Uint8Array): Frame {
		try {
			return native.Frame.decode(bytes);
		} catch (error) {
			throw asDatalinkError(error);
		}
	},
};

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

for (const method of ["push", "finish"]) {
	wrapMethod(native.StreamDecoder.prototype, method);
}

const recoveryError = Object.getOwnPropertyDescriptor(
	native.StreamRecovery.prototype,
	"error",
) as PropertyDescriptor & { get: () => unknown };
Object.defineProperty(native.StreamRecovery.prototype, "error", {
	...recoveryError,
	get() {
		return asDatalinkError(Reflect.apply(recoveryError.get, this, []));
	},
});
