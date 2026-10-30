<script setup lang="ts">
import { useDocumentVisibility, useElementSize, useWindowFocus } from '@vueuse/core'
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

/**
 * Draws a Minecraft panorama with WebGL 2. Blur comes from sampling the face mipmaps, so no CSS filter
 * runs over the page, and the canvas resolution and frame rate drop with the blur and adapt to how fast
 * the GPU keeps up. Rotation stops while the launcher is hidden or out of focus, such as during a game.
 */
const props = defineProps<{
	faces: string[]
	fieldOfView: number
	blur: number
	rotate: boolean
}>()

const emit = defineEmits<{ ready: [] }>()

const ROTATION_PERIOD_MS = 240_000
const PITCH_DEGREES = 8
const FACE_SIZE = 1024
const FPS_TIERS = [30, 20, 12]
const SLOW_FRAME_RATIO = 1.6
const SOFTWARE_RENDERER = /swiftshader|llvmpipe|basic render|software/i

const VERTEX_SHADER = `#version 300 es
in vec2 position;
out vec2 screen;
void main() {
	screen = position;
	gl_Position = vec4(position, 0.0, 1.0);
}
`

const FRAGMENT_SHADER = `#version 300 es
precision highp float;
in vec2 screen;
out vec4 color;
uniform vec2 halfExtent;
uniform float distance;
uniform float yaw;
uniform float pitch;
uniform float lod;
uniform float spread;
uniform sampler2D faces[6];

vec3 face(int index, vec2 uv) {
	vec2 coord = uv * 0.5 + 0.5;
	if (index == 0) return textureLod(faces[0], coord, lod).rgb;
	if (index == 1) return textureLod(faces[1], coord, lod).rgb;
	if (index == 2) return textureLod(faces[2], coord, lod).rgb;
	if (index == 3) return textureLod(faces[3], coord, lod).rgb;
	if (index == 4) return textureLod(faces[4], coord, lod).rgb;
	return textureLod(faces[5], coord, lod).rgb;
}

vec3 sampleSky(vec3 d) {
	vec3 a = abs(d);
	if (a.z >= a.x && a.z >= a.y) {
		return d.z < 0.0 ? face(0, d.xy / -d.z) : face(2, vec2(-d.x, d.y) / d.z);
	}
	if (a.x >= a.y) {
		return d.x > 0.0 ? face(1, d.zy / d.x) : face(3, vec2(d.z, -d.y) / d.x);
	}
	return d.y < 0.0 ? face(4, vec2(-d.x, d.z) / d.y) : face(5, d.xz / d.y);
}

vec3 direction(vec2 offset) {
	vec3 view = vec3(screen.x * halfExtent.x + offset.x, -screen.y * halfExtent.y + offset.y, -distance);
	float cp = cos(pitch);
	float sp = sin(pitch);
	view = vec3(view.x, view.y * cp - view.z * sp, view.y * sp + view.z * cp);
	float cy = cos(yaw);
	float sy = sin(yaw);
	return vec3(view.x * cy + view.z * sy, view.y, -view.x * sy + view.z * cy);
}

void main() {
	if (spread <= 0.0) {
		color = vec4(sampleSky(direction(vec2(0.0))), 1.0);
		return;
	}
	vec3 sum = sampleSky(direction(vec2(spread, spread)));
	sum += sampleSky(direction(vec2(-spread, spread)));
	sum += sampleSky(direction(vec2(spread, -spread)));
	sum += sampleSky(direction(vec2(-spread, -spread)));
	color = vec4(sum * 0.25, 1.0);
}
`

const canvas = ref<HTMLCanvasElement>()
const { width, height } = useElementSize(canvas)
const visibility = useDocumentVisibility()
const focused = useWindowFocus()
const fallback = ref(false)

