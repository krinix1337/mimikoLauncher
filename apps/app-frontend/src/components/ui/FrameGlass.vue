<script setup lang="ts">
import { useEventListener, useMutationObserver, useResizeObserver } from '@vueuse/core'
import { onMounted, ref } from 'vue'

/**
 * One frosted layer over the whole window with a hole cut out for the page, so the title bar,
 * navigation rail and right panel share a single seamless glass surface.
 */
const clipPath = ref<string>()
const viewport = ref<HTMLElement | null>(null)

function update() {
	viewport.value ??= document.querySelector<HTMLElement>('.app-viewport')
	const element = viewport.value
	if (!element) {
		requestAnimationFrame(update)
		return
	}
	const width = window.innerWidth
	const height = window.innerHeight
	const rect = element.getBoundingClientRect()
	const radius = Math.min(
		parseFloat(getComputedStyle(element.parentElement ?? element).borderTopLeftRadius) || 0,
		rect.width / 2,
		rect.height / 2,
	)
	const { left, top, right, bottom } = rect
	clipPath.value =
		`path(evenodd, 'M0 0H${width}V${height}H0Z ` +
		`M${left + radius} ${top}H${right}V${bottom}H${left}V${top + radius}` +
		`A${radius} ${radius} 0 0 1 ${left + radius} ${top}Z')`
}

onMounted(update)
useEventListener(window, 'resize', update)
useResizeObserver(viewport, update)
useMutationObserver(
	() => document.querySelector('.app-contents'),
	update,
	{ attributes: true, attributeFilter: ['class', 'style'] },
)
</script>

<template>
	<div class="frame-glass" :style="{ clipPath }" aria-hidden="true" />
</template>

<style scoped>
.frame-glass {
	position: absolute;
	inset: 0;
	background: var(--frame-tint);
}
</style>
