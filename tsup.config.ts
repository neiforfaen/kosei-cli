import { defineConfig, type Options } from "tsup"

export default defineConfig((options: Options) => ({
	entry: ["src/cli.ts"],
	format: ["cjs"],
	dts: false,
	sourcemap: false,

	bundle: true,
	external: [],
	shims: true,

	platform: "node",
	target: "node20",
	keepNames: true,

	noExternal: [/^@?[^./]|^\.\//],
	banner: { js: "#!/usr/bin/env node" },

	clean: true,

	watch: options.watch,
	treeshake: !options.watch,
	minify: !options.watch,
}))
