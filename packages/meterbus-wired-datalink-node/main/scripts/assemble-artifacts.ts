import { copyFile, mkdir, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { parseArgs } from "node:util";

const binaryName = "meterbus-wired-datalink";
const targets = [
	"darwin-arm64",
	"darwin-x64",
	"linux-arm64-gnu",
	"linux-arm64-musl",
	"linux-x64-gnu",
	"linux-x64-musl",
	"win32-arm64-msvc",
	"win32-x64-msvc",
] as const;
const { values } = parseArgs({
	options: {
		artifacts: { type: "string" },
		loader: { type: "string" },
	},
	strict: true,
});

if (values.artifacts === undefined || values.loader === undefined) {
	throw new Error("--artifacts and --loader are required");
}

const mainRoot = resolve(import.meta.dirname, "..");
const packageGroup = resolve(mainRoot, "..");
const artifactsRoot = resolve(values.artifacts);
const loaderRoot = resolve(values.loader);
const expectedBinaries = targets.map(
	(target) => `${binaryName}.${target}.node`,
);
const actualBinaries = (await readdir(artifactsRoot))
	.filter((file) => file.endsWith(".node"))
	.sort();

if (actualBinaries.join("\n") !== [...expectedBinaries].sort().join("\n")) {
	throw new Error(
		`native artifacts do not match the expected targets:\n${actualBinaries.join("\n")}`,
	);
}

for (const target of targets) {
	const binary = `${binaryName}.${target}.node`;
	await copyFile(
		resolve(artifactsRoot, binary),
		resolve(packageGroup, "arch", target, binary),
	);
}

const napiRoot = resolve(mainRoot, ".napi");
await mkdir(napiRoot, { recursive: true });
await Promise.all(
	["index.js", "index.d.ts"].map((file) =>
		copyFile(resolve(loaderRoot, file), resolve(napiRoot, file)),
	),
);

console.log(`assembled ${targets.length} native packages and the JavaScript loader`);
