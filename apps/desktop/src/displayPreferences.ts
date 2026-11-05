type Theme = "light" | "dark" | "sepia";
type Palette =
	| "standard"
	| "blue-amber"
	| "deuteranomaly"
	| "deuteranopia"
	| "protanomaly"
	| "protanopia"
	| "tritanomaly"
	| "tritanopia"
	| "achromatopsia";
type VisionPalette = Exclude<Palette, "standard" | "blue-amber">;
type FontFamily =
	| "system"
	| "andika"
	| "atkinson"
	| "lexend"
	| "luciole"
	| "opendyslexic"
	| "ubuntu";

type DisplayPreferences = {
	theme: Theme;
	palette: Palette;
	accent: string;
	font: FontFamily;
};

const storageKey = "accessiguide.display-preferences.v1";
const defaultPreferences: DisplayPreferences = {
	theme: "light",
	palette: "standard",
	accent: "#4662dc",
	font: "system",
};

const themeSurfaces: Record<Theme, [string, string]> = {
	light: ["#f6f7fb", "#ffffff"],
	dark: ["#131a27", "#202a3b"],
	sepia: ["#f2e7d2", "#fbf4e6"],
};

const visionPalettes: Record<VisionPalette, { accent: string; status: string }> = {
	deuteranomaly: { accent: "#124e96", status: "#8c4b00" },
	deuteranopia: { accent: "#004b87", status: "#a65300" },
	protanomaly: { accent: "#3f51a8", status: "#805500" },
	protanopia: { accent: "#283593", status: "#985000" },
	tritanomaly: { accent: "#7a3163", status: "#176b58" },
	tritanopia: { accent: "#70263b", status: "#006b67" },
	achromatopsia: { accent: "#343a40", status: "#555b61" },
};

function isTheme(value: unknown): value is Theme {
	return value === "light" || value === "dark" || value === "sepia";
}

function isPalette(value: unknown): value is Palette {
	return (
		value === "standard" ||
		value === "blue-amber" ||
		value === "deuteranomaly" ||
		value === "deuteranopia" ||
		value === "protanomaly" ||
		value === "protanopia" ||
		value === "tritanomaly" ||
		value === "tritanopia" ||
		value === "achromatopsia"
	);
}

function isVisionPalette(value: Palette): value is VisionPalette {
	return Object.prototype.hasOwnProperty.call(visionPalettes, value);
}

function isFontFamily(value: unknown): value is FontFamily {
	return (
		value === "system" ||
		value === "andika" ||
		value === "atkinson" ||
		value === "lexend" ||
		value === "luciole" ||
		value === "opendyslexic" ||
		value === "ubuntu"
	);
}

function isAccent(value: unknown): value is string {
	return typeof value === "string" && /^#[\da-f]{6}$/i.test(value);
}

function parsePreferences(value: string): DisplayPreferences {
	const parsed: unknown = JSON.parse(value);
	if (!parsed || typeof parsed !== "object") {
		throw new Error("Les préférences enregistrées ne sont pas valides.");
	}
	const font = "font" in parsed ? parsed.font : "system";
	if (
		!("theme" in parsed) ||
		!("palette" in parsed) ||
		!("accent" in parsed) ||
		!isTheme(parsed.theme) ||
		!isPalette(parsed.palette) ||
		!isAccent(parsed.accent) ||
		!isFontFamily(font)
	) {
		throw new Error("Les préférences enregistrées ne sont pas valides.");
	}
	return {
		theme: parsed.theme,
		palette: parsed.palette,
		accent: parsed.accent,
		font,
	};
}

function toRgb(color: string): [number, number, number] {
	const hex = color.slice(1);
	return [
		Number.parseInt(hex.slice(0, 2), 16),
		Number.parseInt(hex.slice(2, 4), 16),
		Number.parseInt(hex.slice(4, 6), 16),
	];
}