let gl: WebGL2RenderingContext | null = null
let uniforms: Record<string, WebGLUniformLocation | null> = {}
let textures: WebGLTexture[] = []
let frame = 0
let lastDraw = 0
let lastTime = 0
let yaw = 0
let texturesReady = false
let loadRequest = 0
let fpsTier = 0
let slowFrames = 0
let staticOnly = false

function distance() {
	const halfFov = (props.fieldOfView * Math.PI) / 360
	return Math.max(width.value, height.value * 1.6) / 2 / Math.tan(halfFov)
}

function renderScale() {
	if (props.blur > 0) return Math.max(0.12, 1 / (1 + props.blur / 3))
	return Math.min(window.devicePixelRatio || 1, fpsTier > 0 ? 1 : 1.5)
}

function compile(type: number, source: string) {
	const shader = gl!.createShader(type)!
	gl!.shaderSource(shader, source)
	gl!.compileShader(shader)
	return shader
}

function setup() {
	gl = canvas.value!.getContext('webgl2', {
		alpha: false,
		antialias: false,
		depth: false,
		stencil: false,
		powerPreference: 'low-power',
		preserveDrawingBuffer: false,
	})
	if (!gl) {
		fallback.value = true
		return
	}
	const debug = gl.getExtension('WEBGL_debug_renderer_info')
	const renderer = debug ? String(gl.getParameter(debug.UNMASKED_RENDERER_WEBGL)) : ''
	staticOnly = SOFTWARE_RENDERER.test(renderer)
	const program = gl.createProgram()!
	gl.attachShader(program, compile(gl.VERTEX_SHADER, VERTEX_SHADER))
	gl.attachShader(program, compile(gl.FRAGMENT_SHADER, FRAGMENT_SHADER))
	gl.linkProgram(program)
	if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
		fallback.value = true
		gl = null
		return
	}
	gl.useProgram(program)
	const buffer = gl.createBuffer()
	gl.bindBuffer(gl.ARRAY_BUFFER, buffer)
	gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW)
	const position = gl.getAttribLocation(program, 'position')
	gl.enableVertexAttribArray(position)
	gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0)
	for (const name of ['halfExtent', 'distance', 'yaw', 'pitch', 'lod', 'spread']) {
		uniforms[name] = gl.getUniformLocation(program, name)
	}
	gl.uniform1iv(gl.getUniformLocation(program, 'faces'), [0, 1, 2, 3, 4, 5])
	gl.uniform1f(uniforms.pitch, (PITCH_DEGREES * Math.PI) / 180)
}

function loadImage(url: string) {
	return new Promise<HTMLImageElement>((resolve, reject) => {
		const image = new Image()
		image.crossOrigin = 'anonymous'
		image.decoding = 'async'
		image.onload = () => resolve(image)
		image.onerror = reject
		image.src = url
	})
}

async function loadFaces() {
	const current = ++loadRequest
	texturesReady = false
	if (!gl) {
		emit('ready')
		return
	}
	const images = await Promise.all(props.faces.map(loadImage)).catch(() => null)
	if (current !== loadRequest || !gl || !images) return
	for (const texture of textures) gl.deleteTexture(texture)
	textures = images.map((image, index) => {
		const texture = gl!.createTexture()!
		gl!.activeTexture(gl!.TEXTURE0 + index)
		gl!.bindTexture(gl!.TEXTURE_2D, texture)
		gl!.texImage2D(gl!.TEXTURE_2D, 0, gl!.RGB8, gl!.RGB, gl!.UNSIGNED_BYTE, image)
		gl!.generateMipmap(gl!.TEXTURE_2D)
		gl!.texParameteri(gl!.TEXTURE_2D, gl!.TEXTURE_MIN_FILTER, gl!.LINEAR_MIPMAP_LINEAR)
		gl!.texParameteri(gl!.TEXTURE_2D, gl!.TEXTURE_MAG_FILTER, gl!.LINEAR)
		gl!.texParameteri(gl!.TEXTURE_2D, gl!.TEXTURE_WRAP_S, gl!.CLAMP_TO_EDGE)
		gl!.texParameteri(gl!.TEXTURE_2D, gl!.TEXTURE_WRAP_T, gl!.CLAMP_TO_EDGE)
		return texture
	})
	texturesReady = true
	draw()
	emit('ready')
	startLoop()
}

