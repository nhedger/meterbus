import { readFile, readdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

interface PackageManifest {
	name: string;
	version: string;
	optionalDependencies?: Record<string, string>;
}

const mainRoot = resolve(import.meta.dirname, "..");
const packageGroup = resolve(mainRoot, "..");
const architectureRoot = resolve(packageGroup, "arch");
const cargoManifest = await readFile(
	resolve(packageGroup, "../../crates/meterbus-wired-datalink/Cargo.toml"),
	"utf8",
);
const cargoVersion = cargoManifest.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

if (cargoVersion === undefined) {
	throw new Error("could not read the meterbus-wired-datalink crate version");
}

const version = process.argv.includes("--reset") ? "0.0.0" : cargoVersion;
const architectureDirectories = (await readdir(architectureRoot, {
	withFileTypes: true,
}))
	.filter((entry) => entry.isDirectory())
	.map((entry) => entry.name)
	.sort();
const architecturePackages = new Map<string, string>();

for (const directory of architectureDirectories) {
	const packagePath = resolve(architectureRoot, directory, "package.json");
	const manifest = JSON.parse(
		await readFile(packagePath, "utf8"),
	) as PackageManifest;
	manifest.version = version;
	architecturePackages.set(manifest.name, version);
	await writeFile(packagePath, `${JSON.stringify(manifest, null, "\t")}\n`);
}

const mainPackagePath = resolve(mainRoot, "package.json");
const mainManifest = JSON.parse(
	await readFile(mainPackagePath, "utf8"),
) as PackageManifest;
mainManifest.version = version;
mainManifest.optionalDependencies = Object.fromEntries(architecturePackages);
await writeFile(mainPackagePath, `${JSON.stringify(mainManifest, null, "\t")}\n`);

console.log(
	`set ${architecturePackages.size + 1} package versions to ${version}`,
);
