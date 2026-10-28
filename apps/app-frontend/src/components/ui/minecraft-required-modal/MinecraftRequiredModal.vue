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

		<div class="flex flex-col gap-6 px-6 pb-6">
			<div class="flex gap-2">
				<input
					v-model="offlineName"
					type="text"
					maxlength="16"
					class="grow"
					:placeholder="formatMessage(messages.nicknamePlaceholder)"
					@keydown.enter="playOffline"
				/>
				<Button type="colored" color="brand" :disabled="loadingSignIn" @click="playOffline">
					{{ formatMessage(messages.playOffline) }}
				</Button>
			</div>
			<div class="grid grid-cols-1 gap-2">
				<Button :disabled="loadingSignIn" @click="signIn">
					<SpinnerIcon v-if="loadingSignIn" class="animate-spin" />
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
import { Button, defineMessages, injectNotificationManager, NewModal, useVIntl } from '@modrinth/ui'
import { ref } from 'vue'

import steveImage from '@/assets/steve-look-up-left.webp'
import { handleSevereError } from '@/composables/use-error.js'
import { injectMinecraftAccounts } from '@/features/minecraft-accounts/context'
import { trackEvent } from '@/helpers/analytics'
import { login as loginFlow, login_offline, set_default_user } from '@/helpers/auth.js'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const minecraftAccounts = injectMinecraftAccounts()

const messages = defineMessages({
	header: {
		id: 'minecraft-required.offline.header',
		defaultMessage: 'Нужен ник',
	},
	descriptionHeader: {
		id: 'minecraft-required.offline.description-header',
		defaultMessage: 'Введи ник, чтобы играть',
	},
	description: {
		id: 'minecraft-required.offline.description',
		defaultMessage:
			'Аккаунт не нужен: играй по нику. Если у тебя есть лицензия, можно войти через Microsoft.',
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
const loadingSignIn = ref(false)
const offlineName = ref('')

async function playOffline() {
	if (!offlineName.value.trim()) return
	loadingSignIn.value = true

	try {
		const loggedIn = await login_offline(offlineName.value)
		await set_default_user(loggedIn.profile.id)
		await minecraftAccounts.value?.refreshValues()
		offlineName.value = ''
		modal.value?.hide()
	} catch (error) {
		handleError(error)
	} finally {
		loadingSignIn.value = false
	}
}

function show() {
	modal.value?.show()
}

async function signIn() {
	loadingSignIn.value = true

	try {
		const loggedIn = await loginFlow()
		if (!loggedIn) return

		await set_default_user(loggedIn.profile.id)
		await minecraftAccounts.value?.refreshValues()
		await trackEvent('AccountLogIn', { source: 'MinecraftRequiredModal' })
		modal.value?.hide()
	} catch (error) {
		handleSevereError(error)
	} finally {
		loadingSignIn.value = false
	}
}

defineExpose({
	show,
})
</script>
