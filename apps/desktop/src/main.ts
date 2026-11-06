import { invoke, isTauri } from "@tauri-apps/api/core";
import authorAvatarUrl from "../../../assets/images/Avatar_BE.jpg";
import brandLogoUrl from "../../../assets/images/logo_All-Inclusive.png";
import { initializeDisplayPreferences } from "./displayPreferences";
import "./styles.css";

document.querySelector<HTMLImageElement>("#brand-logo")!.src = brandLogoUrl;
document.querySelector<HTMLImageElement>("#author-avatar")!.src = authorAvatarUrl;

type Topic = {
	id: string;
	title: string;
	criteria: Criterion[];
};

type Criterion = {
	id: string;
	official: {
		number: number;
		title: string;
		tests: Record<string, string | string[]>;
		references: Record<string, unknown>[];
		[key: string]: unknown;
	};
	editorial: {
		fr: { explanation: string | null; userImpact: string | null };
		tags: { components: string[]; disabilitySituations: string[] };
		examples: { language: string; bad: string; good: string }[];
		pedagogicalTests: PedagogicalTest[];
	};
};

type PedagogicalTest = {
	testId: string;
	status: "draft" | "inReview" | "validated";
	plainLanguageDraft: string;
	userBenefit: string;
	cases: PedagogicalCase[];
};

type PedagogicalCase = {
	id: string;
	title: string;
	context: string;
	explanation: string;
	visualEffect?: "blur";
	resources: PedagogicalResource[];
	code: { language: string; snippet: string };
};

type PedagogicalResource = {
	id: string;
	file: string;
	altText: string | null;
	source: string;
	license: string;
};

type SearchResult = {
	topicId: string;
	topicTitle: string;
	criterion: Criterion;
};

type Dataset = {
	topics: Topic[];
	methodologies: Record<string, string>;
	counts: { criteria: number; tests: number };
};

type DatasetInfo = Pick<Dataset, "methodologies" | "counts">;

type UpdateCheckResult = {
	updated: boolean;
	sequence: number | null;
	rgaaVersion: string | null;
};

const elements = {
	form: document.querySelector<HTMLFormElement>("#search-form")!,
	search: document.querySelector<HTMLInputElement>("#search-input")!,
	topic: document.querySelector<HTMLSelectElement>("#topic-filter")!,
	sort: document.querySelector<HTMLSelectElement>("#sort-order")!,
	results: document.querySelector<HTMLDivElement>("#results-list")!,
	count: document.querySelector<HTMLParagraphElement>("#result-count")!,
	empty: document.querySelector<HTMLDivElement>("#empty-state")!,
	more: document.querySelector<HTMLButtonElement>("#load-more")!,
	dialog: document.querySelector<HTMLDialogElement>("#criterion-dialog")!,
	dialogTitle: document.querySelector<HTMLHeadingElement>("#dialog-title")!,
	dialogEyebrow: document.querySelector<HTMLParagraphElement>("#dialog-eyebrow")!,
	dialogContent: document.querySelector<HTMLDivElement>("#dialog-content")!,
	announcement: document.querySelector<HTMLDivElement>("#announcement")!,
	checkUpdates: document.querySelector<HTMLButtonElement>("#check-updates")!,
	updateStatus: document.querySelector<HTMLParagraphElement>("#dataset-update-status")!,
};

const runtimeIsTauri = isTauri();
let dataset: Dataset | undefined;
let allResults: SearchResult[] = [];
let visibleCount = 10;
let searchTimer: number | undefined;

function element<K extends keyof HTMLElementTagNameMap>(
	tag: K,
	className?: string,
	text?: string,
): HTMLElementTagNameMap[K] {
	const node = document.createElement(tag);
	if (className) node.className = className;
	if (text !== undefined) node.textContent = text;
	return node;
}

function normalize(value: string): string {
	return value
		.normalize("NFD")
		.replace(/\p{Diacritic}/gu, "")
		.toLocaleLowerCase("fr");
}

