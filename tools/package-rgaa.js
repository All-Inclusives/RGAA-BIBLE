#!/usr/bin/env node
"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");

const ROOT = path.resolve(__dirname, "..");
const DATASET_PATH = path.join(ROOT, "data", "rgaa-4.1.2.json");
const UPDATE_MANIFEST_SCHEMA_VERSION = "1.0.0";
const SUPPORTED_DATASET_SCHEMA_VERSIONS = new Set(["1.0.0", "1.1.0"]);

function signingPayload(manifest) {
	return [
		"AccessiGuide RGAA update manifest v1",
		`schemaVersion:${manifest.schemaVersion}`,
		`sequence:${manifest.sequence}`,
		`rgaaVersion:${manifest.rgaaVersion}`,
		`sourceCommit:${manifest.sourceCommit}`,
		`packageFile:${manifest.packageFile}`,
		`packageSha256:${manifest.packageSha256}`,
		`packageBytes:${manifest.packageBytes}`,
		"",
	].join("\n");
}

function createManifest(dataset, packageBytes, sequence, privateKeyPem) {
	const manifest = {
		schemaVersion: UPDATE_MANIFEST_SCHEMA_VERSION,
		sequence,
		rgaaVersion: dataset.source.rgaaVersion,
		sourceCommit: dataset.source.commit,
		packageFile: `rgaa-${dataset.source.rgaaVersion}.json`,
		packageSha256: crypto
			.createHash("sha256")
			.update(packageBytes)
			.digest("hex"),
		packageBytes: packageBytes.length,
	};
	if (privateKeyPem) {
		manifest.signature = crypto
			.sign(
				null,
				Buffer.from(signingPayload(manifest), "utf8"),
				privateKeyPem,
			)
			.toString("hex");
	}
	return manifest;
}

function fail(message) {
	console.error(`Packaging RGAA impossible : ${message}`);
	process.exitCode = 1;
}

function main() {
	const sequence = Number(process.argv[2]);
	const requireSignature = process.argv.includes("--require-signature");
	if (!Number.isSafeInteger(sequence) || sequence < 1) {
		fail("fournir un numéro de séquence entier strictement positif");
		return;
	}

	try {
		const packageBytes = fs.readFileSync(DATASET_PATH);
		const dataset = JSON.parse(packageBytes.toString("utf8"));
		if (
			!SUPPORTED_DATASET_SCHEMA_VERSIONS.has(dataset.schemaVersion) ||
			dataset.source.rgaaVersion !== "4.1.2" ||
			!/^[a-f0-9]{40}$/i.test(dataset.source.commit)
		) {
			throw new Error("schéma, version RGAA ou provenance du corpus invalide");
		}
		const privateKeyPem = process.env.A11Y_UPDATE_PRIVATE_KEY;
		if (requireSignature && !privateKeyPem) {
			throw new Error(
				"--require-signature exige le secret A11Y_UPDATE_PRIVATE_KEY",
			);
		}

		const outputDirectory = path.join(
			ROOT,
			"releases",
			`rgaa-${dataset.source.rgaaVersion}`,
			String(sequence),
		);
		if (fs.existsSync(outputDirectory)) {
			throw new Error(`le dossier de sortie existe déjà : ${outputDirectory}`);
		}
		const stagingDirectory = `${outputDirectory}.staging-${process.pid}`;
		fs.mkdirSync(path.dirname(outputDirectory), { recursive: true });
		const manifest = createManifest(
			dataset,
			packageBytes,
			sequence,
			privateKeyPem,
		);

		try {
			fs.mkdirSync(stagingDirectory, { recursive: false });
			fs.writeFileSync(
				path.join(stagingDirectory, manifest.packageFile),
				packageBytes,
				{ flag: "wx" },
			);
			fs.writeFileSync(
				path.join(stagingDirectory, "manifest.json"),
				`${JSON.stringify(manifest, null, 2)}\n`,
				{ flag: "wx" },
			);
			fs.renameSync(stagingDirectory, outputDirectory);
		} catch (error) {
			if (fs.existsSync(stagingDirectory)) {
				fs.rmSync(stagingDirectory, { recursive: true });
			}
			throw error;
		}
		console.log(
			`Paquet RGAA ${manifest.rgaaVersion} séquence ${sequence} créé : ${outputDirectory}`,
		);
		if (!manifest.signature) {
			console.warn("AVERTISSEMENT : manifeste non signé, réservé aux tests locaux.");
		}
	} catch (error) {
		fail(error.message);
	}
}

if (require.main === module) {
	main();
}

module.exports = { createManifest, signingPayload };
