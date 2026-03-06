import { existsSync } from "node:fs"
import { readFile } from "node:fs/promises"
import { dirname, join } from "node:path"
import { bold, red } from "./format"

export const ROOT_KOSEI_CONFIG_FILE_NAME = "kosei.config.json"

export interface Replacement {
	files: string[]
	regex: string
	value: string
}

export interface Environment {
	description?: string
	replacements: Replacement[]
}

export interface Config {
	environments: Record<string, Environment>
}

const REGEX_LITERAL = /^\/(.+)\/([gimsuy]*)$/

function validateConfig(raw: unknown): Config {
	if (!raw || typeof raw !== "object" || Array.isArray(raw))
		throw new Error("config must be an object")

	const { environments } = raw as Record<string, unknown>
	if (
		!environments ||
		typeof environments !== "object" ||
		Array.isArray(environments)
	)
		throw new Error("config.environments must be an object")

	for (const [envName, env] of Object.entries(
		environments as Record<string, unknown>,
	)) {
		if (!env || typeof env !== "object" || Array.isArray(env))
			throw new Error(`environments.${envName} must be an object`)

		const { replacements } = env as Record<string, unknown>
		if (!Array.isArray(replacements) || replacements.length === 0)
			throw new Error(
				`environments.${envName}.replacements must be a non-empty array`,
			)

		for (let i = 0; i < replacements.length; i++) {
			const r = replacements[i]
			const path = `environments.${envName}.replacements[${i}]`
			if (!r || typeof r !== "object")
				throw new Error(`${path} must be an object`)
			const { files, regex, value } = r as Record<string, unknown>
			if (
				!Array.isArray(files) ||
				files.length === 0 ||
				!files.every((f) => typeof f === "string")
			)
				throw new Error(`${path}.files must be a non-empty array of strings`)
			if (typeof regex !== "string" || !REGEX_LITERAL.test(regex))
				throw new Error(
					`${path}.regex must be a regex literal string like "/pattern/flags"`,
				)
			if (typeof value !== "string")
				throw new Error(`${path}.value must be a string`)
		}
	}

	return raw as Config
}

const findConfigRoot = (startDir: string): string | null => {
	let dir = startDir
	while (true) {
		if (existsSync(join(dir, ROOT_KOSEI_CONFIG_FILE_NAME))) return dir
		const parent = dirname(dir)
		if (parent === dir) return null
		dir = parent
	}
}

export const loadKoseiConfig = async (): Promise<Config> => {
	const root = findConfigRoot(process.cwd())

	if (!root) {
		console.error(
			red(
				`Could not find ${bold(ROOT_KOSEI_CONFIG_FILE_NAME)} in the current directory or any parent directories.`,
			),
		)
		process.exit(1)
	}

	const configPath = join(root, ROOT_KOSEI_CONFIG_FILE_NAME)
	const content = await readFile(configPath, "utf-8")

	let raw: unknown
	try {
		raw = JSON.parse(content)
	} catch (err) {
		console.error(
			red(
				`Failed to parse ${bold(ROOT_KOSEI_CONFIG_FILE_NAME)}: ${(err as Error).message}`,
			),
		)
		process.exit(1)
	}

	try {
		return validateConfig(raw)
	} catch (err) {
		console.error(
			red(
				`Invalid ${bold(ROOT_KOSEI_CONFIG_FILE_NAME)}: ${(err as Error).message}`,
			),
		)
		process.exit(1)
	}
}
