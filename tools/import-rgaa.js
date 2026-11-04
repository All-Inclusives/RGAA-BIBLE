#!/usr/bin/env node
"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const ROOT = path.resolve(__dirname, "..");
const SOURCE_REPOSITORY = "https://github.com/DISIC/accessibilite.numerique.gouv.fr";
const RGAA_VERSION = "4.1.2";
const SCHEMA_VERSION = "1.0.0";
const SOURCE_FILES = [
	"RGAA/criteres.json",
	"RGAA/glossaire.json",
	"RGAA/methodologies.json",
];

function fail(message) {
	console.error(`Import RGAA impossible : ${message}`);
	process.exitCode = 1;
}

function readJson(filePath) {
	try {
		return JSON.parse(fs.readFileSync(filePath, "utf8"));
	} catch (error) {
		throw new Error(`lecture ou JSON invalide pour ${filePath}: ${error.message}`);
	}
}

function assert(condition, message) {
	if (!condition) {
		throw new Error(message);
	}
}

function getSourceCommit(sourceRoot) {
	try {
		return execFileSync("git", ["-C", sourceRoot, "rev-parse", "HEAD"], {
			encoding: "utf8",
			stdio: ["ignore", "pipe", "pipe"],
		}).trim();
	} catch (error) {
		throw new Error(
			`impossible de lire la révision Git de ${sourceRoot}: ${error.message}`,
		);
	}
}

function sha256(filePath) {
	return crypto
		.createHash("sha256")
		.update(fs.readFileSync(filePath))
		.digest("hex");
}

function validateSource(criteriaData, glossaryData, methodologyData) {
	assert(
		criteriaData && Array.isArray(criteriaData.topics),
		"criteres.json doit contenir un tableau topics",
	);
	assert(
		criteriaData.wcag && typeof criteriaData.wcag.version === "number",
		"la version WCAG est absente ou invalide",
	);
	assert(
		glossaryData && Array.isArray(glossaryData.glossary),
		"glossaire.json doit contenir un tableau glossary",
	);
	assert(
		methodologyData &&
			typeof methodologyData === "object" &&
			!Array.isArray(methodologyData),
		"methodologies.json doit contenir un objet indexé par test",
	);
	assert(
		Object.values(methodologyData).every((value) => typeof value === "string"),
		"les méthodologies doivent être des chaînes de caractères",
	);

	const criterionIds = new Set();
	const testIds = new Set();
	for (const topic of criteriaData.topics) {
		assert(
			Number.isInteger(topic.number) &&
				typeof topic.topic === "string" &&
				Array.isArray(topic.criteria),
			"un thème RGAA n’a pas le format attendu",
		);
		for (const entry of topic.criteria) {
			const criterion = entry && entry.criterium;
			assert(
				criterion &&
					Number.isInteger(criterion.number) &&
					typeof criterion.title === "string" &&
					criterion.tests &&
					typeof criterion.tests === "object" &&
					Array.isArray(criterion.references) &&
					criterion.references.every(
						(reference) =>
							reference && typeof reference === "object" && !Array.isArray(reference),
					),
				`critère invalide dans le thème ${topic.number}`,
			);
			const criterionId = `${topic.number}.${criterion.number}`;
			assert(!criterionIds.has(criterionId), `critère dupliqué : ${criterionId}`);
			criterionIds.add(criterionId);
			for (const testNumber of Object.keys(criterion.tests)) {
				const test = criterion.tests[testNumber];
				assert(
					typeof test === "string" ||
						(Array.isArray(test) &&
							test.every((line) => typeof line === "string")),
					`contenu de test invalide : ${criterionId}.${testNumber}`,
				);
				const testId = `${criterionId}.${testNumber}`;
				assert(!testIds.has(testId), `test dupliqué : ${testId}`);
				testIds.add(testId);
			}
		}
	}

	const methodologyIds = new Set(Object.keys(methodologyData));
	for (const testId of testIds) {
		assert(methodologyIds.has(testId), `méthodologie manquante pour ${testId}`);
	}
	for (const testId of methodologyIds) {
		assert(testIds.has(testId), `méthodologie sans test correspondant : ${testId}`);
	}
	for (const definition of glossaryData.glossary) {
		assert(
			definition &&
				typeof definition.title === "string" &&
				typeof definition.body === "string",
			"une définition du glossaire est invalide",
		);
	}

	return { criteria: criterionIds.size, tests: testIds.size };
}

