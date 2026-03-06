import packageJson from "../package.json"
import { switchCommand } from "./commands/switch"

const [, , command, ...args] = process.argv

if (command === "switch") {
	switchCommand(args)
} else if (command === "--version" || command === "-v") {
	console.log(packageJson.version)
} else {
	console.error("Usage: kosei switch <env> [--dry-run]")
	process.exit(1)
}
