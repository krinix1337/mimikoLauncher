<script setup lang="ts">
import { DownloadIcon, ExternalIcon, GithubIcon, RefreshCwIcon, RotateCounterClockwiseIcon } from '@modrinth/assets'
import { Button, defineMessages, ProgressBar, Toggle, useFormatBytes, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed } from 'vue'

import {
	appUpdateState,
	checkForAppUpdate,
	formatAppVersion,
	downloadAvailableAppUpdate,
	installAvailableAppUpdate,
	openAppUpdateChangelog,
	setAppUpdateAutoCheck,
} from '@/providers/app-update.ts'

const LAUNCHER_REPOSITORY_URL = 'https://github.com/krinix1337/mimikoLauncher'

const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()

const {
	availableUpdate,
	checkError,
	checkStatus,
	downloading,
	downloadPercent,
	downloadProgress,
	finishedDownloading,
	lastCheckedAt,
	restarting,
	updateSize,
	updatesEnabled,
	autoCheckEnabled,
} = appUpdateState

const { data: currentVersion } = useQuery({
	queryKey: ['app-info', 'display-version'],
	queryFn: async () => formatAppVersion(await getVersion()),
	staleTime: Infinity,
})

const messages = defineMessages({
	title: {
		id: 'app.updates-settings.title',
		defaultMessage: 'Updates',
	},
	currentVersion: {
		id: 'app.updates-settings.current-version',
		defaultMessage: 'Current version: {version}',
	},
	checkButton: {
		id: 'app.updates-settings.check-button',
		defaultMessage: 'Check for updates',
	},
	checking: {
		id: 'app.updates-settings.status.checking',
		defaultMessage: 'Checking for updates...',
	},
	upToDate: {
		id: 'app.updates-settings.status.up-to-date',
		defaultMessage: 'You are using the latest version.',
	},
	available: {
		id: 'app.updates-settings.status.available',
		defaultMessage: 'Version {version} is available.',
	},
	error: {
		id: 'app.updates-settings.status.error',
		defaultMessage: 'Could not check for updates. Check your internet connection and try again.',
	},
	idle: {
		id: 'app.updates-settings.status.idle',
		defaultMessage: 'Updates have not been checked yet.',
	},
	disabled: {
		id: 'app.updates-settings.status.disabled',
		defaultMessage: 'Automatic updates are not available in this build.',
	},
	lastChecked: {
		id: 'app.updates-settings.last-checked',
		defaultMessage: 'Last checked: {time}',
	},
	downloading: {
		id: 'app.updates-settings.downloading',
		defaultMessage: 'Downloading update... {percent}%',
	},
	downloaded: {
		id: 'app.updates-settings.downloaded',
		defaultMessage: 'The update is downloaded and will be installed when you restart the app.',
	},
	downloadButton: {
		id: 'app.updates-settings.download-button',
		defaultMessage: 'Download ({size})',
	},
	installButton: {
		id: 'app.updates-settings.install-button',
		defaultMessage: 'Restart and install',
	},
	changelogButton: {
		id: 'app.updates-settings.changelog-button',
		defaultMessage: 'Release notes',
	},
	repositoryTitle: {
		id: 'app.updates-settings.repository.title',
		defaultMessage: 'GitHub',
	},
	repositoryDescription: {
		id: 'app.updates-settings.repository.description',
		defaultMessage: 'Source code, releases and bug reports for the launcher.',
	},
	repositoryButton: {
		id: 'app.updates-settings.repository.button',
		defaultMessage: 'Open on GitHub',
	},
	autoCheckTitle: {
		id: 'app.updates-settings.auto-check.title',
		defaultMessage: 'Check for updates on startup',
	},
	autoCheckDescription: {
		id: 'app.updates-settings.auto-check.description',
		defaultMessage:
			'Look for a new version when the app starts and periodically while it runs, and download it in the background.',
	},
})