function luminance(color: string): number {
	const channels = toRgb(color).map((value) => {
		const normalized = value / 255;
		return normalized <= 0.04045
			? normalized / 12.92
			: ((normalized + 0.055) / 1.055) ** 2.4;
	});
	return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrastRatio(first: string, second: string): number {
	const values = [luminance(first), luminance(second)].sort((a, b) => b - a);
	return (values[0] + 0.05) / (values[1] + 0.05);
}

function blend(first: string, second: string, amount: number): string {
	const firstChannels = toRgb(first);
	const secondChannels = toRgb(second);
	const channels = firstChannels.map((channel, index) =>
		Math.round(channel * (1 - amount) + secondChannels[index] * amount),
	);
	return `#${channels.map((channel) => channel.toString(16).padStart(2, "0")).join("")}`;
}

function contrastingTextColor(accent: string, surfaces: string[]): string {
	const direction = luminance(surfaces[0]) < 0.5 ? "#ffffff" : "#000000";
	if (surfaces.every((surface) => contrastRatio(accent, surface) >= 4.5)) {
		return accent;
	}
	for (let step = 1; step <= 100; step += 1) {
		const candidate = blend(accent, direction, step / 100);
		if (surfaces.every((surface) => contrastRatio(candidate, surface) >= 4.5)) {
			return candidate;
		}
	}
	return direction;
}

function foregroundForAccent(accent: string): string {
	return contrastRatio(accent, "#000000") >= contrastRatio(accent, "#ffffff")
		? "#000000"
		: "#ffffff";
}

function paletteColors(
	theme: Theme,
	palette: Palette,
	accent: string,
	accentSurfaces: string[],
): { accent: string; status: string } {
	const profile = isVisionPalette(palette) ? visionPalettes[palette] : undefined;
	const profileAccent = profile?.accent ?? accent;
	let status = profile?.status;
	if (!status && palette === "blue-amber") {
		status = theme === "dark" ? "#f2ad50" : "#8a4c00";
	}
	if (!status) {
		status = theme === "dark" ? "#4fd1b5" : theme === "sepia" ? "#16725e" : "#0b806d";
	}
	return {
		accent: contrastingTextColor(profileAccent, accentSurfaces),
		status: contrastingTextColor(status, accentSurfaces),
	};
}

export function initializeDisplayPreferences(): void {
	const root = document.documentElement;
	const dialog = document.querySelector<HTMLDialogElement>("#settings-dialog")!;
	const form = document.querySelector<HTMLFormElement>("#settings-form")!;
	const openButton = document.querySelector<HTMLButtonElement>("#open-settings")!;
	const closeButton = document.querySelector<HTMLButtonElement>("#close-settings")!;
	const themeControl = document.querySelector<HTMLSelectElement>("#theme-choice")!;
	const paletteControl = document.querySelector<HTMLSelectElement>("#palette-choice")!;
	const accentControl = document.querySelector<HTMLInputElement>("#accent-choice")!;
	const fontControl = document.querySelector<HTMLSelectElement>("#font-choice")!;
	const status = document.querySelector<HTMLParagraphElement>("#preference-status")!;
	let preferences = defaultPreferences;

	try {
		const stored = localStorage.getItem(storageKey);
		if (stored) preferences = parsePreferences(stored);
	} catch (error) {
		status.textContent =
			error instanceof Error
				? `${error.message} Les réglages par défaut sont utilisés.`
				: "Les préférences n’ont pas pu être lues; les réglages par défaut sont utilisés.";
	}

	function applyPreferences(): void {
		const accentSoft = blend(preferences.accent, themeSurfaces[preferences.theme][1], 0.9);
		const surfaces = [...themeSurfaces[preferences.theme], accentSoft];
		const palette = paletteColors(
			preferences.theme,
			preferences.palette,
			preferences.accent,
			surfaces,
		);
		const accentForeground = foregroundForAccent(preferences.accent);
		root.dataset.theme = preferences.theme;
		root.dataset.palette = preferences.palette;
		root.style.setProperty("--blue", palette.accent);
		root.style.setProperty("--accent-fill", preferences.accent);
		root.style.setProperty("--accent-on", accentForeground);
		root.style.setProperty(
			"--blue-dark",
			blend(preferences.accent, accentForeground === "#000000" ? "#ffffff" : "#000000", 0.12),
		);
		root.style.setProperty("--accent-soft", accentSoft);
		root.style.setProperty("--status-color", palette.status);
		root.style.setProperty("--font-family", fontFamilies[preferences.font]);
		themeControl.value = preferences.theme;
		paletteControl.value = preferences.palette;
		accentControl.value = preferences.accent;
		fontControl.value = preferences.font;
	}

	function persistPreferences(): void {
		try {
			localStorage.setItem(storageKey, JSON.stringify(preferences));
			status.textContent = "Préférences enregistrées sur cet appareil.";
		} catch {
			status.textContent =
				"Les préférences sont appliquées pour cette session, mais n’ont pas pu être enregistrées.";
		}
	}

	function updatePreferences(next: DisplayPreferences): void {
		preferences = next;
		applyPreferences();
		persistPreferences();
	}

	applyPreferences();
	openButton.addEventListener("click", () => dialog.showModal());
	closeButton.addEventListener("click", () => dialog.close());
	dialog.addEventListener("keydown", (event) => {
		if (event.key === "Escape") {
			event.preventDefault();
			dialog.close();
		}
	});
	dialog.addEventListener("close", () => {
		requestAnimationFrame(() => openButton.focus());
	});
	dialog.addEventListener("click", (event) => {
		if (event.target === dialog) dialog.close();
	});
	form.addEventListener("submit", (event) => event.preventDefault());
	themeControl.addEventListener("change", () => {
		if (isTheme(themeControl.value)) {
			updatePreferences({ ...preferences, theme: themeControl.value });
		}
	});
	paletteControl.addEventListener("change", () => {
		if (isPalette(paletteControl.value)) {
			updatePreferences({ ...preferences, palette: paletteControl.value });
		}
	});
	accentControl.addEventListener("change", () => {
		if (isAccent(accentControl.value)) {
			updatePreferences({ ...preferences, accent: accentControl.value });
		}
	});
	fontControl.addEventListener("change", () => {
		if (isFontFamily(fontControl.value)) {
			updatePreferences({ ...preferences, font: fontControl.value });
		}
	});
}

const fontFamilies: Record<FontFamily, string> = {
	system: '"Segoe UI", "Arial", sans-serif',
	andika: '"AccessiGuide Andika", sans-serif',
	atkinson: '"AccessiGuide Atkinson", sans-serif',
	lexend: '"AccessiGuide Lexend", sans-serif',
	luciole: '"AccessiGuide Luciole", sans-serif',
	opendyslexic: '"AccessiGuide OpenDyslexic", sans-serif',
	ubuntu: '"AccessiGuide Ubuntu", sans-serif',
};
