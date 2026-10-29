<script setup lang="ts">
import {
	AppearanceSettingsLayout,
	injectAuth,
	injectUserPreferences,
	provideAppearanceSettings,
	Toggle,
	useSavable,
} from '@modrinth/ui'
import { platform } from '@tauri-apps/plugin-os'
import { computed, onBeforeUnmount, onMounted, watch } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import {
	M3_PRESETS,
	type M3Palette,
	previewColor,
	useM3Palette,
} from '@/composables/use-m3-palette.ts'
import { useNavLabels } from '@/composables/use-nav-labels.ts'
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
const m3Palette = useM3Palette()

function isSelected(palette: M3Palette): boolean {
	return (
		m3Palette.value.monochrome === palette.monochrome &&
		m3Palette.value.seed.toLowerCase() === palette.seed.toLowerCase()
	)
}

const isCustom = computed(() => !M3_PRESETS.some((preset) => isSelected(preset.palette)))

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
	<div class="mt-6 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">Подписи в боковой панели</h2>
			<p class="m-0 mt-1 text-secondary">
				Показывать названия под иконками слева. Работает в темах Material 3.
			</p>
		</div>
		<Toggle id="nav-labels" v-model="showNavLabels" />
	</div>
</template>

<style scoped>
.m3-swatch {
	position: relative;
	width: 2.5rem;
	height: 2.5rem;
	padding: 0;
	border: 2px solid var(--color-divider);
	border-radius: 9999px;
	background: var(--swatch);
	cursor: pointer;
	transition:
		border-radius 0.35s cubic-bezier(0.42, 1.67, 0.21, 0.9),
		box-shadow 0.2s ease;
}

.m3-swatch--selected {
	border-radius: 12px;
	box-shadow:
		0 0 0 2px var(--color-bg),
		0 0 0 4px var(--color-contrast);
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
