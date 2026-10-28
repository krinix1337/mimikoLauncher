import { getVersion } from '@tauri-apps/api/app'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { openUrl } from '@tauri-apps/plugin-opener'
import { onMounted } from 'vue'

/** GitHub repository whose releases are checked for new launcher versions. */
const RELEASES_URL = 'https://api.github.com/repos/krinix1337/mimikoLauncher/releases/latest'

/** Delay before checking, so the check does not compete with app startup. */
const CHECK_DELAY_MS = 8_000

const NOTIFIED_VERSION_KEY = 'mimiko-notified-update'

interface LatestRelease {
	tag_name: string
	html_url: string
	assets?: { name: string; browser_download_url: string }[]
}

interface UpdateCheckOptions {
	/** Shows a popup notification, as given by the app's popup notification manager. */
	notify: (notification: Record<string, unknown>) => void
}

/** Splits a version like `v1.2.3-local` into its numeric parts. */
function parseVersion(version: string): number[] {
	return version
		.replace(/^v/i, '')
		.split('-')[0]
		.split('.')
		.map((part) => Number.parseInt(part, 10) || 0)
}

function isNewer(candidate: string, current: string): boolean {
	const a = parseVersion(candidate)
	const b = parseVersion(current)

	for (let i = 0; i < Math.max(a.length, b.length); i++) {
		const difference = (a[i] ?? 0) - (b[i] ?? 0)
		if (difference !== 0) return difference > 0
	}

	return false
}

function readNotifiedVersion(): string | null {
	try {
		return window.localStorage.getItem(NOTIFIED_VERSION_KEY)
	} catch {
		return null
	}
}

function rememberNotifiedVersion(version: string) {
	try {
		window.localStorage.setItem(NOTIFIED_VERSION_KEY, version)
	} catch {
		// storage blocked or full
	}
}

/**
 * Checks GitHub for a newer launcher release once per start and offers to download it. Failures,
 * such as a missing release or no network, are ignored.
 */
export function useUpdateCheck({ notify }: UpdateCheckOptions) {
	onMounted(() => {
		window.setTimeout(async () => {
			try {
				const response = await tauriFetch(RELEASES_URL, {
					headers: { Accept: 'application/vnd.github+json' },
				})
				if (!response.ok) return

				const release = (await response.json()) as LatestRelease
				const current = await getVersion()
				if (!isNewer(release.tag_name, current)) return
				if (readNotifiedVersion() === release.tag_name) return

				const installer = release.assets?.find((asset) => asset.name.endsWith('-setup.exe'))
				notify({
					contentType: 'standard',
					title: `Вышла версия ${release.tag_name.replace(/^v/i, '')}`,
					text: 'Скачай новый установщик и поставь поверх текущего. Сборки и аккаунты сохранятся.',
					type: 'info',
					autoCloseMs: null,
					buttons: [
						{
							label: 'Скачать',
							action: () => openUrl(installer?.browser_download_url ?? release.html_url),
							color: 'brand',
						},
					],
				})
				rememberNotifiedVersion(release.tag_name)
			} catch {
				// no network or no published release
			}
		}, CHECK_DELAY_MS)
	})
}
