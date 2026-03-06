import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

vi.mock("../src/commands/switch")

import { switchCommand } from "../src/commands/switch"

const mockSwitchCommand = vi.mocked(switchCommand)

describe("cli entry point", () => {
	let exitSpy: ReturnType<typeof vi.spyOn>
	let logSpy: ReturnType<typeof vi.spyOn>
	let errorSpy: ReturnType<typeof vi.spyOn>

	beforeEach(() => {
		vi.resetModules()
		exitSpy = vi
			.spyOn(process, "exit")
			.mockImplementation((() => {}) as () => never)
		logSpy = vi.spyOn(console, "log").mockImplementation(() => {})
		errorSpy = vi.spyOn(console, "error").mockImplementation(() => {})
		mockSwitchCommand.mockResolvedValue(undefined)
	})

	afterEach(() => {
		vi.restoreAllMocks()
	})

	it("calls switchCommand with remaining args when command is 'switch'", async () => {
		vi.resetModules()
		vi.spyOn(process, "argv", "get").mockReturnValue([
			"node",
			"cli.js",
			"switch",
			"dev",
		])
		await import("../src/cli")
		expect(mockSwitchCommand).toHaveBeenCalledWith(["dev"])
	})

	it("passes multiple args to switchCommand", async () => {
		vi.resetModules()
		vi.spyOn(process, "argv", "get").mockReturnValue([
			"node",
			"cli.js",
			"switch",
			"dev",
			"--dry-run",
		])
		await import("../src/cli")
		expect(mockSwitchCommand).toHaveBeenCalledWith(["dev", "--dry-run"])
	})

	it("logs version for --version flag", async () => {
		vi.resetModules()
		vi.spyOn(process, "argv", "get").mockReturnValue([
			"node",
			"cli.js",
			"--version",
		])
		await import("../src/cli")
		expect(logSpy).toHaveBeenCalledWith(expect.stringMatching(/^\d+\.\d+\.\d+/))
	})

	it("logs version for -v flag", async () => {
		vi.resetModules()
		vi.spyOn(process, "argv", "get").mockReturnValue(["node", "cli.js", "-v"])
		await import("../src/cli")
		expect(logSpy).toHaveBeenCalledWith(expect.stringMatching(/^\d+\.\d+\.\d+/))
	})

	it("logs usage error and exits for unknown command", async () => {
		vi.resetModules()
		vi.spyOn(process, "argv", "get").mockReturnValue([
			"node",
			"cli.js",
			"unknown",
		])
		await import("../src/cli")
		expect(errorSpy).toHaveBeenCalledWith(expect.stringContaining("Usage"))
		expect(exitSpy).toHaveBeenCalledWith(1)
	})

	it("logs usage error and exits when no command provided", async () => {
		vi.resetModules()
		vi.spyOn(process, "argv", "get").mockReturnValue(["node", "cli.js"])
		await import("../src/cli")
		expect(errorSpy).toHaveBeenCalledWith(expect.stringContaining("Usage"))
		expect(exitSpy).toHaveBeenCalledWith(1)
	})
})