function draw() {
	if (!gl || !texturesReady || !canvas.value || !width.value || !height.value) return
	const scale = renderScale()
	const pixelWidth = Math.max(1, Math.round(width.value * scale))
	const pixelHeight = Math.max(1, Math.round(height.value * scale))
	if (canvas.value.width !== pixelWidth || canvas.value.height !== pixelHeight) {
		canvas.value.width = pixelWidth
		canvas.value.height = pixelHeight
	}
	gl.viewport(0, 0, pixelWidth, pixelHeight)
	const viewDistance = distance()
	const texelsPerPixel = FACE_SIZE / 2 / viewDistance
	const blurTexels = props.blur * texelsPerPixel
	gl.uniform2f(uniforms.halfExtent, width.value / 2, height.value / 2)
	gl.uniform1f(uniforms.distance, viewDistance)
	gl.uniform1f(uniforms.yaw, yaw)
	gl.uniform1f(uniforms.lod, blurTexels > 1 ? Math.log2(blurTexels) : 0)
	gl.uniform1f(uniforms.spread, props.blur * 0.5)
	gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4)
}

function shouldAnimate() {
	return (
		props.rotate &&
		!staticOnly &&
		visibility.value !== 'hidden' &&
		focused.value &&
		!window.matchMedia('(prefers-reduced-motion: reduce)').matches
	)
}

/** Drops to a lower frame rate when frames keep arriving late, which means the GPU is struggling. */
function adapt(delta: number, interval: number) {
	if (fpsTier >= FPS_TIERS.length - 1) return
	slowFrames = delta > interval * SLOW_FRAME_RATIO ? slowFrames + 1 : Math.max(0, slowFrames - 1)
	if (slowFrames > 8) {
		fpsTier++
		slowFrames = 0
	}
}

function tick(time: number) {
	frame = 0
	if (!shouldAnimate()) {
		lastTime = 0
		return
	}
	const fps = FPS_TIERS[Math.min(FPS_TIERS.length - 1, fpsTier + (props.blur > 0 ? 1 : 0))]
	const interval = 1000 / fps
	if (lastTime) yaw = (yaw + ((time - lastTime) / ROTATION_PERIOD_MS) * Math.PI * 2) % (Math.PI * 2)
	lastTime = time
	if (time - lastDraw >= interval - 1) {
		if (lastDraw) adapt(time - lastDraw, interval)
		lastDraw = time
		draw()
	}
	frame = requestAnimationFrame(tick)
}

function startLoop() {
	if (frame || !texturesReady || !shouldAnimate()) return
	lastDraw = 0
	frame = requestAnimationFrame(tick)
}

onMounted(() => {
	setup()
	loadFaces()
})

onBeforeUnmount(() => {
	cancelAnimationFrame(frame)
	loadRequest++
	gl?.getExtension('WEBGL_lose_context')?.loseContext()
	gl = null
})

watch(() => props.faces.join('|'), loadFaces)
watch([width, height, () => props.blur, () => props.fieldOfView], draw)
watch([() => props.rotate, visibility, focused], startLoop)
</script>

<template>
	<canvas
		ref="canvas"
		class="panorama-canvas"
		:style="
			fallback
				? {
						backgroundImage: `url(${faces[0]})`,
						filter: blur > 0 ? `blur(${blur}px)` : undefined,
					}
				: undefined
		"
	/>
</template>

<style scoped>
.panorama-canvas {
	display: block;
	width: 100%;
	height: 100%;
	background-size: cover;
	background-position: center;
}
</style>
