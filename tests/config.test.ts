import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

vi.mock("node:fs")
vi.mock("node:fs/promises")

import * as fs from "node:fs"
import * as fsp from "node:fs/promises"

const mockExistsSync = vi.mocked(fs.existsSync)
const mockReadFile = vi.mocked(fsp.readFile)

const validConfig = {
	environments: {
		dev: {
			replacements: [{ files: ["a.env"], regex: "/FOO=.*/", value: "FOO=bar" }],
		},
	},
}

describe("loadKoseiConfig", () => {
	let exitSpy: ReturnType<typeof vi.spyOn>
	let errorSpy: ReturnType<typeof vi.spyOn>

	beforeEach(() => {
		vi.resetModules()
		exitSpy = vi
			.spyOn(process, "exit")
			.mockImplementation((() => {}) as () => never)
		errorSpy = vi.spyOn(console, "error").mockImplementation(() => {})
	})

	afterEach(() => {
		vi.restoreAllMocks()
	})

	const load = () => import("../src/config").then((m) => m.loadKoseiConfig)

	it("returns parsed config when config file is valid", async () => {
		mockExistsSync.mockImplementation((p) =>
			String(p).endsWith("kosei.config.json"),
		)
		mockReadFile.mockResolvedValue(JSON.stringify(validConfig) as never)
		const loadKoseiConfig = await load()
		const result = await loadKoseiConfig()
		expect(result).toEqual(validConfig)
	})

	it("walks up directories to find config file", async () => {
		const cwd = process.cwd()
		const parent = require("node:path").dirname(cwd)
		mockExistsSync.mockImplementation(
			(p) =>
				String(p) === require("node:path").join(parent, "kosei.config.json"),
		)
		mockReadFile.mockResolvedValue(JSON.stringify(validConfig) as never)
		const loadKoseiConfig = await load()
		await loadKoseiConfig()
		expect(mockReadFile).toHaveBeenCalledWith(
			require("node:path").join(parent, "kosei.config.json"),
			"utf-8",
		)
	})

	it("exits with 1 when config file is not found", async () => {
		mockExistsSync.mockReturnValue(false)
		exitSpy.mockImplementation(() => {
			throw new Error("process.exit")
		})
		const loadKoseiConfig = await load()
		await expect(loadKoseiConfig()).rejects.toThrow("process.exit")
		expect(exitSpy).toHaveBeenCalledWith(1)
		expect(errorSpy).toHaveBeenCalled()
	})

	it("exits with 1 when config file contains invalid JSON", async () => {
		mockExistsSync.mockImplementation((p) =>
			String(p).endsWith("kosei.config.json"),
		)
		mockReadFile.mockResolvedValue("not json {{{" as never)
		const loadKoseiConfig = await load()
		await loadKoseiConfig()
		expect(exitSpy).toHaveBeenCalledWith(1)
		expect(errorSpy).toHaveBeenCalled()
	})

	it("exits with 1 when config fails validation", async () => {
		mockExistsSync.mockImplementation((p) =>
			String(p).endsWith("kosei.config.json"),
		)
		mockReadFile.mockResolvedValue(
			JSON.stringify({ environments: null }) as never,
		)
		const loadKoseiConfig = await load()
		await loadKoseiConfig()
		expect(exitSpy).toHaveBeenCalledWith(1)
		expect(errorSpy).toHaveBeenCalled()
	})
})

describe("validateConfig (via loadKoseiConfig)", () => {
	let exitSpy: ReturnType<typeof vi.spyOn>

	beforeEach(() => {
		vi.resetModules()
		exitSpy = vi
			.spyOn(process, "exit")
			.mockImplementation((() => {}) as () => never)
		vi.spyOn(console, "error").mockImplementation(() => {})
		mockExistsSync.mockImplementation((p) =>
			String(p).endsWith("kosei.config.json"),
		)
	})

	afterEach(() => {
		vi.restoreAllMocks()
	})

	const attempt = async (raw: unknown) => {
		vi.resetModules()
		mockReadFile.mockResolvedValue(JSON.stringify(raw) as never)
		const { loadKoseiConfig } = await import("../src/config")
		await loadKoseiConfig()
	}

	it("rejects null", async () => {
		await attempt(null)
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects a string", async () => {
		await attempt("string")
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects an array", async () => {
		await attempt([])
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects missing environments", async () => {
		await attempt({})
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects environments as array", async () => {
		await attempt({ environments: [] })
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects environments as string", async () => {
		await attempt({ environments: "bad" })
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects an environment entry that is not an object", async () => {
		await attempt({ environments: { dev: "bad" } })
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects an environment entry that is an array", async () => {
		await attempt({ environments: { dev: [] } })
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects missing replacements", async () => {
		await attempt({ environments: { dev: {} } })
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects empty replacements array", async () => {
		await attempt({ environments: { dev: { replacements: [] } } })
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects replacements as non-array", async () => {
		await attempt({ environments: { dev: { replacements: "bad" } } })
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects a replacement that is not an object", async () => {
		await attempt({ environments: { dev: { replacements: ["bad"] } } })
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects a replacement with missing files", async () => {
		await attempt({
			environments: { dev: { replacements: [{ regex: "/x/", value: "y" }] } },
		})
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects a replacement with empty files array", async () => {
		await attempt({
			environments: {
				dev: { replacements: [{ files: [], regex: "/x/", value: "y" }] },
			},
		})
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects a replacement with non-string in files array", async () => {
		await attempt({
			environments: {
				dev: { replacements: [{ files: [1], regex: "/x/", value: "y" }] },
			},
		})
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects a replacement with missing regex", async () => {
		await attempt({
			environments: { dev: { replacements: [{ files: ["f"], value: "y" }] } },
		})
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects a replacement with regex not wrapped in slashes", async () => {
		await attempt({
			environments: {
				dev: {
					replacements: [{ files: ["f"], regex: "noSlashes", value: "y" }],
				},
			},
		})
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects a replacement with invalid regex flags", async () => {
		await attempt({
			environments: {
				dev: { replacements: [{ files: ["f"], regex: "/x/z", value: "y" }] },
			},
		})
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects a replacement with missing value", async () => {
		await attempt({
			environments: { dev: { replacements: [{ files: ["f"], regex: "/x/" }] } },
		})
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("rejects a replacement with non-string value", async () => {
		await attempt({
			environments: {
				dev: { replacements: [{ files: ["f"], regex: "/x/", value: 42 }] },
			},
		})
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("accepts valid config with optional description", async () => {
		vi.resetModules()
		mockReadFile.mockResolvedValue(
			JSON.stringify({
				environments: {
					dev: {
						description: "Development",
						replacements: [
							{ files: ["f.env"], regex: "/FOO=.*/g", value: "FOO=bar" },
						],
					},
				},
			}) as never,
		)
		const { loadKoseiConfig } = await import("../src/config")
		const result = await loadKoseiConfig()
		expect(result.environments.dev.description).toBe("Development")
		expect(exitSpy).not.toHaveBeenCalled()
	})

	it("accepts regex with multiple valid flags", async () => {
		vi.resetModules()
		mockReadFile.mockResolvedValue(
			JSON.stringify({
				environments: {
					prod: {
						replacements: [
							{ files: ["f"], regex: "/pattern/gimsuy", value: "x" },
						],
					},
				},
			}) as never,
		)
		const { loadKoseiConfig } = await import("../src/config")
		await loadKoseiConfig()
		expect(exitSpy).not.toHaveBeenCalled()
	})
})
