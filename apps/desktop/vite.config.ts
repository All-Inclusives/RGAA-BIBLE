import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

const projectRoot = fileURLToPath(new URL(".", import.meta.url));
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
	clearScreen: false,
	publicDir: path.resolve(projectRoot, "../../data"),
	server: {
		port: 5173,
		strictPort: true,
		host: host || false,
		fs: {
			allow: [projectRoot, path.resolve(projectRoot, "../..")],
		},
		hmr: host
			? {
					protocol: "ws",
					host,
					port: 5174,
				}
			: undefined,
		watch: {
			ignored: ["**/src-tauri/**"],
		},
	},
});
