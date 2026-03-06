import { existsSync, readFileSync } from "node:fs"
import { readFile, writeFile } from "node:fs/promises"
import { resolve } from "node:path"
import { type Config, loadKoseiConfig, type Replacement } from "../config"
import { bold, dim, green, red, yellow } from "../format"

const applyReplacement = async (replacement: Replacement) => {
	const regexFlag = replacement.regex.match(/^\/.*\/([gimsuy]*)$/)?.[1] || "g"
	const regex = new RegExp(replacement.regex.slice(1, -1), regexFlag)

	await Promise.all(
		replacement.files.map(async (file) => {
			const absPath = resolve(process.cwd(), file)
			const content = await readFile(absPath, "utf8")
			const updated = content.replace(regex, replacement.value)
			if (content !== updated) await writeFile(absPath, updated, "utf8")
		}),
	)
}

const previewReplacement = (replacement: Replacement) => {
	const regexFlag = replacement.regex.match(/^\/.*\/([gimsuy]*)$/)?.[1] || "g"
	const regex = new RegExp(replacement.regex.slice(1, -1), regexFlag)

	for (const file of replacement.files) {
		const absPath = resolve(process.cwd(), file)
		const original = readFileSync(absPath, "utf8")
		const updated = original.replace(regex, replacement.value)

		if (original === updated) {
			console.log(dim(`${file}: no changes`))
			continue
		}

		console.log(bold(`\n${file}:`))
		const originalLines = original.split("\n")
		const updatedLines = updated.split("\n")

		for (
			let i = 0;
			i < Math.max(originalLines.length, updatedLines.length);
			i++
		) {
			const before = originalLines[i]
			const after = updatedLines[i]
			if (before !== after) {
				if (before !== undefined) console.log(red(`- ${before}`))
				if (after !== undefined) console.log(green(`+ ${after}`))
			}
		}
	}
}

const switchAction = async (config: Config, env: string, dryRun: boolean) => {
	const environment = config.environments[env]

	if (!environment) {
		console.error(
			red(
				`Unknown environment "${env}". Available: ${Object.keys(config.environments).join(", ")}`,
			),
		)
		process.exit(1)
	}

	for (const r of environment.replacements) {
		const missingFiles = r.files.filter((f) => !existsSync(f))
		if (missingFiles.length > 0) {
			console.warn(
				yellow(
					`Warning: There were issues resolving files from paths:\n${missingFiles.map((f) => `- ${f}`).join("\n")}`,
				),
			)
			process.exit(0)
		}

		if (dryRun) {
			previewReplacement(r)
		} else {
			await applyReplacement(r)
		}
	}
}

export const switchCommand = async (args: string[]) => {
	const env = args.find((a) => !a.startsWith("-"))
	const dryRun = args.includes("--dry-run")

	if (!env) {
		console.error(red("Usage: kosei switch <env> [--dry-run]"))
		process.exit(1)
	}

	try {
		const config = await loadKoseiConfig()
		await switchAction(config, env, dryRun)
	} catch (err) {
		console.error(`Error switching environment: ${(err as Error).message}`)
		process.exit(1)
	}
}
