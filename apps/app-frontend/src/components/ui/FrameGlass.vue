<script setup lang="ts">
import { useEventListener, useMutationObserver, useResizeObserver } from '@vueuse/core'
import { onMounted, ref } from 'vue'

/**
 * One frosted layer over the whole window with a hole cut out for the page, so the title bar,
 * navigation rail and right panel share a single seamless glass surface.
 */
const clipPath = ref<string>()
const outline = ref<Record<string, string>>({})
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
	const hasRightPanel = right < width - 1
	const rightRadius = hasRightPanel ? radius : 0
	document.body.style.setProperty('--page-radius-right', `${rightRadius}px`)
	outline.value = {
		left: `${left}px`,
		top: `${top}px`,
		width: `${right - left}px`,
		height: `${bottom - top}px`,
		borderTopLeftRadius: `${radius}px`,
		borderTopRightRadius: `${rightRadius}px`,
		borderRightStyle: hasRightPanel ? 'solid' : 'none',
	}
	clipPath.value =
		`path(evenodd, 'M0 0H${width}V${height}H0Z ` +
		`M${left + radius} ${top}H${right - rightRadius}` +
		`A${rightRadius} ${rightRadius} 0 0 1 ${right} ${top + rightRadius}` +
		`V${bottom}H${left}V${top + radius}` +
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
	<div class="frame-outline" :style="outline" aria-hidden="true" />
</template>

<style scoped>
.frame-outline {
	position: absolute;
	box-sizing: border-box;
	border: 1px solid color-mix(in srgb, #fff 12%, transparent);
	border-bottom: none;
	box-shadow:
		inset 0 10px 18px -10px rgb(0 0 0 / 0.4),
		inset 10px 0 18px -10px rgb(0 0 0 / 0.4),
		inset -10px 0 18px -10px rgb(0 0 0 / 0.4);
}

.frame-glass {
	position: absolute;
	inset: 0;
	background: var(--frame-tint);
}
</style>
