import { invoke } from '@tauri-apps/api/core'
import { ref, watch } from 'vue'

import aurora from '@/assets/backgrounds/aurora.webp'
import auroraPeaks from '@/assets/backgrounds/aurora_peaks.webp'
import blue from '@/assets/backgrounds/blue.webp'
import forest from '@/assets/backgrounds/forest.webp'
import lake from '@/assets/backgrounds/lake.webp'
import peak from '@/assets/backgrounds/peak.webp'
import ridges from '@/assets/backgrounds/ridges.webp'
import sunrise from '@/assets/backgrounds/sunrise.webp'

export type BackgroundSource =
	| { kind: 'none' }
	| { kind: 'preset'; id: string }
	| { kind: 'panorama'; id: string }
	| { kind: 'custom'; name: string }

export interface BackgroundSettings {
	source: BackgroundSource
	/** Darkening over the image, 0–90 percent. */
	dim: number
	/** Blur radius in pixels. */
	blur: number
	/** Show the background on every page instead of only the home page. */
	everywhere: boolean
	/** Slowly rotate the panorama. */
	rotate: boolean
}

export interface BackgroundPreset {
	id: string
	name: string
	url: string
	author: string
}

/** Photos from Unsplash, free to use under the Unsplash License. */
export const BACKGROUND_PRESETS: BackgroundPreset[] = [
	{ id: 'peak', name: 'Вершина на закате', url: peak, author: 'Cristina Gottardi' },
	{ id: 'ridges', name: 'Хребты', url: ridges, author: 'Sven Pieren' },
	{ id: 'blue', name: 'Синие горы', url: blue, author: 'conti_photos' },
	{ id: 'lake', name: 'Отражение', url: lake, author: 'Johannes Plenio' },
	{ id: 'forest', name: 'Туманный лес', url: forest, author: 'Dejan Zakic' },
	{ id: 'aurora', name: 'Северное сияние', url: aurora, author: 'vingtcent' },
	{ id: 'aurora_peaks', name: 'Сияние над горами', url: auroraPeaks, author: 'supergios' },
	{ id: 'sunrise', name: 'Горное озеро', url: sunrise, author: 'onthesearchforpineapples' },
]

export interface PanoramaPreset {
	id: string
	name: string
	versions: string
	preview: string
}

const panoramaPreviews = import.meta.glob<string>('@/assets/backgrounds/panoramas/*.jpg', {
	eager: true,
	import: 'default',
})

function panorama(id: string, name: string, versions: string): PanoramaPreset {
	const preview = Object.entries(panoramaPreviews).find(([path]) => path.endsWith(`/${id}.jpg`))
	return { id, name, versions, preview: preview?.[1] ?? '' }
}

/** Every distinct title screen panorama of Minecraft: Java Edition, newest first. */
export const PANORAMA_PRESETS: PanoramaPreset[] = [
	panorama('26.3', 'Minecraft 26.3', '26.3'),
	panorama('26.2', 'Minecraft 26.2', '26.2'),
	panorama('26.1', 'Minecraft 26.1', '26.1 – 26.1.2'),
	panorama('1.21.11', 'Mounts of Mayhem', '1.21.11'),
	panorama('1.21.9', 'The Copper Age', '1.21.9 – 1.21.10'),
	panorama('1.21.6', 'Chase the Skies', '1.21.6 – 1.21.8'),
	panorama('1.21.5', 'Spring to Life', '1.21.5'),
	panorama('1.21.4', 'The Garden Awakens', '1.21.4'),
	panorama('1.21', 'Tricky Trials', '1.21 – 1.21.3'),
	panorama('1.20', 'Trails & Tales', '1.20 – 1.20.6'),
	panorama('1.19', 'The Wild Update', '1.19 – 1.19.4'),
	panorama('1.18', 'Caves & Cliffs II', '1.18 – 1.18.2'),
	panorama('1.17', 'Caves & Cliffs I', '1.17 – 1.17.1'),
	panorama('1.16', 'Nether Update', '1.16 – 1.16.5'),
	panorama('1.15', 'Buzzy Bees', '1.15 – 1.15.2'),
	panorama('1.14', 'Village & Pillage', '1.14 – 1.14.4'),
	panorama('1.13', 'Update Aquatic', '1.13 – 1.13.2'),
	panorama('classic', 'Классическая', '1.0 – 1.12.2'),
	panorama('potato', 'Poisonous Potato Update', '24w14potato'),
]

const STORAGE_KEY = 'mimiko-background'

const DEFAULTS: BackgroundSettings = {
	source: { kind: 'none' },
	dim: 55,
	blur: 0,
	everywhere: false,
	rotate: true,
}

function readStored(): BackgroundSettings {
	try {
		const stored = JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? 'null')
		return stored ? { ...DEFAULTS, ...stored } : { ...DEFAULTS }
	} catch {
		return { ...DEFAULTS }
	}
}

const settings = ref<BackgroundSettings>(readStored())

watch(
	settings,
	(value) => {
		try {
			window.localStorage.setItem(STORAGE_KEY, JSON.stringify(value))
		} catch {
			// storage blocked or full
		}
	},
	{ deep: true },
)

/** Background image or panorama shown behind the launcher pages. */
export function useBackground() {
	return settings
}

const panoramaCache = new Map<string, Promise<string[]>>()

/** URLs of the six panorama faces in Minecraft order: front, right, back, left, top, bottom. */
export function getPanorama(id: string): Promise<string[]> {
	let faces = panoramaCache.get(id)
	if (!faces) {
		faces = invoke<string[]>('plugin:utils|background_panorama', { id })
		faces.catch(() => panoramaCache.delete(id))
		panoramaCache.set(id, faces)
	}
	return faces
}

export function importBackground(path: string): Promise<string> {
	return invoke<string>('plugin:utils|background_import', { path })
}

export function getCustomBackgroundUrl(name: string): Promise<string> {
	return invoke<string>('plugin:utils|background_custom_url', { name })
}