function validateDataset(dataset) {
	assert(dataset.schemaVersion === SCHEMA_VERSION, "version de schéma invalide");
	assert(
		dataset.source &&
			dataset.source.rgaaVersion === RGAA_VERSION &&
			/^[a-f0-9]{40}$/i.test(dataset.source.commit),
		"provenance ou version RGAA invalide",
	);
	assert(Array.isArray(dataset.topics), "thèmes importés invalides");

	const criterionIds = new Set();
	const testIds = new Set();
	for (const topic of dataset.topics) {
		assert(
			typeof topic.id === "string" &&
				typeof topic.title === "string" &&
				Array.isArray(topic.criteria),
			"thème importé invalide",
		);
		for (const criterion of topic.criteria) {
			assert(
				criterion.id === `${topic.id}.${criterion.official.number}` &&
					typeof criterion.official.title === "string" &&
					criterion.official.tests &&
					Array.isArray(criterion.official.references),
				`critère importé invalide : ${criterion.id}`,
			);
			assert(!criterionIds.has(criterion.id), `critère dupliqué : ${criterion.id}`);
			criterionIds.add(criterion.id);
			assert(
				criterion.editorial &&
					criterion.editorial.fr &&
					Array.isArray(criterion.editorial.tags.components) &&
					Array.isArray(criterion.editorial.tags.disabilitySituations) &&
					Array.isArray(criterion.editorial.examples),
				`champs éditoriaux invalides : ${criterion.id}`,
			);
			for (const testNumber of Object.keys(criterion.official.tests)) {
				testIds.add(`${criterion.id}.${testNumber}`);
			}
		}
	}

	assert(
		dataset.glossary.every(
			(entry) =>
				typeof entry.title === "string" && typeof entry.bodyHtml === "string",
		),
		"glossaire importé invalide",
	);
	const methodologyIds = Object.keys(dataset.methodologies);
	assert(
		methodologyIds.length === testIds.size &&
			methodologyIds.every((id) => testIds.has(id)) &&
			Object.values(dataset.methodologies).every(
				(value) => typeof value === "string",
			),
		"les méthodologies importées ne correspondent pas aux tests",
	);
	assert(
		dataset.counts.topics === dataset.topics.length &&
			dataset.counts.criteria === criterionIds.size &&
			dataset.counts.tests === testIds.size &&
			dataset.counts.glossaryEntries === dataset.glossary.length &&
			dataset.counts.methodologies === methodologyIds.length,
		"les compteurs du corpus ne correspondent pas aux données",
	);
}

function buildDataset(sourceRoot) {
	const readSource = (relativePath) =>
		readJson(path.join(sourceRoot, ...relativePath.split("/")));
	const criteriaData = readSource(SOURCE_FILES[0]);
	const glossaryData = readSource(SOURCE_FILES[1]);
	const methodologyData = readSource(SOURCE_FILES[2]);
	const counts = validateSource(criteriaData, glossaryData, methodologyData);
	const readmePath = path.join(sourceRoot, "README.md");
	assert(fs.existsSync(readmePath), "README.md source introuvable");
	assert(
		fs.readFileSync(readmePath, "utf8").includes(RGAA_VERSION),
		`README source ne confirme pas la version RGAA ${RGAA_VERSION}`,
	);

	const commit = getSourceCommit(sourceRoot);
	assert(/^[a-f0-9]{40}$/i.test(commit), "révision Git source invalide");

	const topics = criteriaData.topics.map((topic) => ({
		id: String(topic.number),
		title: topic.topic,
		criteria: topic.criteria.map(({ criterium }) => ({
			id: `${topic.number}.${criterium.number}`,
			official: criterium,
			editorial: {
				fr: { explanation: null, userImpact: null },
				tags: { components: [], disabilitySituations: [] },
				examples: [],
			},
		})),
	}));

	return {
		schemaVersion: SCHEMA_VERSION,
		source: {
			repository: SOURCE_REPOSITORY,
			commit,
			rgaaVersion: RGAA_VERSION,
			license: "Licence Ouverte 2.0 (Etalab), sous réserve des droits de tiers",
			files: SOURCE_FILES.map((relativePath) => ({
				path: relativePath,
				sha256: sha256(
					path.join(sourceRoot, ...relativePath.split("/")),
				),
			})),
		},
		wcagVersion: criteriaData.wcag.version,
		topics,
		glossary: glossaryData.glossary.map(({ title, body }) => ({
			title,
			bodyHtml: body,
		})),
		methodologies: methodologyData,
		counts: {
			topics: topics.length,
			criteria: counts.criteria,
			tests: counts.tests,
			glossaryEntries: glossaryData.glossary.length,
			methodologies: Object.keys(methodologyData).length,
		},
	};
}

function main() {
	const sourceRoot = path.resolve(
		process.argv[2] || path.join(ROOT, "accessibilite.numerique.gouv.fr"),
	);
	const outputPath = path.join(ROOT, "data", "rgaa-4.1.2.json");

	try {
		const dataset = buildDataset(sourceRoot);
		validateDataset(dataset);
		fs.mkdirSync(path.dirname(outputPath), { recursive: true });
		fs.writeFileSync(outputPath, `${JSON.stringify(dataset, null, 2)}\n`, "utf8");
		console.log(
			`Corpus RGAA ${RGAA_VERSION} importé : ${dataset.counts.criteria} critères, ` +
				`${dataset.counts.tests} tests, ${dataset.counts.glossaryEntries} définitions. ` +
				`Révision source ${dataset.source.commit}.`,
		);
	} catch (error) {
		fail(error.message);
	}
}

main();
