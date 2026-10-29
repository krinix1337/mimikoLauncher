<script setup lang="ts">
import { PlayIcon, StopCircleIcon } from '@modrinth/assets'
import { injectNotificationManager } from '@modrinth/ui'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import dayjs from 'dayjs'
import { onMounted, onScopeDispose, ref } from 'vue'
import { useRouter } from 'vue-router'

import { useAppEvent } from '@/composables/use-app-event'
import { get_many } from '@/helpers/instance'
import { get_all, kill } from '@/helpers/process'
import {
	get_recent_worlds,
	getWorldDisplayName,
	start_join_server,
	start_join_singleplayer_world,
	type WorldWithInstance,
} from '@/helpers/worlds'

const NEWS_URL = 'https://launchercontent.mojang.com/v2/news.json'
const NEWS_IMAGE_BASE = 'https://launchercontent.mojang.com'

interface RunningGame {
	uuid: string
	instanceId: string
	name: string
}

interface NewsEntry {
	id: string
	title: string
	date: string
	readMoreLink: string
	newsType?: string[]
	newsPageImage?: { url: string }
	playPageImage?: { url: string }
}

const { handleError } = injectNotificationManager()
const router = useRouter()
const running = ref<RunningGame[]>([])
const recent = ref<(WorldWithInstance & { instanceName: string })[]>([])
const news = ref<NewsEntry[]>([])

async function refreshRunning() {
	const processes = ((await get_all().catch(() => [])) ?? []) as Array<{
		uuid: string
		instance_id: string
	}>
	const instances = processes.length
		? await get_many(processes.map((process) => process.instance_id)).catch(() => [])
		: []
	running.value = processes.map((process) => ({
		uuid: process.uuid,
		instanceId: process.instance_id,
		name: instances.find((instance) => instance.id === process.instance_id)?.name ?? '',
	}))
}

async function refreshRecent() {
	const worlds = await get_recent_worlds(5).catch(() => [] as WorldWithInstance[])
	const instances = worlds.length
		? await get_many([...new Set(worlds.map((world) => world.instance_id))]).catch(() => [])
		: []
	recent.value = worlds.map((world) => ({
		...world,
		instanceName: instances.find((instance) => instance.id === world.instance_id)?.name ?? '',
	}))
}

async function loadNews() {
	try {
		const response = await tauriFetch(NEWS_URL)
		if (!response.ok) return
		const entries = ((await response.json()) as { entries: NewsEntry[] }).entries ?? []
		const java = entries.filter((entry) => entry.newsType?.includes('Java'))
		news.value = (java.length ? java : entries).slice(0, 4)
	} catch {
		// no network
	}
}

async function stopGame(uuid: string) {
	await kill(uuid).catch(handleError)
	await refreshRunning()
}

async function joinWorld(world: WorldWithInstance) {
	try {
		if (world.type === 'server') {
			await start_join_server(world.instance_id, world.address)
		} else {
			await start_join_singleplayer_world(world.instance_id, world.path)
		}
	} catch (error) {
		handleError(error)
	}
}

function openNews(url: string) {
	void router.push({ path: '/news', query: { url } })
}

function newsImage(entry: NewsEntry): string | null {
	const url = entry.newsPageImage?.url ?? entry.playPageImage?.url
	return url ? NEWS_IMAGE_BASE + url : null
}

useAppEvent('process', () => {
	void refreshRunning()
	void refreshRecent()
})

const pollId = window.setInterval(refreshRunning, 5000)
onScopeDispose(() => window.clearInterval(pollId))

onMounted(() => {
	void refreshRunning()
	void refreshRecent()
	void loadNews()
})
</script>

