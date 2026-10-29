<script setup lang="ts">
import {
	AppearanceSettingsLayout,
	injectAuth,
	injectUserPreferences,
	provideAppearanceSettings,
	Slider,
	Toggle,
	useSavable,
} from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { platform } from '@tauri-apps/plugin-os'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import {
	BACKGROUND_PRESETS,
	type BackgroundSource,
	getCustomBackgroundUrl,
	getPanorama,
	importBackground,
	PANORAMA_PRESETS,
	useBackground,
} from '@/composables/use-background.ts'
import {
	M3_PRESETS,
	type M3Palette,
	previewColor,
	useM3Palette,
} from '@/composables/use-m3-palette.ts'
import { useNavLabels } from '@/composables/use-nav-labels.ts'
import { useShowNews } from '@/composables/use-show-news.ts'
import { type ColorTheme, isDarkTheme, useTheme } from '@/composables/use-theme.ts'
import { injectAppSettingsModal } from '@/features/settings/app-settings-modal/context'
import { type AppSettings, get, set } from '@/helpers/settings.ts'

const theme = useTheme()
const appSettings = useAppSettings()
const auth = injectAuth()
const { updatePreferences } = injectUserPreferences()
const settingsModal = injectAppSettingsModal()
const os = platform()
const showNavLabels = useNavLabels()
const showNews = useShowNews()
const m3Palette = useM3Palette()

function isSelected(palette: M3Palette): boolean {
	return (
		m3Palette.value.monochrome === palette.monochrome &&
		m3Palette.value.seed.toLowerCase() === palette.seed.toLowerCase()
	)
}

const isCustom = computed(() => !M3_PRESETS.some((preset) => isSelected(preset.palette)))

const background = useBackground()
const customPreview = ref<string | null>(null)
const importing = ref(false)
const downloadingPanorama = ref<string | null>(null)
const panoramaError = ref<string | null>(null)

function isBackground(source: BackgroundSource): boolean {
	return JSON.stringify(background.value.source) === JSON.stringify(source)
}

function setBackground(source: BackgroundSource) {
	background.value.source = source
}

async function selectPanorama(id: string) {
	if (downloadingPanorama.value) return
	downloadingPanorama.value = id
	panoramaError.value = null
	try {
		await getPanorama(id)
		setBackground({ kind: 'panorama', id })
	} catch {
		panoramaError.value = 'Не удалось скачать панораму. Проверь подключение к интернету.'
	} finally {
		downloadingPanorama.value = null
	}
}

async function refreshCustomPreview() {
	const source = background.value.source
	customPreview.value =
		source.kind === 'custom' ? await getCustomBackgroundUrl(source.name).catch(() => null) : null
}