function plainText(value: string): string {
	return value
		.replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
		.replace(/`([^`]+)`/g, "$1")
		.replace(/<[^>]*>/g, "")
		.trim();
}

function searchableText(value: unknown): string {
	if (typeof value === "string") return value;
	if (Array.isArray(value)) return value.map(searchableText).join(" ");
	if (value && typeof value === "object") {
		return Object.values(value).map(searchableText).join(" ");
	}
	return "";
}

async function loadDataset(): Promise<Dataset> {
	if (runtimeIsTauri) {
		const [topics, info] = await Promise.all([
			invoke<Topic[]>("list_topics"),
			invoke<DatasetInfo>("get_dataset_info"),
		]);
		return { topics, ...info };
	}

	const response = await fetch("/rgaa-4.1.2.json");
	if (!response.ok) {
		throw new Error(`Impossible de charger le corpus RGAA (${response.status}).`);
	}
	return (await response.json()) as Dataset;
}

async function search(criteria: { text: string; topicId: string }): Promise<SearchResult[]> {
	if (runtimeIsTauri) {
		return invoke<SearchResult[]>("query_articles", {
			text: criteria.text || null,
			topicId: criteria.topicId || null,
			limit: dataset?.counts.criteria ?? 106,
		});
	}

	const localDataset = dataset ?? (await loadDataset());
	const tokens = normalize(criteria.text).trim().split(/\s+/).filter(Boolean);
	const matches: { result: SearchResult; rank: number; order: number }[] = [];
	let sourceOrder = 0;

	for (const topic of localDataset.topics) {
		if (
			criteria.topicId &&
			normalize(criteria.topicId) !== normalize(topic.id) &&
			normalize(criteria.topicId) !== normalize(topic.title)
		) {
			continue;
		}
		for (const criterion of topic.criteria) {
			const haystack = normalize(
				[
					topic.title,
					criterion.id,
					criterion.official.title,
					searchableText(criterion.official.tests),
					searchableText(criterion.official.references),
					criterion.editorial.fr.explanation ?? "",
					criterion.editorial.fr.userImpact ?? "",
				].join(" "),
			);
			if (tokens.every((token) => haystack.includes(token))) {
				const reference = normalize(criteria.text).replace(/[^\da-z]/g, "");
				const id = criterion.id.replace(/[^\da-z]/gi, "").toLowerCase();
				const title = normalize(plainText(criterion.official.title));
				const fullText = normalize(criteria.text);
				const rank = !fullText
					? 0
					: id === reference
						? 0
						: id.startsWith(reference) && reference.length > 0
							? 1
							: title.startsWith(fullText)
								? 2
								: 3;
				matches.push({
					result: { topicId: topic.id, topicTitle: topic.title, criterion },
					rank,
					order: sourceOrder,
				});
			}
			sourceOrder += 1;
		}
	}

	return matches
		.sort((left, right) => left.rank - right.rank || left.order - right.order)
		.map(({ result }) => result);
}

function addTopics(topics: Topic[]): void {
	for (const topic of topics) {
		const option = element("option", undefined, `${topic.id}. ${topic.title}`);
		option.value = topic.id;
		elements.topic.append(option);
	}
}

function renderResults(): void {
	const results =
		elements.sort.value === "reference"
			? [...allResults].sort((a, b) =>
					a.criterion.id.localeCompare(b.criterion.id, "fr", { numeric: true }),
				)
			: allResults;
	const shown = results.slice(0, visibleCount);

	elements.results.replaceChildren(...shown.map(createResultCard));
	elements.empty.hidden = results.length !== 0;
	elements.results.setAttribute("aria-busy", "false");
	elements.count.textContent = `${results.length} ${results.length > 1 ? "critères" : "critère"}${elements.search.value.trim() ? " trouvé" + (results.length > 1 ? "s" : "") : " disponibles"}`;
	elements.more.hidden = shown.length >= results.length;
}

function createResultCard(result: SearchResult): HTMLButtonElement {
	const { criterion } = result;
	const card = element("button", "result-card");
	card.type = "button";
	card.setAttribute("aria-label", `Ouvrir le critère ${criterion.id} : ${plainText(criterion.official.title)}`);

	const topLine = element("span", "result-meta");
	topLine.append(
		element("span", "topic-label", result.topicTitle),
		element("span", "result-reference", `Critère ${criterion.id}`),
	);

	const title = element("span", "result-title", plainText(criterion.official.title));
	const summary = criterion.editorial.fr.explanation
		? element("span", "result-summary", criterion.editorial.fr.explanation)
		: element(
				"span",
				"result-summary result-summary-muted",
				"Consulter le critère officiel et ses tests de vérification.",
			);
	const footer = element("span", "result-footer");
	const testCount = Object.keys(criterion.official.tests).length;
	footer.append(
		element("span", "test-count", `${testCount} ${testCount > 1 ? "tests" : "test"}`),
		element("span", "card-arrow", "Voir le détail →"),
	);
	card.append(topLine, title, summary, footer);
	card.addEventListener("click", () => void openCriterion(criterion.id));
	return card;
}

function showError(message: string): void {
	elements.results.setAttribute("aria-busy", "false");
	elements.results.replaceChildren();
	elements.empty.hidden = false;
	elements.empty.querySelector("h4")!.textContent = "Le référentiel n’a pas pu être chargé";
	elements.empty.querySelector("p")!.textContent = message;
	elements.count.textContent = message;
}

async function updateResults(): Promise<void> {
	elements.results.setAttribute("aria-busy", "true");
	elements.count.textContent = "Recherche en cours…";
	try {
		allResults = await search({
			text: elements.search.value,
			topicId: elements.topic.value,
		});
		visibleCount = 10;
		renderResults();
	} catch (error) {
		showError(error instanceof Error ? error.message : "Une erreur inattendue est survenue.");
	}
}

function addDetailSection(container: HTMLElement, heading: string, content: string): void {
	const section = element("section", "detail-section");
	section.append(element("h3", undefined, heading), element("p", undefined, content));
	container.append(section);
}

function renderPedagogicalTest(criterion: Criterion, pedagogy: PedagogicalTest): HTMLElement {
	const section = element("section", "detail-section pedagogy-section");
	const statusLabel = {
		draft: "Brouillon à relire",
		inReview: "En relecture",
		validated: "Contenu pédagogique relu",
	}[pedagogy.status];
	section.append(
		element("h3", undefined, `Contenu pédagogique — test ${pedagogy.testId}`),
		element("p", "pedagogy-status", statusLabel),
	);

	const testNumber = pedagogy.testId.slice(criterion.id.length + 1);
	const officialTest = criterion.official.tests[testNumber];
	if (officialTest) {
		const official = element("p", "pedagogy-official-test");
		official.append(
			element("strong", undefined, "Texte officiel du test : "),
			document.createTextNode(
				(Array.isArray(officialTest) ? officialTest : [officialTest])
					.map(plainText)
					.join(" "),
			),
		);
		section.append(official);
	}

	const plainLanguage = element("p", "pedagogy-summary");
	plainLanguage.append(
		element("strong", undefined, "Explication en langage clair — brouillon, non validé FALC : "),
		document.createTextNode(pedagogy.plainLanguageDraft),
	);
	const benefit = element("p", "pedagogy-benefit");
	benefit.append(
		element("strong", undefined, "Bénéfice pour les personnes concernées : "),
		document.createTextNode(pedagogy.userBenefit),
	);
	section.append(plainLanguage, benefit);

	const cases = element("div", "pedagogy-cases");
	for (const [index, example] of pedagogy.cases.entries()) {
		const card = element("article", "pedagogy-case");
		card.append(element("h4", undefined, `${index + 1}. ${example.title}`));

		card.append(element("p", "pedagogy-context", example.context));
		const resources = element("div", "pedagogy-resources");
		for (const resource of example.resources) {
			const resourceCard = element("div", "pedagogy-resource");
			const figure = element("figure", "pedagogy-figure");
			const image = element("img", "pedagogy-image");
			image.src = `/pedagogy/${encodeURIComponent(resource.file)}`;
			image.alt = "";
			if (example.visualEffect === "blur") image.classList.add("pedagogy-image-blurred");
			figure.append(image);
			if (example.id === "missing-alternative") {
				figure.append(
					element("figcaption", undefined, "Pouvez-vous dire ce que montre cette image ?"),
				);
			} else {
				figure.append(
					element(
						"figcaption",
						undefined,
						"Aperçu décoratif ; le comportement démontré est dans l’extrait HTML.",
					),
				);
			}
			image.addEventListener("error", () => {
				image.hidden = true;
				figure.append(
					element("figcaption", undefined, `Ressource non intégrée à l’application : ${resource.file}`),
				);
			});
			resourceCard.append(figure);

			const alternative = element("p", "pedagogy-resource-detail");
			alternative.append(
				element("strong", undefined, `Alternative pour ${resource.id} (${resource.file}) : `),
				element(
					"code",
					undefined,
					resource.altText === null
						? "attribut alt absent"
						: `alt="${resource.altText}"`,
				),
			);
			const provenance = element("p", "pedagogy-resource-detail");
			provenance.append(
				element("strong", undefined, "Source : "),
				document.createTextNode(resource.source),
				document.createTextNode(" "),
				element("strong", undefined, "Licence : "),
				document.createTextNode(resource.license),
			);
			resourceCard.append(alternative, provenance);
			resources.append(resourceCard);
		}
		card.append(resources);
		card.append(element("p", "pedagogy-case-explanation", example.explanation));

		const codeLabel = element("p", "pedagogy-code-label", `Exemple ${example.code.language.toUpperCase()}`);
		const codeBlock = element("pre", "pedagogy-code");
		codeBlock.append(element("code", undefined, example.code.snippet));
		card.append(codeLabel, codeBlock);
		cases.append(card);
	}
	section.append(cases);
	return section;
}

async function openCriterion(reference: string): Promise<void> {
	try {
		const result = runtimeIsTauri
			? await invoke<SearchResult | null>("get_criterion", { reference })
			: allResults.find((entry) => entry.criterion.id === reference) ??
				(await search({ text: reference, topicId: "" })).find(
					(entry) => entry.criterion.id === reference,
				);
		if (!result) throw new Error(`Le critère ${reference} est introuvable.`);

		const { criterion } = result;
		elements.dialogEyebrow.textContent = `${result.topicId}. ${result.topicTitle.toLocaleUpperCase("fr")}`;
		elements.dialogTitle.textContent = `Critère ${criterion.id}`;
		elements.dialogContent.replaceChildren();

		const title = element("p", "detail-official-title", plainText(criterion.official.title));
		elements.dialogContent.append(title);
		if (criterion.editorial.fr.explanation) {
			addDetailSection(elements.dialogContent, "En clair", criterion.editorial.fr.explanation);
		} else if (criterion.editorial.pedagogicalTests.length === 0) {
			addDetailSection(
				elements.dialogContent,
				"Contenu pédagogique à venir",
				"L’explication en langage clair et l’impact pour les personnes concernées seront ajoutés séparément du texte officiel.",
			);
		}
		for (const pedagogy of criterion.editorial.pedagogicalTests) {
			elements.dialogContent.append(renderPedagogicalTest(criterion, pedagogy));
		}

		const testsSection = element("section", "detail-section");
		testsSection.append(element("h3", undefined, "Tests de vérification"));
		const testsList = element("ol", "test-list");
		for (const [testNumber, rawTest] of Object.entries(criterion.official.tests)) {
			const testId = `${criterion.id}.${testNumber}`;
			const item = element("li", "test-item");
			const details = element("details", "test-details");
			const summary = element("summary", undefined, `Test ${testId}`);
			details.append(summary);
			const lines = Array.isArray(rawTest) ? rawTest : [rawTest];
			const description = element("ul", "test-lines");
			for (const line of lines) {
				description.append(element("li", undefined, plainText(line)));
			}
			details.append(description);
			const methodology = dataset?.methodologies[testId];
			if (methodology) {
				const method = element("p", "methodology");
				method.append(element("strong", undefined, "Méthodologie : "));
				method.append(document.createTextNode(plainText(methodology)));
				details.append(method);
			}
			item.append(details);
			testsList.append(item);
		}
		testsSection.append(testsList);
		elements.dialogContent.append(testsSection);

		const referenceValues: string[] = [];
		for (const reference of criterion.official.references) {
			for (const [key, value] of Object.entries(reference)) {
				const values = Array.isArray(value) ? value : [value];
				for (const entry of values) {
					if (typeof entry === "string") {
						referenceValues.push(`${key.toUpperCase()} · ${entry}`);
					}
				}
			}
		}
		if (referenceValues.length) {
			const references = element("section", "detail-section");
			references.append(element("h3", undefined, "Références normatives"));
			const referenceList = element("ul", "reference-list");
			for (const value of referenceValues) {
				referenceList.append(element("li", undefined, value));
			}
			references.append(referenceList);
			elements.dialogContent.append(references);
		}

		elements.dialog.showModal();
	} catch (error) {
		elements.announcement.textContent =
			error instanceof Error ? error.message : "Impossible d’ouvrir ce critère.";
	}
}

function resetSearch(): void {
	elements.search.value = "";
	elements.topic.value = "";
	elements.sort.value = "relevance";
	void updateResults();
	elements.search.focus();
}

async function initialize(): Promise<void> {
	initializeDisplayPreferences();
	elements.checkUpdates.disabled = !runtimeIsTauri;
	if (!runtimeIsTauri) {
		elements.updateStatus.textContent = "La vérification est disponible dans l’application Windows.";
	}
	const searchShortcut = document.querySelector<HTMLElement>("#search-shortcut");
	if (searchShortcut) {
		searchShortcut.textContent = /Macintosh|Mac OS X/.test(navigator.userAgent) ? "⌘ K" : "Ctrl K";
	}
	try {
		dataset = await loadDataset();
		addTopics(dataset.topics);
		await updateResults();
	} catch (error) {
		showError(error instanceof Error ? error.message : "Une erreur inattendue est survenue.");
	}

	elements.checkUpdates.addEventListener("click", async () => {
		elements.checkUpdates.disabled = true;
		elements.updateStatus.textContent = "Recherche des mises à jour…";
		try {
			const result = await invoke<UpdateCheckResult>("check_for_updates");
			if (result.updated) {
				const info = await invoke<DatasetInfo>("get_dataset_info");
				if (!dataset) throw new Error("Le corpus RGAA n’est pas encore chargé.");
				dataset = { ...dataset, ...info };
				await updateResults();
				elements.updateStatus.textContent = `Corpus RGAA ${result.rgaaVersion} mis à jour (séquence ${result.sequence}).`;
			} else if (result.sequence === null) {
				elements.updateStatus.textContent = "Aucune mise à jour des données n’est publiée pour le moment.";
			} else {
				elements.updateStatus.textContent = `Le corpus RGAA ${result.rgaaVersion} (séquence ${result.sequence}) est déjà à jour.`;
			}
		} catch (error) {
			elements.updateStatus.textContent =
				error instanceof Error ? error.message : "La vérification des mises à jour a échoué.";
		} finally {
			elements.checkUpdates.disabled = false;
		}
	});

	elements.form.addEventListener("submit", (event) => {
		event.preventDefault();
		if (searchTimer !== undefined) window.clearTimeout(searchTimer);
		void updateResults();
	});
	elements.search.addEventListener("input", () => {
		if (searchTimer !== undefined) window.clearTimeout(searchTimer);
		searchTimer = window.setTimeout(() => void updateResults(), 160);
	});
	elements.topic.addEventListener("change", () => void updateResults());
	elements.sort.addEventListener("change", renderResults);
	document.querySelector("#reset-filters")?.addEventListener("click", resetSearch);
	document.querySelector("#empty-reset")?.addEventListener("click", resetSearch);
	elements.more.addEventListener("click", () => {
		visibleCount += 10;
		renderResults();
	});
	document.querySelector("#dialog-close")?.addEventListener("click", () => elements.dialog.close());
	elements.dialog.addEventListener("click", (event) => {
		if (event.target === elements.dialog) elements.dialog.close();
	});
	document.addEventListener("keydown", (event) => {
		if (event.key === "Escape" && elements.dialog.open) {
			elements.dialog.close();
			return;
		}
		if ((event.ctrlKey || event.metaKey) && event.key.toLocaleLowerCase() === "k") {
			event.preventDefault();
			elements.search.focus();
			elements.search.select();
		}
	});
}

void initialize();
