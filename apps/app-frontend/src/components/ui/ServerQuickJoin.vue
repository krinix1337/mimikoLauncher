<template>
	<div
		class="flex flex-col gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4"
	>
		<div v-if="!server || editing" class="flex flex-col gap-3">
			<div class="flex items-center gap-2 font-semibold text-contrast">
				<ServerIcon class="size-5" />
				{{ formatMessage(messages.title) }}
			</div>
			<p class="m-0 text-secondary">{{ formatMessage(messages.description) }}</p>
			<div class="grid gap-2 sm:grid-cols-2">
				<Input v-model="draft.name" :placeholder="formatMessage(messages.namePlaceholder)" />
				<Input
					v-model="draft.address"
					:placeholder="formatMessage(messages.addressPlaceholder)"
					@keydown.enter="save()"
				/>
			</div>
			<div class="flex flex-wrap items-center gap-2">
				<select
					v-model="draft.instanceId"
					class="h-9 rounded-xl border border-solid border-surface-5 bg-surface-4 px-3 text-contrast"
					:aria-label="formatMessage(messages.instanceLabel)"
				>
					<option v-for="instance in instances" :key="instance.path" :value="instance.path">
						{{ instance.name }}
					</option>
				</select>
				<Button type="colored" color="brand" @click="save()">
					<CheckIcon />
					{{ formatMessage(messages.save) }}
				</Button>
				<Button v-if="server" @click="editing = false">
					{{ formatMessage(messages.cancel) }}
				</Button>
			</div>
		</div>

		<div v-else class="flex flex-wrap items-center gap-4">
			<div class="flex min-w-0 grow items-center gap-3">
				<span
					class="size-3 shrink-0 rounded-full"
					:class="statusClass"
					:title="formatMessage(online ? messages.online : messages.offline)"
				></span>
				<div class="min-w-0">
					<div class="truncate text-lg font-semibold text-contrast">{{ server.name }}</div>
					<div class="truncate text-secondary">
						{{ server.address }}
						<template v-if="status">
							·
							{{
								formatMessage(messages.players, {
									online: status.players?.online ?? 0,
									max: status.players?.max ?? 0,
								})
							}}
							<template v-if="status.ping != null"> · {{ status.ping }} ms </template>
						</template>
						<template v-else-if="!refreshing">
							· {{ formatMessage(messages.offline) }}
						</template>
					</div>
				</div>
			</div>
			<div class="flex items-center gap-2">
				<Button type="colored" color="brand" size="lg" :disabled="joining" @click="join()">
					<SpinnerIcon v-if="joining" class="animate-spin" />
					<PlayIcon v-else />
					{{ formatMessage(messages.join) }}
				</Button>
				<IconButton
					:label="formatMessage(messages.edit)"
					@click="startEditing()"
				>
					<EditIcon />
				</IconButton>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon, EditIcon, PlayIcon, ServerIcon, SpinnerIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	IconButton,
	injectNotificationManager,
	Input,
	useVIntl,
} from '@modrinth/ui'
import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue'

import type { ServerStatus } from '@/helpers/worlds'
import { get_server_status, start_join_server } from '@/helpers/worlds'

type QuickServer = {
	name: string
	address: string
	instanceId: string
}

const props = defineProps<{
	instances: { path: string; name: string }[]
}>()

const STORAGE_KEY = 'mimiko-quick-server'
const REFRESH_INTERVAL_MS = 30_000

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const messages = defineMessages({
	title: { id: 'mimiko.quick-server.title', defaultMessage: 'Мой сервер' },
	description: {
		id: 'mimiko.quick-server.description',
		defaultMessage: 'Добавь адрес сервера, и сюда можно будет заходить одной кнопкой.',
	},
	namePlaceholder: { id: 'mimiko.quick-server.name', defaultMessage: 'Название' },
	addressPlaceholder: {
		id: 'mimiko.quick-server.address',
		defaultMessage: 'Адрес, например play.mimiko.fun',
	},
	instanceLabel: { id: 'mimiko.quick-server.instance', defaultMessage: 'Сборка для входа' },
	save: { id: 'mimiko.quick-server.save', defaultMessage: 'Сохранить' },
	cancel: { id: 'mimiko.quick-server.cancel', defaultMessage: 'Отмена' },
	edit: { id: 'mimiko.quick-server.edit', defaultMessage: 'Изменить сервер' },
	join: { id: 'mimiko.quick-server.join', defaultMessage: 'Зайти' },
	online: { id: 'mimiko.quick-server.online', defaultMessage: 'Онлайн' },
	offline: { id: 'mimiko.quick-server.offline', defaultMessage: 'Недоступен' },
	players: {
		id: 'mimiko.quick-server.players',
		defaultMessage: '{online} из {max} игроков',
	},
})

function load(): QuickServer | null {
	try {
		const raw = window.localStorage.getItem(STORAGE_KEY)
		return raw ? (JSON.parse(raw) as QuickServer) : null
	} catch {
		return null
	}
}

const server = ref<QuickServer | null>(load())
const editing = ref(false)
const draft = reactive<QuickServer>({
	name: server.value?.name ?? '',
	address: server.value?.address ?? '',
	instanceId: server.value?.instanceId ?? props.instances[0]?.path ?? '',
})
const status = ref<ServerStatus | null>(null)
const refreshing = ref(false)
const joining = ref(false)
let timer: number | undefined

const online = computed(() => status.value !== null)
const statusClass = computed(() =>
	online.value ? 'bg-green' : refreshing.value ? 'bg-orange' : 'bg-red',
)

function startEditing() {
	if (server.value) Object.assign(draft, server.value)
	editing.value = true
}

function save() {
	const address = draft.address.trim()
	if (!address || !draft.instanceId) return

	const next: QuickServer = {
		name: draft.name.trim() || address,
		address,
		instanceId: draft.instanceId,
	}
	server.value = next
	editing.value = false

	try {
		window.localStorage.setItem(STORAGE_KEY, JSON.stringify(next))
	} catch {
		// storage blocked or full
	}
	void refresh()
}

async function refresh() {
	if (!server.value) return

	refreshing.value = true
	try {
		status.value = await get_server_status(server.value.address)
	} catch {
		status.value = null
	} finally {
		refreshing.value = false
	}
}

async function join() {
	if (!server.value) return

	joining.value = true
	try {
		await start_join_server(server.value.instanceId, server.value.address)
	} catch (error) {
		handleError(error as Error)
	} finally {
		joining.value = false
	}
}

onMounted(() => {
	void refresh()
	timer = window.setInterval(refresh, REFRESH_INTERVAL_MS)
})

onBeforeUnmount(() => window.clearInterval(timer))
</script>