async function pickCustomBackground() {
	const path = await open({
		multiple: false,
		directory: false,
		filters: [{ name: 'Изображения', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif'] }],
	})
	if (typeof path !== 'string') return
	importing.value = true
	try {
		setBackground({ kind: 'custom', name: await importBackground(path) })
	} finally {
		importing.value = false
	}
}

watch(() => background.value.source, refreshCustomPreview, { immediate: true })

type AppearanceSettingsState = {
	theme: ColorTheme
	syncAcrossDevices: boolean
	advancedRendering: boolean
	nativeDecorations: boolean
}

function getAppearanceSettingsState(): AppearanceSettingsState {
	return {
		theme: theme.preferred,
		syncAcrossDevices: theme.syncAcrossDevices,
		advancedRendering: theme.advancedRendering,
		nativeDecorations: appSettings.nativeDecorations,
	}
}

const { saved, current, changes, saving, hasChanges, reset, save } = useSavable(
	getAppearanceSettingsState,
	async (appearanceChanges) => {
		const value = current.value
		if (
			value.syncAcrossDevices &&
			auth.user.value &&
			(appearanceChanges.theme !== undefined || appearanceChanges.syncAcrossDevices !== undefined)
		) {
			await updatePreferences({
				appearance: value.theme === 'system' ? { auto: true } : { auto: false, theme: value.theme },
			})
		}

		const nextSettings: AppSettings = {
			...(await get()),
			theme: value.theme,
			sync_theme_across_devices: value.syncAcrossDevices,
			advanced_rendering: value.advancedRendering,
			native_decorations: value.nativeDecorations,
		}

		await set(nextSettings)
		if (isDarkTheme(value.theme)) {
			theme.preferredDark = value.theme
		}
		theme.preferred = value.theme
		theme.syncAcrossDevices = value.syncAcrossDevices
		theme.advancedRendering = value.advancedRendering
		appSettings.nativeDecorations = value.nativeDecorations
	},
)

const themeOptions = computed(() =>
	theme.options.filter(
		(option) => option !== 'retro' || appSettings.devMode || current.value.theme === 'retro',
	),
)

const preferredDarkTheme = computed(() =>
	isDarkTheme(current.value.theme) ? current.value.theme : theme.preferredDark,
)

function setTheme(value: ColorTheme): void {
	current.value.theme = value
}

function setSyncAcrossDevices(enabled: boolean): void {
	current.value.syncAcrossDevices = enabled
}

function setAdvancedRendering(enabled: boolean): void {
	current.value.advancedRendering = enabled
}

function setNativeDecorations(enabled: boolean): void {
	current.value.nativeDecorations = enabled
}

watch(
	[() => current.value.theme, () => saved.value.theme],
	([selectedTheme, savedTheme]) => {
		theme.preview = selectedTheme === savedTheme ? null : selectedTheme
	},
	{ immediate: true },
)

async function saveAppearanceSettings(): Promise<void> {
	try {
		await save()
	} catch {
		return
	}
}

onMounted(() => {
	settingsModal.registerUnsavedChangesController({
		hasChanges: () => hasChanges.value,
		getOriginal: () => saved.value,
		getModified: () => changes.value,
		isSaving: () => saving.value,
		reset,
		save: saveAppearanceSettings,
	})
})

onBeforeUnmount(() => {
	theme.preview = null
	settingsModal.registerUnsavedChangesController(null)
})

provideAppearanceSettings({
	deferPersistence: true,
	theme: {
		current: computed(() => current.value.theme),
		options: themeOptions,
		system: computed(() => (theme.native === 'light' ? 'light' : preferredDarkTheme.value)),
		preferredDark: preferredDarkTheme,
		set: setTheme,
		syncAcrossDevices: {
			value: computed(() => current.value.syncAcrossDevices),
			set: setSyncAcrossDevices,
		},
		syncDisabled: computed(() => !auth.user.value),
	},
	advancedRendering: {
		value: computed(() => current.value.advancedRendering),
		set: setAdvancedRendering,
	},
	nativeDecorations:
		os !== 'macos'
			? {
					value: computed(() => current.value.nativeDecorations),
					set: setNativeDecorations,
				}
			: undefined,
	updatePreferences,
})
</script>

<template>
	<AppearanceSettingsLayout />
	<div class="mt-6">
		<h2 class="m-0 text-lg font-semibold text-contrast">Цвет Material 3</h2>
		<p class="m-0 mt-1 text-secondary">
			Вся палитра темы строится из одного цвета, как в Android. «Чёрный» делает тему без цветных
			акцентов.
		</p>
		<div class="mt-3 flex flex-wrap items-center gap-2">
			<button
				v-for="preset in M3_PRESETS"
				:key="preset.name"
				v-tooltip="preset.name"
				type="button"
				class="m3-swatch"
				:class="{ 'm3-swatch--selected': isSelected(preset.palette) }"
				:style="{ '--swatch': previewColor(preset.palette, true) }"
				:aria-label="preset.name"
				@click="m3Palette = { ...preset.palette }"
			/>
			<label v-tooltip="'Свой цвет'" class="m3-swatch m3-swatch--custom" :class="{ 'm3-swatch--selected': isCustom }">
				<input
					type="color"
					:value="m3Palette.seed"
					aria-label="Свой цвет"
					@input="m3Palette = { seed: ($event.target as HTMLInputElement).value, monochrome: false }"
				/>
			</label>
		</div>
	</div>
	<div class="mt-6">
		<h2 class="m-0 text-lg font-semibold text-contrast">Фон</h2>
		<p class="m-0 mt-1 text-secondary">
			Картинка или вращающаяся панорама из главного меню Minecraft за страницами лаунчера.
		</p>
		<div class="bg-grid mt-3">
			<button
				type="button"
				class="bg-tile bg-tile--none"
				:class="{ 'bg-tile--selected': isBackground({ kind: 'none' }) }"
				@click="setBackground({ kind: 'none' })"
			>
				<span class="bg-tile__label">Без фона</span>
			</button>
			<button
				v-for="preset in BACKGROUND_PRESETS"
				:key="preset.id"
				v-tooltip="`${preset.name} · фото ${preset.author}, Unsplash`"
				type="button"
				class="bg-tile"
				:class="{ 'bg-tile--selected': isBackground({ kind: 'preset', id: preset.id }) }"
				:aria-label="preset.name"
				@click="setBackground({ kind: 'preset', id: preset.id })"
			>
				<img :src="preset.url" alt="" loading="lazy" draggable="false" />
			</button>
			<button
				v-if="customPreview"
				type="button"
				class="bg-tile bg-tile--selected"
				aria-label="Своё изображение"
			>
				<img :src="customPreview" alt="" draggable="false" />
				<span class="bg-tile__badge">Своё</span>
			</button>
			<button
				type="button"
				class="bg-tile bg-tile--add"
				:disabled="importing"
				@click="pickCustomBackground"
			>
				<span class="bg-tile__label">{{ importing ? 'Загрузка…' : '+ Своё изображение' }}</span>
			</button>
		</div>

		<h3 class="m-0 mt-5 text-base font-semibold text-contrast">Панорамы Minecraft</h3>
		<p class="m-0 mt-1 text-secondary">
			Все панорамы главного меню Java Edition без повторов. Скачиваются с серверов Mojang при
			первом выборе.
		</p>
		<div class="bg-grid mt-3">
			<button
				v-for="panorama in PANORAMA_PRESETS"
				:key="panorama.id"
				v-tooltip="`${panorama.name} · ${panorama.versions}`"
				type="button"
				class="bg-tile"
				:class="{
					'bg-tile--selected': isBackground({ kind: 'panorama', id: panorama.id }),
					'bg-tile--loading': downloadingPanorama === panorama.id,
				}"
				:disabled="downloadingPanorama !== null"
				:aria-label="panorama.name"
				@click="selectPanorama(panorama.id)"
			>
				<img :src="panorama.preview" alt="" loading="lazy" draggable="false" />
				<span class="bg-tile__badge">{{ panorama.versions }}</span>
				<span v-if="downloadingPanorama === panorama.id" class="bg-tile__spinner" />
			</button>
		</div>
		<p v-if="panoramaError" class="m-0 mt-2 text-red">{{ panoramaError }}</p>

		<template v-if="background.source.kind !== 'none'">
			<div class="mt-5 flex flex-col gap-2.5">
				<h3 class="m-0 text-base font-semibold text-contrast">Затемнение</h3>
				<Slider id="background-dim" v-model="background.dim" :min="0" :max="90" :step="5" unit="%" />
			</div>
			<div class="mt-4 flex flex-col gap-2.5">
				<h3 class="m-0 text-base font-semibold text-contrast">Размытие</h3>
				<Slider id="background-blur" v-model="background.blur" :min="0" :max="24" :step="1" unit="px" />
			</div>
			<div class="mt-4 flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-base font-semibold text-contrast">Акцент под фон</h3>
					<p class="m-0 mt-1 text-secondary">
						Цвет кнопок и выделений берётся из фона, как на Android. Заменяет выбранный цвет
						Material 3.
					</p>
				</div>
				<Toggle id="background-accent" v-model="background.accent" />
			</div>
			<div class="mt-4 flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-base font-semibold text-contrast">На всех страницах</h3>
					<p class="m-0 mt-1 text-secondary">Иначе фон виден только на главной.</p>
				</div>
				<Toggle id="background-everywhere" v-model="background.everywhere" />
			</div>
			<div
				v-if="background.source.kind === 'panorama'"
				class="mt-4 flex items-center justify-between gap-4"
			>
				<div>
					<h3 class="m-0 text-base font-semibold text-contrast">Вращать панораму</h3>
					<p class="m-0 mt-1 text-secondary">Медленный поворот камеры, как в игре.</p>
				</div>
				<Toggle id="background-rotate" v-model="background.rotate" />
			</div>
		</template>
	</div>
	<div class="mt-6 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">Подписи в боковой панели</h2>
			<p class="m-0 mt-1 text-secondary">
				Показывать названия под иконками слева. Работает в темах Material 3.
			</p>
		</div>
		<Toggle id="nav-labels" v-model="showNavLabels" />
	</div>
	<div class="mt-6 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">Новости Minecraft</h2>
			<p class="m-0 mt-1 text-secondary">Показывать новости Minecraft в панели справа.</p>
		</div>
		<Toggle id="show-news" v-model="showNews" />
	</div>
</template>

<style scoped>
.m3-swatch {
	position: relative;
	width: 2.5rem;
	height: 2.5rem;
	padding: 0;
	border: 2px solid var(--color-divider);
	border-radius: 50%;
	background: var(--swatch);
	cursor: pointer;
	transition:
		border-radius 0.3s cubic-bezier(0.34, 0.8, 0.34, 1),
		box-shadow 0.2s cubic-bezier(0.34, 0.8, 0.34, 1);
}

.m3-swatch--selected {
	border-radius: 12px;
	box-shadow:
		0 0 0 2px var(--color-bg),
		0 0 0 4px var(--color-contrast);
}

.bg-grid {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(8.5rem, 1fr));
	gap: 0.5rem;
}

