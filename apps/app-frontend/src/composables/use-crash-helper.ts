import { analyseGameLog } from '@/helpers/crash-advice'
import { get_live_log_buffer } from '@/helpers/logs.js'

import { useAppEvent } from './use-app-event'

/** How many of the last log lines are checked for known failures. */
const LOG_TAIL_LINES = 600

interface CrashHelperOptions {
	/** Shows a popup notification, as given by the app's popup notification manager. */
	notify: (notification: Record<string, unknown>) => void
	/** Opens the logs of an instance. */
	openLogs: (instanceId: string) => void
	/** The app event emitter, required when used from the root component. */
	appEvents?: Parameters<typeof useAppEvent>[2]
}

type LogLine = string | { text?: string }

/**
 * Explains in plain language why the game stopped, when the end of its log matches a known
 * failure. Nothing is shown for a normal exit.
 */
export function useCrashHelper({ notify, openLogs, appEvents }: CrashHelperOptions) {
	useAppEvent(
		'process',
		async (event) => {
			if (event.event !== 'finished') return

			let lines: LogLine[]
			try {
				lines = (await get_live_log_buffer(event.instance_id)) ?? []
			} catch {
				return
			}

			const tail = lines
				.slice(-LOG_TAIL_LINES)
				.map((line) => (typeof line === 'string' ? line : (line?.text ?? '')))
			const advice = analyseGameLog(tail)
			if (!advice) return

			notify({
				contentType: 'standard',
				title: advice.title,
				text: advice.advice,
				type: 'error',
				autoCloseMs: null,
				buttons: [
					{
						label: 'Открыть логи',
						action: () => openLogs(event.instance_id),
						color: 'brand',
					},
				],
			})
		},
		appEvents,
	)
}
