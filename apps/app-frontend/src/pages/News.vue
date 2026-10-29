<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

const SITE = 'https://www.minecraft.net'
const ALLOWED_TAGS = new Set([
	'P', 'H2', 'H3', 'H4', 'UL', 'OL', 'LI', 'A', 'B', 'STRONG', 'I', 'EM', 'BR', 'IMG', 'BLOCKQUOTE', 'SPAN',
])

const route = useRoute()

const loading = ref(true)
const failed = ref(false)
const title = ref('')
const subtitle = ref('')
const heroImage = ref<string | null>(null)
const bodyHtml = ref('')

const articleUrl = computed(() => {
	try {
		const url = new URL(String(route.query.url ?? ''))
		if (url.protocol !== 'https:' || !/(^|\.)minecraft\.net$/.test(url.hostname)) return null
		url.hostname = 'www.minecraft.net'
		url.search = ''
		if (!/^\/[a-z]{2}-[a-z]{2}\//.test(url.pathname)) url.pathname = '/en-us' + url.pathname
		return url.toString()
	} catch {
		return null
	}
})

function absolute(src: string | null): string | null {
	if (!src) return null
	try {
		return new URL(src, SITE).toString()
	} catch {
		return null
	}
}

/** Copies only safe text and image markup from the article, dropping scripts, styles and attributes. */
function sanitize(source: Element, target: Element) {
	for (const node of Array.from(source.childNodes)) {
		if (node.nodeType === Node.TEXT_NODE) {
			target.appendChild(document.createTextNode(node.textContent ?? ''))
			continue
		}
		if (!(node instanceof Element)) continue
		if (!ALLOWED_TAGS.has(node.tagName)) {
			if (!['SCRIPT', 'STYLE', 'IFRAME', 'NOSCRIPT', 'BUTTON', 'FORM'].includes(node.tagName)) {
				sanitize(node, target)
			}
			continue
		}
		const clean = document.createElement(node.tagName)
		if (node.tagName === 'A') {
			const href = absolute(node.getAttribute('href'))
			if (href?.startsWith('https://')) clean.setAttribute('href', href)
		}
		if (node.tagName === 'IMG') {
			const src = absolute(node.getAttribute('src') ?? node.getAttribute('data-src'))
			if (!src?.startsWith('https://')) continue
			clean.setAttribute('src', src)
			clean.setAttribute('alt', node.getAttribute('alt') ?? '')
			clean.setAttribute('loading', 'lazy')
		}
		sanitize(node, clean)
		target.appendChild(clean)
	}
}

async function load(url: string | null) {
	loading.value = true
	failed.value = false
	bodyHtml.value = ''
	if (!url) {
		failed.value = true
		loading.value = false
		return
	}
	try {
		const response = await tauriFetch(url)
		if (!response.ok) throw new Error(String(response.status))
		const page = new DOMParser().parseFromString(await response.text(), 'text/html')
		const hero = page.querySelector('.MC_articleHeroA')
		title.value = hero?.querySelector('h1')?.textContent?.trim() ?? page.title
		subtitle.value =
			hero?.querySelector('.MC_articleHeroA_header_subheadline')?.textContent?.trim() ?? ''
		heroImage.value = absolute(hero?.querySelector('img')?.getAttribute('src') ?? null)

		const body = document.createElement('div')
		for (const section of page.querySelectorAll(
			'.MC_articleGridA .MC_Link_Style_RichText, .MC_articleGridA picture img',
		)) {
			if (section.tagName === 'IMG') {
				const wrapper = document.createElement('p')
				wrapper.appendChild(section.cloneNode())
				sanitize(wrapper, body)
			} else {
				sanitize(section, body)
			}
		}
		bodyHtml.value = body.innerHTML
		if (!bodyHtml.value.trim()) failed.value = true
	} catch {
		failed.value = true
	} finally {
		loading.value = false
	}
}

function onBodyClick(event: MouseEvent) {
	const link = (event.target as HTMLElement).closest('a')
	if (!link?.href) return
	event.preventDefault()
	void openUrl(link.href)
}

watch(articleUrl, load, { immediate: true })
</script>

<template>
	<div class="mx-auto flex w-full max-w-[52rem] flex-col gap-4 p-6">
		<div v-if="loading" class="py-16 text-center text-secondary">Загружаем статью…</div>
		<template v-else>
			<img v-if="heroImage" :src="heroImage" alt="" class="news-hero" />
			<div class="flex items-start justify-between gap-4">
				<div class="min-w-0">
					<h1 class="m-0 text-3xl font-bold text-contrast">{{ title }}</h1>
					<p v-if="subtitle" class="m-0 mt-2 text-lg text-secondary">{{ subtitle }}</p>
				</div>
				<ButtonStyled v-if="articleUrl">
					<button class="shrink-0" @click="openUrl(articleUrl)">
						<ExternalIcon />
						В браузере
					</button>
				</ButtonStyled>
			</div>
			<p v-if="failed" class="text-secondary">
				Не удалось показать статью в лаунчере. Открой её в браузере.
			</p>
			<!-- eslint-disable-next-line vue/no-v-html -- sanitized to a fixed set of text and image tags -->
			<article v-else class="news-body" @click="onBodyClick" v-html="bodyHtml" />
		</template>
	</div>
</template>

<style scoped>
.news-hero {
	width: 100%;
	aspect-ratio: 5 / 2;
	object-fit: cover;
	border-radius: var(--radius-xl);
}

.news-body {
	color: var(--color-text-default);
	font-size: 1rem;
	line-height: 1.6;

	:deep(h2),
	:deep(h3),
	:deep(h4) {
		margin: 1.5rem 0 0.5rem;
		color: var(--color-contrast);
	}

	:deep(p) {
		margin: 0 0 1rem;
	}

	:deep(img) {
		display: block;
		max-width: 100%;
		margin: 1rem auto;
		border-radius: var(--radius-lg);
	}

	:deep(a) {
		color: var(--color-brand);
	}

	:deep(blockquote) {
		margin: 1rem 0;
		padding-left: 1rem;
		border-left: 4px solid var(--color-brand);
	}
}
</style>