.bg-tile {
	position: relative;
	display: flex;
	align-items: center;
	justify-content: center;
	aspect-ratio: 16 / 9;
	overflow: hidden;
	padding: 0;
	border: none;
	border-radius: 12px;
	background: var(--color-button-bg);
	color: var(--color-text-primary);
	cursor: pointer;
	outline: 2px solid transparent;
	outline-offset: 2px;
	transition:
		border-radius 0.3s cubic-bezier(0.34, 0.8, 0.34, 1),
		outline-color 0.2s cubic-bezier(0.34, 0.8, 0.34, 1);

	img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		transition: transform 0.4s cubic-bezier(0.34, 0.8, 0.34, 1);
	}

	&:hover img {
		transform: scale(1.06);
	}

	&:disabled {
		cursor: progress;
		opacity: 0.6;
	}
}

.bg-tile--selected {
	border-radius: 20px;
	outline-color: var(--color-brand);
}

.bg-tile--add {
	border: 2px dashed var(--color-divider);
}

.bg-tile__label {
	padding: 0 0.5rem;
	font-size: 0.875rem;
	font-weight: 600;
	text-align: center;
}

.bg-tile:disabled:not(.bg-tile--loading) {
	cursor: default;
	opacity: 1;
}

.bg-tile--loading img {
	filter: brightness(0.5);
}

.bg-tile__spinner {
	position: absolute;
	width: 1.75rem;
	height: 1.75rem;
	border: 3px solid rgb(255 255 255 / 0.3);
	border-top-color: #fff;
	border-radius: 50%;
	animation: bg-tile-spin 0.8s linear infinite;
}

@keyframes bg-tile-spin {
	to {
		transform: rotate(360deg);
	}
}

.bg-tile__badge {
	position: absolute;
	left: 0.375rem;
	bottom: 0.375rem;
	padding: 0.125rem 0.5rem;
	border-radius: 999px;
	background: rgb(0 0 0 / 0.6);
	color: #fff;
	font-size: 0.75rem;
	font-weight: 600;
}

.m3-swatch--custom {
	overflow: hidden;
	background: conic-gradient(red, yellow, lime, aqua, blue, magenta, red);

	input {
		position: absolute;
		inset: 0;
		opacity: 0;
		cursor: pointer;
	}
}
</style>
