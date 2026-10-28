export interface CrashAdvice {
	title: string
	advice: string
}

interface CrashRule {
	pattern: RegExp
	advice: CrashAdvice
}

/**
 * Known failure signatures in the game log, most specific first. Each one maps to a plain-language
 * explanation of what went wrong and what to do about it.
 */
const CRASH_RULES: CrashRule[] = [
	{
		pattern:
			/OutOfMemoryError|Java heap space|GC overhead limit exceeded|Could not reserve enough space|There is insufficient memory/i,
		advice: {
			title: 'Не хватило памяти',
			advice:
				'Игре не хватило оперативной памяти. Открой настройки сборки, вкладка «Java и память», и увеличь выделенную память.',
		},
	},
	{
		pattern:
			/UnsupportedClassVersionError|Unsupported class file major version|compiled by a more recent version of the Java Runtime/i,
		advice: {
			title: 'Не подходит версия Java',
			advice:
				'Эта версия игры или мода требует другую Java. В настройках сборки, вкладка «Java и память», оставь автоматический выбор: лаунчер сам скачает нужную.',
		},
	},
	{
		pattern:
			/Missing or unsupported mandatory dependencies|Incompatible mod set|Mod resolution failed|requires (any version of|version)|depends on mod/i,
		advice: {
			title: 'Модам не хватает зависимостей',
			advice:
				'Какому-то моду нужен другой мод или другая версия. Чаще всего не хватает Fabric API. Проверь вкладку «Контент» и установи недостающее.',
		},
	},
	{
		pattern: /MixinApplyError|Mixin apply failed|InvalidMixinException|Mixin transformation/i,
		advice: {
			title: 'Моды конфликтуют',
			advice:
				'Два мода не уживаются друг с другом или мод не подходит к этой версии игры. Отключи последние добавленные моды и запусти снова.',
		},
	},
	{
		pattern: /NoClassDefFoundError|ClassNotFoundException|NoSuchMethodError|NoSuchFieldError/i,
		advice: {
			title: 'Мод не подходит к версии игры',
			advice:
				'Не нашлась часть кода, на которую рассчитывает мод. Обычно мод собран под другую версию игры или загрузчика. Обнови его или отключи.',
		},
	},
	{
		pattern:
			/GLFW error|Pixel format not accepted|failed to create (the )?window|OpenGL|org\.lwjgl\.LWJGLException/i,
		advice: {
			title: 'Проблема с видеодрайвером',
			advice:
				'Игре не удалось создать окно или включить графику. Обнови драйвер видеокарты и проверь, что игра запускается не на встроенном видео.',
		},
	},
	{
		pattern: /Failed to verify username|Invalid session|Not authenticated with Minecraft\.net/i,
		advice: {
			title: 'Сервер проверяет лицензию',
			advice:
				'Сервер с включённой проверкой не пускает без лицензии. Зайди через аккаунт Ely.by (если сервер его принимает) или на сервер с отключённой проверкой.',
		},
	},
	{
		pattern: /---- Minecraft Crash Report ----|Crash report saved to|Game crashed/i,
		advice: {
			title: 'Игра вылетела',
			advice:
				'Игра завершилась с ошибкой. Подробности лежат на вкладке «Логи» этой сборки. Если вылеты начались после установки мода, отключи его.',
		},
	},
]

/**
 * Looks for a known failure in the end of a game log and returns advice for it, or null when the
 * log does not look like a failure.
 */
export function analyseGameLog(lines: string[]): CrashAdvice | null {
	const text = lines.join('\n')

	for (const rule of CRASH_RULES) {
		if (rule.pattern.test(text)) return rule.advice
	}

	return null
}