const statusText = computed(() => {
	if (!updatesEnabled.value) return formatMessage(messages.disabled)
	switch (checkStatus.value) {
		case 'checking':
			return formatMessage(messages.checking)
		case 'error':
			return formatMessage(messages.error)
		case 'up-to-date':
			return formatMessage(messages.upToDate)
		case 'available':
			return formatMessage(messages.available, {
				version: availableUpdate.value ? formatAppVersion(availableUpdate.value.version) : '',
			})
		default:
			return availableUpdate.value
				? formatMessage(messages.available, {
						version: formatAppVersion(availableUpdate.value.version),
					})
				: formatMessage(messages.idle)
	}
})

const lastCheckedText = computed(() =>
	lastCheckedAt.value
		? formatMessage(messages.lastChecked, {
				time: new Date(lastCheckedAt.value).toLocaleTimeString(),
			})
		: null,
)

const autoCheck = computed({
	get: () => autoCheckEnabled.value,
	set: (enabled: boolean) => setAppUpdateAutoCheck(enabled),
})
</script>

<template>
	<section>
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(messages.title) }}
		</h2>
		<div class="mt-4 flex flex-col gap-6">
			<div class="flex flex-col gap-3 rounded-2xl bg-bg p-4">
				<div class="flex items-center justify-between gap-4">
					<div>
						<h3 class="m-0 text-lg font-semibold text-contrast">
							{{ formatMessage(messages.currentVersion, { version: currentVersion ?? '…' }) }}
						</h3>
						<p class="m-0 mt-1" :class="{ 'text-red': checkStatus === 'error' }">
							{{ statusText }}
						</p>
						<p v-if="checkStatus === 'error' && checkError" class="m-0 mt-1 text-sm text-secondary">
							{{ checkError }}
						</p>
						<p v-if="lastCheckedText" class="m-0 mt-1 text-sm text-secondary">
							{{ lastCheckedText }}
						</p>
					</div>
					<Button
						:disabled="!updatesEnabled || checkStatus === 'checking' || downloading"
						@click="checkForAppUpdate"
					>
						<RefreshCwIcon :class="{ 'animate-spin': checkStatus === 'checking' }" />
						{{ formatMessage(messages.checkButton) }}
					</Button>
				</div>

				<template v-if="availableUpdate && updatesEnabled">
					<div v-if="downloading" class="flex flex-col gap-2">
						<p class="m-0">
							{{ formatMessage(messages.downloading, { percent: downloadPercent }) }}
						</p>
						<ProgressBar :progress="downloadProgress" />
					</div>
					<p v-else-if="finishedDownloading" class="m-0">
						{{ formatMessage(messages.downloaded) }}
					</p>
					<div class="flex flex-wrap gap-2">
						<Button
							v-if="finishedDownloading"
							type="colored"
							color="brand"
							:disabled="restarting"
							@click="installAvailableAppUpdate"
						>
							<RotateCounterClockwiseIcon />
							{{ formatMessage(messages.installButton) }}
						</Button>
						<Button
							v-else-if="!downloading"
							type="colored"
							color="brand"
							@click="downloadAvailableAppUpdate"
						>
							<DownloadIcon />
							{{ formatMessage(messages.downloadButton, { size: formatBytes(updateSize ?? 0) }) }}
						</Button>
						<Button @click="openAppUpdateChangelog">
							{{ formatMessage(messages.changelogButton) }}
						</Button>
					</div>
				</template>
			</div>

			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 id="auto-check-updates-label" class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.autoCheckTitle) }}
					</h3>
					<p id="auto-check-updates-description" class="m-0 mt-1">
						{{ formatMessage(messages.autoCheckDescription) }}
					</p>
				</div>
				<Toggle
					id="auto-check-updates"
					v-model="autoCheck"
					:disabled="!updatesEnabled"
					aria-labelledby="auto-check-updates-label"
					aria-describedby="auto-check-updates-description"
				/>
			</div>

			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.repositoryTitle) }}
					</h3>
					<p class="m-0 mt-1">
						{{ formatMessage(messages.repositoryDescription) }}
					</p>
				</div>
				<Button @click="openUrl(LAUNCHER_REPOSITORY_URL)">
					<GithubIcon />
					{{ formatMessage(messages.repositoryButton) }}
					<ExternalIcon />
				</Button>
			</div>
		</div>
	</section>
</template>
