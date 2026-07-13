import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "tsdown";

const root = fileURLToPath(new URL(".", import.meta.url));

export default defineConfig({
	entry: { index: resolve(root, "src/index.ts") },
	format: "esm",
	platform: "node",
	target: "node24",
	outDir: "dist",
	clean: true,
	dts: true,
	sourcemap: true,
	fixedExtension: false,
});
