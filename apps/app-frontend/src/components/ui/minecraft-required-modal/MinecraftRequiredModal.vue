<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="544px" no-padding>
		<div class="grid grid-cols-[1fr_auto] gap-2.5 h-[154px] px-7 pt-4 pb-1 pr-9">
			<div class="flex flex-col gap-2.5 items-start justify-center h-min mt-5">
				<div class="font-semibold text-xl text-contrast">
					{{ formatMessage(messages.descriptionHeader) }}
				</div>
				<div class="text-secondary leading-6">
					{{ formatMessage(messages.description) }}
				</div>
			</div>
			<div class="relative h-full w-[96px] overflow-hidden mx-3">
				<div class="absolute top-0 left-0 z-0 w-full flex grow-0 flex-col items-end p-0">
					<img :src="steveImage" alt="" class="self-stretch" />
				</div>
				<div
					class="absolute left-0 bottom-0 z-10 order-1 h-6 w-[120px] shrink-0 grow-0 bg-[linear-gradient(180deg,rgba(39,41,46,0)_0%,#27292E_80%,#27292E_100%)]"
				></div>
			</div>
		</div>

		<div class="flex flex-col gap-4 px-6 pb-6">
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.offlineTitle) }}</span>
				<div class="flex gap-2">
					<Input
						v-model="offlineName"
						:maxlength="16"
						class="grow"
						:placeholder="formatMessage(messages.nicknamePlaceholder)"
						@keydown.enter="playOffline"
					/>
					<Button type="colored" color="brand" :disabled="loading" @click="playOffline">
						{{ formatMessage(messages.playOffline) }}
					</Button>
				</div>
			</div>

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.elybyTitle) }}</span>
				<ElybyLoginForm :disabled="loading" @logged-in="onLoggedIn" />
			</div>

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{
					formatMessage(messages.microsoftTitle)
				}}</span>
				<Button :disabled="loading" @click="signIn">
					<SpinnerIcon v-if="loadingMicrosoft" class="animate-spin" />
					<svg
						v-else
						width="20"
						height="20"
						viewBox="0 0 20 20"
						fill="none"
						xmlns="http://www.w3.org/2000/svg"
					>
						<rect width="9.25" height="9.25" fill="currentColor" fill-opacity="0.9" />
						<rect x="10.75" width="9.25" height="9.25" fill="currentColor" fill-opacity="0.9" />
						<rect y="10.75" width="9.25" height="9.25" fill="currentColor" fill-opacity="0.9" />
						<rect
							x="10.75"
							y="10.75"
							width="9.25"
							height="9.25"
							fill="currentColor"
							fill-opacity="0.9"
						/>
					</svg>
					{{ formatMessage(messages.signIn) }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectNotificationManager,
	Input,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import steveImage from '@/assets/steve-look-up-left.webp'
import { handleSevereError } from '@/composables/use-error.js'
import { injectMinecraftAccounts } from '@/features/minecraft-accounts/context'
import ElybyLoginForm from '@/features/minecraft-accounts/elyby-login-form.vue'
import { trackEvent } from '@/helpers/analytics'
import { login as loginFlow, login_offline, set_default_user } from '@/helpers/auth.js'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const minecraftAccounts = injectMinecraftAccounts()

const messages = defineMessages({
	header: {
		id: 'minecraft-required.mimiko.header',
		defaultMessage: 'Вход в игру',
	},
	descriptionHeader: {
		id: 'minecraft-required.mimiko.description-header',
		defaultMessage: 'Выбери, как играть',
	},
	description: {
		id: 'minecraft-required.mimiko.description',
		defaultMessage: 'Лицензия не нужна: играй по нику или войди через Ely.by со своим скином.',
	},
	offlineTitle: {
		id: 'minecraft-required.mimiko.offline-title',
		defaultMessage: 'По нику',
	},
	elybyTitle: {
		id: 'minecraft-required.mimiko.elyby-title',
		defaultMessage: 'Ely.by',
	},
	microsoftTitle: {
		id: 'minecraft-required.mimiko.microsoft-title',
		defaultMessage: 'Лицензия Microsoft',
	},
	nicknamePlaceholder: {
		id: 'minecraft-account.offline.nickname-placeholder',
		defaultMessage: 'Ник',
	},
	playOffline: {
		id: 'minecraft-account.offline.play',
		defaultMessage: 'Играть по нику',
	},
	signIn: {
		id: 'minecraft-required.sign-in',
		defaultMessage: 'Sign in to Microsoft',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const loadingOffline = ref(false)
const loadingMicrosoft = ref(false)
const loading = computed(() => loadingOffline.value || loadingMicrosoft.value)
const offlineName = ref('')

async function onLoggedIn(credentials: { profile: { id: string } }) {
	await set_default_user(credentials.profile.id)
	await minecraftAccounts.value?.refreshValues()
	modal.value?.hide()
}

async function playOffline() {
	if (!offlineName.value.trim()) return
	loadingOffline.value = true

	try {
		const loggedIn = await login_offline(offlineName.value)
		offlineName.value = ''
		await onLoggedIn(loggedIn)
	} catch (error) {
		handleError(error)
	} finally {
		loadingOffline.value = false
	}
}

function show() {
	modal.value?.show()
}

async function signIn() {
	loadingMicrosoft.value = true

	try {
		const loggedIn = await loginFlow()
		if (!loggedIn) return

		await onLoggedIn(loggedIn)
		await trackEvent('AccountLogIn', { source: 'MinecraftRequiredModal' })
	} catch (error) {
		handleSevereError(error)
	} finally {
		loadingMicrosoft.value = false
	}
}

defineExpose({
	show,
})
</script>
