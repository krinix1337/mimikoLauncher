<script setup lang="ts">
import { useElementSize } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import {
	BACKGROUND_PRESETS,
	type BackgroundSource,
	getCustomBackgroundUrl,
	getPanorama,
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

const PANORAMA_FACE_SIZE = 1024
const PANORAMA_FIELD_OF_VIEW = 85

const FACE_TRANSFORMS = [
	'translateZ(-512px)',
	'rotateY(-90deg) translateZ(-512px)',
	'rotateY(180deg) translateZ(-512px)',
	'rotateY(90deg) translateZ(-512px)',
	'rotateX(-90deg) translateZ(-512px)',
	'rotateX(90deg) translateZ(-512px)',
]

const root = ref<HTMLElement>()
const { width, height } = useElementSize(root)

const imageUrl = ref<string | null>(null)
const panoramaFaces = ref<string[] | null>(null)
const loaded = ref(false)

const perspective = computed(() => {
	const halfFov = (PANORAMA_FIELD_OF_VIEW * Math.PI) / 360
	return Math.max(width.value, height.value * 1.6) / 2 / Math.tan(halfFov)
})

let request = 0

watch(
	() => JSON.stringify(props.source),
	async () => {
		const current = ++request
		const source = props.source
		loaded.value = false
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
		if (faces) await Promise.all(faces.map(preload))
		if (current === request) loaded.value = true
	},
	{ immediate: true },
)

function preload(url: string) {
	return new Promise<void>((resolve) => {
		const image = new Image()
		image.onload = image.onerror = () => resolve()
		image.src = url
	})
}

const layerStyle = computed(() => ({
	filter: props.blur > 0 ? `blur(${props.blur}px)` : undefined,
	transform: props.blur > 0 ? `scale(${1 + props.blur / 200})` : undefined,
}))
</script>

<template>
	<div ref="root" class="app-background" aria-hidden="true">
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
			<div
				v-else-if="panoramaFaces"
				:key="panoramaFaces[0]"
				class="app-background__layer app-background__panorama"
				:class="{ 'app-background__layer--hidden': !loaded }"
				:style="{ ...layerStyle, perspective: `${perspective}px` }"
			>
				<div
					class="app-background__camera"
					:class="{ 'app-background__camera--rotating': rotate }"
					:style="{ '--panorama-distance': `${perspective}px` }"
				>
					<img
						v-for="(face, index) in panoramaFaces"
						:key="face"
						:src="face"
						alt=""
						class="app-background__face"
						:style="{
							width: `${PANORAMA_FACE_SIZE}px`,
							height: `${PANORAMA_FACE_SIZE}px`,
							transform: `${FACE_TRANSFORMS[index]} scale(1.003)`,
						}"
						draggable="false"
					/>
				</div>
			</div>
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

.app-background__panorama {
	display: flex;
	align-items: center;
	justify-content: center;
	overflow: hidden;
}

.app-background__camera {
	position: relative;
	width: 0;
	height: 0;
	transform-style: preserve-3d;
	transform: translateZ(var(--panorama-distance)) rotateX(-8deg) rotateY(0deg);
}

.app-background__camera--rotating {
	animation: app-background-pan 240s linear infinite;
}

.app-background__face {
	position: absolute;
	left: -512px;
	top: -512px;
	max-width: none;
	backface-visibility: hidden;
	image-rendering: auto;
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

@keyframes app-background-pan {
	from {
		transform: translateZ(var(--panorama-distance)) rotateX(-8deg) rotateY(0deg);
	}
	to {
		transform: translateZ(var(--panorama-distance)) rotateX(-8deg) rotateY(-360deg);
	}
}

@media (prefers-reduced-motion: reduce) {
	.app-background__camera--rotating {
		animation: none;
	}
}
</style>
