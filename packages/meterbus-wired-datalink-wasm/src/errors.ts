export class DatalinkError extends Error {
	constructor(message?: string, options?: ErrorOptions) {
		super(message, options);
		this.name = new.target.name;
	}
}

export class AckOutputTooSmallError extends DatalinkError {
	declare readonly actual: number;
}
export class ControlFrameOutputTooSmallError extends DatalinkError {
	declare readonly actual: number;
}
export class EmptyInputError extends DatalinkError {}
export class IncompleteFrameError extends DatalinkError {
	declare readonly receivedBytes: number;
	declare readonly expectedLength: number | null;
}
export class IncompleteLongFrameHeaderError extends DatalinkError {
	declare readonly actual: number;
}
export class IncompleteVariableHeaderError extends DatalinkError {}
export class InvalidAckByteError extends DatalinkError {
	declare readonly actual: number;
}
export class InvalidAckLengthError extends DatalinkError {
	declare readonly actual: number;
}
export class InvalidControlFrameChecksumError extends DatalinkError {
	declare readonly expected: number;
	declare readonly actual: number;
}
export class InvalidControlFrameControlError extends DatalinkError {
	declare readonly value: number;
}
export class InvalidControlFrameDataLengthError extends DatalinkError {
	declare readonly index: number;
	declare readonly actual: number;
}
export class InvalidControlFrameLengthError extends DatalinkError {
	declare readonly actual: number;
}
export class InvalidControlFrameStartError extends DatalinkError {
	declare readonly index: number;
	declare readonly actual: number;
}
export class InvalidControlFrameStopError extends DatalinkError {
	declare readonly actual: number;
}
export class InvalidLongFrameChecksumError extends DatalinkError {
	declare readonly expected: number;
	declare readonly actual: number;
}
export class InvalidLongFrameControlError extends DatalinkError {
	declare readonly value: number;
}
export class InvalidLongFrameDataLengthError extends DatalinkError {
	declare readonly actual: number;
}
export class InvalidLongFrameLengthError extends DatalinkError {
	declare readonly expected: number;
	declare readonly actual: number;
}
export class InvalidLongFrameStartError extends DatalinkError {
	declare readonly index: number;
	declare readonly actual: number;
}
export class InvalidLongFrameStopError extends DatalinkError {
	declare readonly actual: number;
}
export class InvalidLongFrameUserDataLengthError extends DatalinkError {
	declare readonly actual: number;
}
export class InvalidNackByteError extends DatalinkError {
	declare readonly actual: number;
}
export class InvalidNackLengthError extends DatalinkError {
	declare readonly actual: number;
}
export class InvalidShortFrameChecksumError extends DatalinkError {
	declare readonly expected: number;
	declare readonly actual: number;
}
export class InvalidShortFrameControlError extends DatalinkError {
	declare readonly value: number;
}
export class InvalidShortFrameLengthError extends DatalinkError {
	declare readonly actual: number;
}
export class InvalidShortFrameStartError extends DatalinkError {
	declare readonly actual: number;
}
export class InvalidShortFrameStopError extends DatalinkError {
	declare readonly actual: number;
}
export class LongFrameOutputTooSmallError extends DatalinkError {
	declare readonly required: number;
	declare readonly actual: number;
}
export class MismatchedLongFrameDataLengthsError extends DatalinkError {
	declare readonly first: number;
	declare readonly second: number;
}
export class NackOutputTooSmallError extends DatalinkError {
	declare readonly actual: number;
}
export class ShortFrameOutputTooSmallError extends DatalinkError {
	declare readonly actual: number;
}
export class UnknownStartByteError extends DatalinkError {
	declare readonly actual: number;
}

const errorTypes: Readonly<Record<string, typeof DatalinkError>> = {
	AckOutputTooSmallError,
	ControlFrameOutputTooSmallError,
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
};

export function asDatalinkError(error: unknown): unknown {
	if (!(error instanceof Error)) {
		return error;
	}

	const ErrorType = errorTypes[error.name];
	if (ErrorType !== undefined) {
		Object.setPrototypeOf(error, ErrorType.prototype);
	}
	return error;
}
