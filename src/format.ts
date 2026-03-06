const ESC = "\x1b["
const enabled = process.env.NO_COLOR == null && process.stdout.isTTY !== false
const wrap = (open: string, close: string) => (s: string) =>
	enabled ? `${ESC}${open}m${s}${ESC}${close}m` : s

export const red = wrap("31", "39")
export const green = wrap("32", "39")
export const yellow = wrap("33", "39")
export const bold = wrap("1", "22")
export const dim = wrap("2", "22")
