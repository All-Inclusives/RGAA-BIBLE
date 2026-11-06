#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");

const ROOT = path.resolve(__dirname, "..");
const APP_CONFIG = path.join(
	ROOT,
	"apps",
	"desktop",
	"src-tauri",
	"tauri.conf.json",
);
const APP_BUNDLE = path.resolve(
	process.env.APP_BUNDLE_DIR ||
		path.join(
			ROOT,
			"apps",
			"desktop",
			"src-tauri",
			"target",
			"release",
			"bundle",
		),
);
const NSIS_BUNDLE = path.join(APP_BUNDLE, "nsis");

function assert(condition, message) {
	if (!condition) throw new Error(message);
}

function main() {
	try {
		const repository = process.env.GITHUB_REPOSITORY;
		const releaseTag = process.env.APP_RELEASE_TAG;
		assert(
			repository === "All-Inclusives/RGAA-BIBLE",
			"GITHUB_REPOSITORY must identify All-Inclusives/RGAA-BIBLE",
		);

		const { version } = JSON.parse(fs.readFileSync(APP_CONFIG, "utf8"));
		assert(
			releaseTag === `accessiguide-v${version}`,
			`release tag must be accessiguide-v${version}`,
		);

		const installers = fs
			.readdirSync(NSIS_BUNDLE)
			.filter((name) => name.toLowerCase().endsWith("-setup.exe"));
		assert(installers.length === 1, "expected exactly one NSIS setup executable");

		const installer = installers[0];
		const signaturePath = path.join(NSIS_BUNDLE, `${installer}.sig`);
		assert(fs.existsSync(signaturePath), `signed installer signature is missing: ${installer}.sig`);
		const signature = fs.readFileSync(signaturePath, "utf8").trim();
		assert(signature.length > 0, "installer signature is empty");

		const feed = {
			version,
			notes: `AccessiGuide ${version}`,
			pub_date: new Date().toISOString(),
			platforms: {
				"windows-x86_64": {
					signature,
					url: `https://github.com/${repository}/releases/download/${releaseTag}/${encodeURIComponent(installer)}`,
				},
			},
		};
		const outputPath = path.join(APP_BUNDLE, "latest.json");
		fs.writeFileSync(outputPath, `${JSON.stringify(feed, null, 2)}\n`, "utf8");
		console.log(`Updater feed written for AccessiGuide ${version}: ${outputPath}`);
	} catch (error) {
		console.error(`Unable to create desktop updater feed: ${error.message}`);
		process.exitCode = 1;
	}
}

main();
