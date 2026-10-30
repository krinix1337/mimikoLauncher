<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import PanoramaCanvas from '@/components/ui/PanoramaCanvas.vue'

import {
	BACKGROUND_PRESETS,
	type BackgroundSource,
	getCustomBackgroundUrl,
	getPanorama,
	getResolvedBackground,
} from '@/composables/use-background'

const props = withDefaults(
	defineProps<{
		source: BackgroundSource
		dim?: number
		blur?: number
		rotate?: boolean
	}>(),
	{ dim: 0, blur: 0, rotate: true },
)

const PANORAMA_FIELD_OF_VIEW = 85

const imageUrl = ref<string | null>(null)
const panoramaFaces = ref<string[] | null>(null)
const loaded = ref(false)
const readyPanorama = ref<string | null>(null)

let request = 0

watch(
	() => JSON.stringify(props.source),
	async () => {
		const current = ++request
		const source = props.source
		loaded.value = false
		const cached = getResolvedBackground(source)
		if (cached) {
			imageUrl.value = source.kind === 'panorama' ? null : cached[0]
			panoramaFaces.value = source.kind === 'panorama' ? cached : null
		}
		let image: string | null = null
		let faces: string[] | null = null
		try {
			if (source.kind === 'preset') {
				image = BACKGROUND_PRESETS.find((preset) => preset.id === source.id)?.url ?? null
			} else if (source.kind === 'custom') {
				image = await getCustomBackgroundUrl(source.name)
			} else if (source.kind === 'panorama') {
				faces = await getPanorama(source.id)
			}
		} catch {
			image = null
			faces = null
		}
		if (current !== request) return
		imageUrl.value = image
		panoramaFaces.value = faces
		if (!faces) loaded.value = true
	},
	{ immediate: true },
)

const layerStyle = computed(() => ({
	filter: props.blur > 0 ? `blur(${props.blur}px)` : undefined,
	transform: props.blur > 0 ? `scale(${1 + props.blur / 200})` : undefined,
}))
</script>

<template>
	<div class="app-background" aria-hidden="true">
		<Transition name="app-background-fade">
			<img
				v-if="imageUrl"
				:key="imageUrl"
				:src="imageUrl"
				alt=""
				class="app-background__layer app-background__image"
				:class="{ 'app-background__layer--hidden': !loaded }"
				:style="layerStyle"
				draggable="false"
				@load="loaded = true"
			/>
			<PanoramaCanvas
				v-else-if="panoramaFaces"
				:key="panoramaFaces[0]"
				class="app-background__layer"
				:class="{ 'app-background__layer--hidden': readyPanorama !== panoramaFaces[0] }"
				:style="{ transform: layerStyle.transform }"
				:faces="panoramaFaces"
				:field-of-view="PANORAMA_FIELD_OF_VIEW"
				:blur="blur"
				:rotate="rotate"
				@ready="readyPanorama = panoramaFaces[0]"
			/>
		</Transition>
		<div
			v-if="imageUrl || panoramaFaces"
			class="app-background__scrim"
			:style="{ '--background-dim': `${dim}%` }"
		/>
	</div>
</template>

<style scoped>
.app-background {
	position: absolute;
	inset: 0;
	z-index: -10;
	overflow: hidden;
	border-top-left-radius: var(--radius-xl);
	pointer-events: none;
	user-select: none;
	contain: strict;
}

.app-background__layer {
	position: absolute;
	inset: 0;
	transition: opacity 0.6s cubic-bezier(0.34, 0.8, 0.34, 1);
}

.app-background__layer--hidden {
	opacity: 0;
}

.app-background__image {
	width: 100%;
	height: 100%;
	object-fit: cover;
}

.app-background__scrim {
	position: absolute;
	inset: 0;
	background:
		linear-gradient(
			to bottom,
			color-mix(in srgb, var(--color-bg) calc(var(--background-dim) * 0.7), transparent),
			color-mix(in srgb, var(--color-bg) var(--background-dim), transparent) 45%,
			color-mix(in srgb, var(--color-bg) calc(var(--background-dim) + 10%), transparent)
		);
}

.app-background-fade-enter-from,
.app-background-fade-leave-to {
	opacity: 0;
}

.app-background-fade-leave-active {
	transition: opacity 0.6s cubic-bezier(0.34, 0.8, 0.34, 1);
}
</style>
