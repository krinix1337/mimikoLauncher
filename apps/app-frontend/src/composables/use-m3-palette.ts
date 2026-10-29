import {
	argbFromHex,
	type DynamicColor,
	type DynamicScheme,
	Hct,
	hexFromArgb,
	MaterialDynamicColors,
	SchemeMonochrome,
	SchemeTonalSpot,
} from '@material/material-color-utilities'
import { ref, watch } from 'vue'

const STORAGE_KEY = 'mimiko-m3-palette'

/** The M3 baseline seed color, which gives the default purple scheme. */
export const DEFAULT_M3_SEED = '#6750a4'

export interface M3Palette {
	/** Seed color the scheme is generated from. */
	seed: string
	/** Monochrome uses only greys, black and white. */
	monochrome: boolean
}

export const M3_PRESETS: { name: string; palette: M3Palette }[] = [
	{ name: 'Фиолетовый', palette: { seed: DEFAULT_M3_SEED, monochrome: false } },
	{ name: 'Чёрный', palette: { seed: '#000000', monochrome: true } },
	{ name: 'Синий', palette: { seed: '#1a73e8', monochrome: false } },
	{ name: 'Бирюзовый', palette: { seed: '#00897b', monochrome: false } },
	{ name: 'Зелёный', palette: { seed: '#3b8132', monochrome: false } },
	{ name: 'Жёлтый', palette: { seed: '#c9a400', monochrome: false } },
	{ name: 'Оранжевый', palette: { seed: '#e8710a', monochrome: false } },
	{ name: 'Красный', palette: { seed: '#c5221f', monochrome: false } },
	{ name: 'Розовый', palette: { seed: '#d81b60', monochrome: false } },
]

const ROLES: Record<string, DynamicColor> = {
	primary: MaterialDynamicColors.primary,
	'on-primary': MaterialDynamicColors.onPrimary,
	'primary-container': MaterialDynamicColors.primaryContainer,
	'on-primary-container': MaterialDynamicColors.onPrimaryContainer,
	secondary: MaterialDynamicColors.secondary,
	'on-secondary': MaterialDynamicColors.onSecondary,
	'secondary-container': MaterialDynamicColors.secondaryContainer,
	'on-secondary-container': MaterialDynamicColors.onSecondaryContainer,
	tertiary: MaterialDynamicColors.tertiary,
	'on-tertiary': MaterialDynamicColors.onTertiary,
	'tertiary-container': MaterialDynamicColors.tertiaryContainer,
	'on-tertiary-container': MaterialDynamicColors.onTertiaryContainer,
	error: MaterialDynamicColors.error,
	'on-error': MaterialDynamicColors.onError,
	'error-container': MaterialDynamicColors.errorContainer,
	'on-error-container': MaterialDynamicColors.onErrorContainer,
	surface: MaterialDynamicColors.surface,
	'surface-bright': MaterialDynamicColors.surfaceBright,
	'surface-container-lowest': MaterialDynamicColors.surfaceContainerLowest,
	'surface-container-low': MaterialDynamicColors.surfaceContainerLow,
	'surface-container': MaterialDynamicColors.surfaceContainer,
	'surface-container-high': MaterialDynamicColors.surfaceContainerHigh,
	'surface-container-highest': MaterialDynamicColors.surfaceContainerHighest,
	'on-surface': MaterialDynamicColors.onSurface,
	'on-surface-variant': MaterialDynamicColors.onSurfaceVariant,
	outline: MaterialDynamicColors.outline,
	'outline-variant': MaterialDynamicColors.outlineVariant,
	'inverse-surface': MaterialDynamicColors.inverseSurface,
	'inverse-on-surface': MaterialDynamicColors.inverseOnSurface,
	'inverse-primary': MaterialDynamicColors.inversePrimary,
}

function readStored(): M3Palette {
	try {
		const stored = JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? 'null')
		if (stored && typeof stored.seed === 'string') {
			return { seed: stored.seed, monochrome: !!stored.monochrome }
		}
	} catch {
		// unreadable value or blocked storage
	}
	return { seed: DEFAULT_M3_SEED, monochrome: false }
}

const palette = ref<M3Palette>(readStored())

function buildScheme(value: M3Palette, isDark: boolean): DynamicScheme {
	const source = Hct.fromInt(argbFromHex(value.seed))
	return value.monochrome
		? new SchemeMonochrome(source, isDark, 0)
		: new SchemeTonalSpot(source, isDark, 0)
}

/**
 * Writes the M3 color roles generated from the chosen palette onto the root element while an M3
 * theme is active, and removes them otherwise so other themes keep their own colors.
 */
function applyPalette() {
	const root = document.documentElement
	const isDark = root.classList.contains('m3-mode')
	const isLight = root.classList.contains('m3_light-mode')

	for (const role of Object.keys(ROLES)) {
		root.style.removeProperty(`--md-${role}`)
	}
	if (!isDark && !isLight) return

	const scheme = buildScheme(palette.value, isDark)
	for (const [role, color] of Object.entries(ROLES)) {
		root.style.setProperty(`--md-${role}`, hexFromArgb(color.getArgb(scheme)))
	}
}

let started = false

/** Starts applying the saved palette and keeps it in sync with theme changes. */
export function startM3Palette() {
	if (started) return
	started = true

	applyPalette()
	new MutationObserver(applyPalette).observe(document.documentElement, {
		attributes: true,
		attributeFilter: ['class'],
	})
	watch(
		palette,
		(value) => {
			applyPalette()
			try {
				window.localStorage.setItem(STORAGE_KEY, JSON.stringify(value))
			} catch {
				// storage blocked or full
			}
		},
		{ deep: true },
	)
}

/** The palette the Material 3 themes are generated from. */
export function useM3Palette() {
	return palette
}

/** Generates a preview primary color for a palette, for color swatches. */
export function previewColor(value: M3Palette, isDark: boolean): string {
	return hexFromArgb(MaterialDynamicColors.primary.getArgb(buildScheme(value, isDark)))
}
