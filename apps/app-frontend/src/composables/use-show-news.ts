import { ref, watch } from 'vue'

const STORAGE_KEY = 'mimiko-show-news'

function readStored(): boolean {
	try {
		return window.localStorage.getItem(STORAGE_KEY) !== 'false'
	} catch {
		return true
	}
}

const showNews = ref(readStored())

watch(showNews, (value) => {
	try {
		window.localStorage.setItem(STORAGE_KEY, String(value))
	} catch {
		// storage blocked or full
	}
})

/** Whether the right panel shows Minecraft news. */
export function useShowNews() {
	return showNews
}
