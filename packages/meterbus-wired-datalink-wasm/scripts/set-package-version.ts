import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const packageRoot = resolve(import.meta.dirname, "..");
const cargoManifest = await readFile(
	resolve(packageRoot, "../../crates/meterbus-wired-datalink/Cargo.toml"),
	"utf8",
);
const packagePath = resolve(packageRoot, "package.json");
const packageManifest = await readFile(packagePath, "utf8");
const cargoVersion = cargoManifest.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

if (cargoVersion === undefined) {
	throw new Error("could not read the meterbus-wired-datalink crate version");
}

const version = process.argv.includes("--reset") ? "0.0.0" : cargoVersion;
const updated = packageManifest.replace(
	/^(\s*"version"\s*:\s*)"[^"]+"/m,
	`$1"${version}"`,
);

if (updated === packageManifest) {
	throw new Error("could not update the package version");
}

await writeFile(packagePath, updated);
console.log(`set package version to ${version}`);
