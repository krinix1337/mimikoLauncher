<template>
	<Button v-if="!expanded" class="w-full" :disabled="disabled" @click="expanded = true">
		<LogInIcon />
		{{ formatMessage(messages.signInElyby) }}
	</Button>
	<div v-else class="flex flex-col gap-2">
		<Input
			v-model="username"
			autocomplete="username"
			:placeholder="formatMessage(messages.usernamePlaceholder)"
			@keydown.enter="submit()"
		/>
		<Input
			v-model="password"
			type="password"
			autocomplete="current-password"
			:placeholder="formatMessage(messages.passwordPlaceholder)"
			@keydown.enter="submit()"
		/>
		<div class="flex gap-2">
			<Button
				type="colored"
				color="brand"
				class="grow"
				:disabled="disabled || loading"
				@click="submit()"
			>
				<SpinnerIcon v-if="loading" class="animate-spin" />
				<LogInIcon v-else />
				{{ formatMessage(messages.signIn) }}
			</Button>
			<Button :disabled="loading" @click="cancel()">
				{{ formatMessage(messages.cancel) }}
			</Button>
		</div>
	</div>
</template>

<script setup lang="ts">
import { LogInIcon, SpinnerIcon } from '@modrinth/assets'
import { Button, defineMessages, injectNotificationManager, Input, useVIntl } from '@modrinth/ui'
import { ref } from 'vue'

import { login_elyby } from '@/helpers/auth'

defineProps<{
	disabled?: boolean
}>()

const emit = defineEmits<{
	'logged-in': [credentials: { profile: { id: string; name: string } }]
}>()

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const expanded = ref(false)
const loading = ref(false)
const username = ref('')
const password = ref('')

async function submit() {
	if (!username.value.trim() || !password.value) return
	loading.value = true

	try {
		const credentials = await login_elyby(username.value, password.value)
		cancel()
		emit('logged-in', credentials)
	} catch (error) {
		handleError(error)
	} finally {
		loading.value = false
	}
}

function cancel() {
	expanded.value = false
	username.value = ''
	password.value = ''
}

const messages = defineMessages({
	signInElyby: {
		id: 'minecraft-account.elyby.sign-in-button',
		defaultMessage: 'Войти через Ely.by',
	},
	usernamePlaceholder: {
		id: 'minecraft-account.elyby.username-placeholder',
		defaultMessage: 'Ник или почта Ely.by',
	},
	passwordPlaceholder: {
		id: 'minecraft-account.elyby.password-placeholder',
		defaultMessage: 'Пароль',
	},
	signIn: {
		id: 'minecraft-account.elyby.sign-in',
		defaultMessage: 'Войти',
	},
	cancel: {
		id: 'minecraft-account.elyby.cancel',
		defaultMessage: 'Отмена',
	},
})
</script>