<template>
	<section v-if="running.length" class="panel-section">
		<h3 class="panel-title">Сейчас запущено</h3>
		<div v-for="game in running" :key="game.uuid" class="panel-row">
			<span class="running-dot" aria-hidden="true" />
			<span class="min-w-0 flex-1 truncate font-semibold text-contrast">{{ game.name }}</span>
			<button
				v-tooltip="'Остановить'"
				type="button"
				class="panel-icon-button text-red"
				aria-label="Остановить"
				@click="stopGame(game.uuid)"
			>
				<StopCircleIcon />
			</button>
		</div>
	</section>

	<section v-if="recent.length" class="panel-section">
		<h3 class="panel-title">Недавние миры и серверы</h3>
		<button
			v-for="world in recent"
			:key="`${world.instance_id}-${world.type === 'server' ? world.address : world.path}`"
			type="button"
			class="panel-row panel-row--button"
			@click="joinWorld(world)"
		>
			<img v-if="world.icon" :src="world.icon" alt="" class="world-icon" />
			<span v-else class="world-icon world-icon--empty" aria-hidden="true" />
			<span class="min-w-0 flex-1 text-left">
				<span class="block truncate font-semibold text-contrast">{{ getWorldDisplayName(world) }}</span>
				<span class="block truncate text-sm text-secondary">
					{{ world.type === 'server' ? 'Сервер' : 'Мир' }} · {{ world.instanceName }}
				</span>
			</span>
			<PlayIcon class="play-icon" aria-hidden="true" />
		</button>
	</section>

	<section v-if="news.length" class="panel-section">
		<h3 class="panel-title">Новости Minecraft</h3>
		<button
			v-for="entry in news"
			:key="entry.id"
			type="button"
			class="news-card"
			@click="openNews(entry.readMoreLink)"
		>
			<img v-if="newsImage(entry)" :src="newsImage(entry)!" alt="" class="news-image" />
			<span class="block px-3 pt-2 font-semibold text-contrast">{{ entry.title.trim() }}</span>
			<span class="block px-3 pb-2 text-sm text-secondary">{{ dayjs(entry.date).format('D.MM.YYYY') }}</span>
		</button>
	</section>
</template>

<style scoped>
.panel-section {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
	padding: 1rem;
	border-bottom: 1px solid var(--brand-gradient-border);
}

.panel-title {
	margin: 0 0 0.25rem;
	font-size: 1rem;
	font-weight: 500;
	color: var(--color-text-primary);
}

.panel-row {
	display: flex;
	align-items: center;
	gap: 0.625rem;
	min-width: 0;
	padding: 0.5rem;
	border: none;
	border-radius: var(--radius-lg);
	background: var(--color-button-bg);
	color: inherit;
	font: inherit;
}

.panel-row--button {
	cursor: pointer;
	transition: filter 0.15s ease;

	&:hover {
		filter: brightness(1.12);
	}

	&:hover .play-icon {
		opacity: 1;
	}
}

.running-dot {
	width: 0.5rem;
	height: 0.5rem;
	margin-left: 0.25rem;
	border-radius: 9999px;
	background: var(--color-green);
	box-shadow: 0 0 0 4px color-mix(in srgb, var(--color-green) 25%, transparent);
}

.panel-icon-button {
	display: flex;
	padding: 0.25rem;
	border: none;
	border-radius: 9999px;
	background: transparent;
	cursor: pointer;

	svg {
		width: 1.25rem;
		height: 1.25rem;
	}
}

.world-icon {
	width: 2.25rem;
	height: 2.25rem;
	flex-shrink: 0;
	border-radius: var(--radius-md);
	object-fit: cover;
	image-rendering: pixelated;
}

.world-icon--empty {
	background: var(--surface-5);
}

.play-icon {
	width: 1.25rem;
	height: 1.25rem;
	flex-shrink: 0;
	color: var(--color-brand);
	opacity: 0.5;
	transition: opacity 0.15s ease;
}

.news-card {
	display: block;
	overflow: hidden;
	padding: 0;
	border: none;
	border-radius: var(--radius-lg);
	background: var(--color-button-bg);
	color: inherit;
	font: inherit;
	text-align: left;
	cursor: pointer;
	transition: filter 0.15s ease;

	&:hover {
		filter: brightness(1.12);
	}
}

.news-image {
	display: block;
	width: 100%;
	aspect-ratio: 772 / 350;
	object-fit: cover;
}
</style>
