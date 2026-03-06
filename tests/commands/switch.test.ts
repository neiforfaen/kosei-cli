import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

vi.mock("node:fs")
vi.mock("node:fs/promises")
vi.mock("../../src/config")

import * as fs from "node:fs"
import * as fsp from "node:fs/promises"
import * as configModule from "../../src/config"

const mockExistsSync = vi.mocked(fs.existsSync)
const mockReadFileSync = vi.mocked(fs.readFileSync)
const mockReadFile = vi.mocked(fsp.readFile)
const mockWriteFile = vi.mocked(fsp.writeFile)
const mockLoadKoseiConfig = vi.mocked(configModule.loadKoseiConfig)

import { switchCommand } from "../../src/commands/switch"
import type { Config } from "../../src/config"

const makeConfig = (
	replacementOverrides: Partial<
		Config["environments"]["env"]["replacements"][0]
	> = {},
): Config => ({
	environments: {
		dev: {
			replacements: [
				{
					files: ["app.env"],
					regex: "/FOO=.*/",
					value: "FOO=bar",
					...replacementOverrides,
				},
			],
		},
	},
})

describe("switchCommand", () => {
	let exitSpy: ReturnType<typeof vi.spyOn>
	let errorSpy: ReturnType<typeof vi.spyOn>
	let warnSpy: ReturnType<typeof vi.spyOn>

	beforeEach(() => {
		vi.clearAllMocks()
		exitSpy = vi
			.spyOn(process, "exit")
			.mockImplementation((() => {}) as () => never)
		errorSpy = vi.spyOn(console, "error").mockImplementation(() => {})
		warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {})
		vi.spyOn(console, "log").mockImplementation(() => {})
	})

	afterEach(() => {
		vi.restoreAllMocks()
	})

	it("exits with 1 when no env argument is provided", async () => {
		await switchCommand([])
		expect(exitSpy).toHaveBeenCalledWith(1)
		expect(errorSpy).toHaveBeenCalled()
	})

	it("exits with 1 when only flags are provided", async () => {
		await switchCommand(["--dry-run"])
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("exits with 1 when loadKoseiConfig throws", async () => {
		mockLoadKoseiConfig.mockRejectedValue(new Error("read failed"))
		await switchCommand(["dev"])
		expect(exitSpy).toHaveBeenCalledWith(1)
		expect(errorSpy).toHaveBeenCalledWith(
			expect.stringContaining("read failed"),
		)
	})

	it("exits with 1 when env is not in config", async () => {
		mockLoadKoseiConfig.mockResolvedValue(makeConfig())
		mockExistsSync.mockReturnValue(true)
		await switchCommand(["prod"])
		expect(exitSpy).toHaveBeenCalledWith(1)
		expect(errorSpy).toHaveBeenCalledWith(expect.stringContaining("prod"))
	})

	it("lists available environments in unknown env error", async () => {
		mockLoadKoseiConfig.mockResolvedValue(makeConfig())
		await switchCommand(["staging"])
		expect(errorSpy).toHaveBeenCalledWith(expect.stringContaining("dev"))
	})

	it("extracts env from first non-flag argument", async () => {
		mockLoadKoseiConfig.mockResolvedValue(makeConfig())
		mockExistsSync.mockReturnValue(true)
		mockReadFile.mockResolvedValue("FOO=old" as never)
		mockWriteFile.mockResolvedValue(undefined as never)
		await switchCommand(["dev"])
		expect(mockLoadKoseiConfig).toHaveBeenCalled()
	})

	it("warns and exits 0 when a file is missing", async () => {
		mockLoadKoseiConfig.mockResolvedValue(makeConfig())
		mockExistsSync.mockReturnValue(false)
		await switchCommand(["dev"])
		expect(warnSpy).toHaveBeenCalledWith(expect.stringContaining("app.env"))
		expect(exitSpy).toHaveBeenCalledWith(0)
	})
})

describe("applyReplacement (via switchCommand)", () => {
	beforeEach(() => {
		vi.clearAllMocks()
		vi.spyOn(process, "exit").mockImplementation((() => {}) as () => never)
		vi.spyOn(console, "error").mockImplementation(() => {})
		vi.spyOn(console, "warn").mockImplementation(() => {})
		vi.spyOn(console, "log").mockImplementation(() => {})
		mockExistsSync.mockReturnValue(true)
		mockWriteFile.mockResolvedValue(undefined as never)
	})

	afterEach(() => {
		vi.restoreAllMocks()
	})

	it("writes updated content when regex matches", async () => {
		mockLoadKoseiConfig.mockResolvedValue(makeConfig())
		mockReadFile.mockResolvedValue("FOO=old\nBAR=1" as never)

		await switchCommand(["dev"])

		expect(mockWriteFile).toHaveBeenCalledWith(
			expect.any(String),
			"FOO=bar\nBAR=1",
			"utf8",
		)
	})

	it("does not write when content is unchanged", async () => {
		mockLoadKoseiConfig.mockResolvedValue(makeConfig())
		mockReadFile.mockResolvedValue("NO_MATCH=1" as never)

		await switchCommand(["dev"])

		expect(mockWriteFile).not.toHaveBeenCalled()
	})

	it("applies replacement across multiple files", async () => {
		const config: Config = {
			environments: {
				dev: {
					replacements: [
						{ files: ["a.env", "b.env"], regex: "/FOO=.*/", value: "FOO=bar" },
					],
				},
			},
		}
		mockLoadKoseiConfig.mockResolvedValue(config)
		mockReadFile.mockResolvedValue("FOO=old" as never)

		await switchCommand(["dev"])

		expect(mockWriteFile).toHaveBeenCalledTimes(2)
	})

	it("applies multiple replacements in sequence", async () => {
		const config: Config = {
			environments: {
				dev: {
					replacements: [
						{ files: ["a.env"], regex: "/FOO=.*/", value: "FOO=1" },
						{ files: ["b.env"], regex: "/BAR=.*/", value: "BAR=2" },
					],
				},
			},
		}
		mockLoadKoseiConfig.mockResolvedValue(config)
		mockReadFile.mockResolvedValue("FOO=old\nBAR=old" as never)

		await switchCommand(["dev"])

		expect(mockWriteFile).toHaveBeenCalledTimes(2)
	})
})

describe("previewReplacement (via --dry-run)", () => {
	let logSpy: ReturnType<typeof vi.spyOn>

	beforeEach(() => {
		vi.clearAllMocks()
		vi.spyOn(process, "exit").mockImplementation((() => {}) as () => never)
		vi.spyOn(console, "error").mockImplementation(() => {})
		vi.spyOn(console, "warn").mockImplementation(() => {})
		logSpy = vi.spyOn(console, "log").mockImplementation(() => {})
		mockExistsSync.mockReturnValue(true)
	})

	afterEach(() => {
		vi.restoreAllMocks()
	})

	it("does not write files in dry-run mode", async () => {
		mockLoadKoseiConfig.mockResolvedValue(makeConfig())
		mockReadFileSync.mockReturnValue("FOO=old" as never)

		await switchCommand(["dev", "--dry-run"])

		expect(mockWriteFile).not.toHaveBeenCalled()
	})

	it("logs dim message when no changes", async () => {
		mockLoadKoseiConfig.mockResolvedValue(makeConfig())
		mockReadFileSync.mockReturnValue("NO_MATCH=1" as never)

		await switchCommand(["dev", "--dry-run"])

		expect(logSpy).toHaveBeenCalledWith(expect.stringContaining("no changes"))
	})

	it("shows diff lines when content changes", async () => {
		mockLoadKoseiConfig.mockResolvedValue(makeConfig())
		mockReadFileSync.mockReturnValue("FOO=old" as never)

		await switchCommand(["dev", "--dry-run"])

		const allOutput = logSpy.mock.calls.map((c) => c[0]).join("\n")
		expect(allOutput).toContain("FOO=old")
		expect(allOutput).toContain("FOO=bar")
	})

	it("shows added lines (updated is longer than original)", async () => {
		const config: Config = {
			environments: {
				dev: {
					replacements: [{ files: ["a.env"], regex: "/X/", value: "X\nY" }],
				},
			},
		}
		mockLoadKoseiConfig.mockResolvedValue(config)
		mockReadFileSync.mockReturnValue("X" as never)

		await switchCommand(["dev", "--dry-run"])

		const allOutput = logSpy.mock.calls.map((c) => c[0]).join("\n")
		expect(allOutput).toContain("Y")
	})

	it("shows removed lines (original is longer than updated)", async () => {
		const config: Config = {
			environments: {
				dev: {
					replacements: [{ files: ["a.env"], regex: "/X\nY/", value: "X" }],
				},
			},
		}
		mockLoadKoseiConfig.mockResolvedValue(config)
		mockReadFileSync.mockReturnValue("X\nY" as never)

		await switchCommand(["dev", "--dry-run"])

		const allOutput = logSpy.mock.calls.map((c) => c[0]).join("\n")
		expect(allOutput).toContain("Y")
	})

	it("accepts --dry-run before the env argument", async () => {
		mockLoadKoseiConfig.mockResolvedValue(makeConfig())
		mockReadFileSync.mockReturnValue("NO_MATCH=1" as never)

		await switchCommand(["--dry-run", "dev"])

		expect(mockWriteFile).not.toHaveBeenCalled()
		expect(vi.mocked(process.exit)).not.toHaveBeenCalledWith(1)
	})
})
