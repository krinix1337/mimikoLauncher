import { ref, watch } from 'vue'

const STORAGE_KEY = 'mimiko-nav-labels'

function readStored(): boolean {
	try {
		return window.localStorage.getItem(STORAGE_KEY) !== 'false'
	} catch {
		return true
	}
}

const showNavLabels = ref(readStored())

watch(showNavLabels, (value) => {
	try {
		window.localStorage.setItem(STORAGE_KEY, String(value))
	} catch {
		// storage blocked or full
	}
})

/** Whether the navigation rail shows text labels under its icons. */
export function useNavLabels() {
	return showNavLabels
}
