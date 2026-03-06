import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

const ESC = "\x1b["

describe("format", () => {
	beforeEach(() => {
		vi.resetModules()
		vi.unstubAllEnvs()
	})

	afterEach(() => {
		vi.restoreAllMocks()
	})

	const withTTY = (value: boolean | undefined) => {
		Object.defineProperty(process.stdout, "isTTY", {
			value,
			configurable: true,
			writable: true,
		})
	}

	const load = () => import("../src/format")

	describe("when colors are enabled (TTY, no NO_COLOR)", () => {
		beforeEach(() => {
			vi.stubEnv("NO_COLOR", undefined as unknown as string)
			withTTY(true)
		})

		it("wraps red correctly", async () => {
			const { red } = await load()
			expect(red("text")).toBe(`${ESC}31mtext${ESC}39m`)
		})

		it("wraps green correctly", async () => {
			const { green } = await load()
			expect(green("text")).toBe(`${ESC}32mtext${ESC}39m`)
		})

		it("wraps yellow correctly", async () => {
			const { yellow } = await load()
			expect(yellow("text")).toBe(`${ESC}33mtext${ESC}39m`)
		})

		it("wraps bold correctly", async () => {
			const { bold } = await load()
			expect(bold("text")).toBe(`${ESC}1mtext${ESC}22m`)
		})

		it("wraps dim correctly", async () => {
			const { dim } = await load()
			expect(dim("text")).toBe(`${ESC}2mtext${ESC}22m`)
		})
	})

	describe("when NO_COLOR is set", () => {
		beforeEach(() => {
			vi.stubEnv("NO_COLOR", "1")
			withTTY(true)
		})

		it("returns unstyled string for red", async () => {
			const { red } = await load()
			expect(red("text")).toBe("text")
		})

		it("returns unstyled string for bold", async () => {
			const { bold } = await load()
			expect(bold("text")).toBe("text")
		})
	})

	describe("when stdout is not a TTY", () => {
		beforeEach(() => {
			vi.stubEnv("NO_COLOR", undefined as unknown as string)
			withTTY(false)
		})

		it("returns unstyled string for red", async () => {
			const { red } = await load()
			expect(red("text")).toBe("text")
		})

		it("returns unstyled string for dim", async () => {
			const { dim } = await load()
			expect(dim("text")).toBe("text")
		})
	})
})
