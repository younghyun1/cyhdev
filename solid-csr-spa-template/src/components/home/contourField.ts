/**
 * Animated topographic contour field for the home hero, drawn by one WebGL2
 * fragment shader. It is loaded as a separate chunk only after the page is
 * idle (see HeroField.tsx), never touches layout, and degrades on its own:
 *
 * - renders at no more than 1.25 device pixels per CSS pixel and 30 frames/s;
 * - pauses while the canvas is off-screen or the document is hidden;
 * - stops animating and keeps its last frame if the browser cannot keep up;
 * - draws one still frame when animation is switched off or not wanted;
 * - follows the active theme and accent by re-reading CSS tokens whenever
 *   the root element's class or `data-accent` attribute changes.
 *
 * Passive movement has three layers: the terrain drifts and warps, the contour
 * levels rise so lines keep migrating across it, and a soft brightness swell
 * travels through the elevation bands.
 */

const VERTEX_SHADER = `#version 300 es
in vec2 a_position;
void main() {
  gl_Position = vec4(a_position, 0.0, 1.0);
}`;

const FRAGMENT_SHADER = `#version 300 es
precision highp float;
uniform vec2 u_resolution;
uniform float u_time;
uniform vec3 u_accent;
uniform vec3 u_ink;
uniform vec2 u_pointer;
uniform float u_pointer_strength;
out vec4 out_color;

float hash(vec2 p) {
  p = fract(p * vec2(123.34, 456.21));
  p += dot(p, p + 45.32);
  return fract(p.x * p.y);
}

float noise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  vec2 u = f * f * (3.0 - 2.0 * f);
  return mix(
    mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x),
    mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x),
    u.y
  );
}

float fbm(vec2 p) {
  float value = 0.0;
  float amplitude = 0.5;
  for (int i = 0; i < 4; i++) {
    value += amplitude * noise(p);
    p = p * 2.03 + vec2(1.7, 9.2);
    amplitude *= 0.5;
  }
  return value;
}

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution.y;
  vec2 p = uv * 2.4;
  float t = u_time * 0.075;
  vec2 drift = vec2(0.6 * t, -0.4 * t);
  vec2 warp = vec2(fbm(p + t), fbm(p + vec2(5.2, 1.3) - t));
  float warpAmount = 1.5 + 0.2 * sin(u_time * 0.16);
  float height = fbm(p + warpAmount * warp + drift);

  vec2 pointer = u_pointer / u_resolution.y;
  float d = distance(uv, pointer);
  height += 0.16 * u_pointer_strength * exp(-d * d * 16.0);

  // The rising offset is uniform across the frame, so fwidth and the line
  // width still follow only the terrain gradient.
  float bands = height * 15.0 + u_time * 0.14;
  float width = fwidth(bands);
  float minor = 1.0 - smoothstep(0.0, width * 1.4, min(fract(bands), 1.0 - fract(bands)));
  float majorBands = bands / 5.0;
  float major = 1.0 - smoothstep(0.0, width * 0.45, min(fract(majorBands), 1.0 - fract(majorBands)));
  float swell = 0.5 + 0.5 * sin(height * 21.0 - u_time * 0.55);
  float alpha = (minor * 0.14 + major * 0.28) * (0.7 + 0.6 * swell * swell);
  vec3 color = mix(u_ink, u_accent, smoothstep(0.38, 0.72, height));
  out_color = vec4(color * alpha, alpha);
}`;

const MAX_PIXEL_RATIO = 1.25;
const FRAME_INTERVAL_MS = 1000 / 30;
/** Browser frames to ignore while the shader compiles and the page settles. */
const PROBE_SKIP_FRAMES = 10;
const PROBE_FRAMES = 40;
/** A display callback slower than this on average means under 20 frames/s. */
const SLOW_CALLBACK_MS = 50;

type Rgb = readonly [number, number, number];

/**
 * Decides whether the device can afford continuous animation. It must see every
 * animation-frame callback, including those the 30 frames/s throttle skips: the
 * interval between drawn frames is at least 33 ms by construction, so it says
 * nothing about the device. Returns true once the callbacks prove too slow.
 */
export function createFrameProbe(): (callbackIntervalMs: number) => boolean {
  let seen = 0;
  let total = 0;
  return (callbackIntervalMs) => {
    seen += 1;
    if (seen <= PROBE_SKIP_FRAMES || seen > PROBE_SKIP_FRAMES + PROBE_FRAMES) return false;
    total += callbackIntervalMs;
    return seen === PROBE_SKIP_FRAMES + PROBE_FRAMES && total / PROBE_FRAMES > SLOW_CALLBACK_MS;
  };
}

export interface ContourField {
  /** Starts or stops continuous animation; stopping keeps the current frame. */
  readonly setAnimated: (animated: boolean) => void;
  readonly dispose: () => void;
}

function parseColor(value: string, fallback: Rgb): Rgb {
  const hex = value.trim().match(/^#([0-9a-f]{6})$/iu)?.[1];
  if (!hex) return fallback;
  return [
    Number.parseInt(hex.slice(0, 2), 16) / 255,
    Number.parseInt(hex.slice(2, 4), 16) / 255,
    Number.parseInt(hex.slice(4, 6), 16) / 255,
  ];
}

function compile(gl: WebGL2RenderingContext, type: number, source: string): WebGLShader | null {
  const shader = gl.createShader(type);
  if (!shader) return null;
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    gl.deleteShader(shader);
    return null;
  }
  return shader;
}

/** Starts the field on `canvas`; returns null if WebGL2 is unavailable. */
export function startContourField(
  canvas: HTMLCanvasElement,
  pointerTarget: HTMLElement,
  initiallyAnimated: boolean,
): ContourField | null {
  const gl = canvas.getContext("webgl2", {
    alpha: true,
    antialias: false,
    depth: false,
    stencil: false,
    premultipliedAlpha: true,
    powerPreference: "low-power",
  });
  if (!gl) return null;

  const vertex = compile(gl, gl.VERTEX_SHADER, VERTEX_SHADER);
  const fragment = compile(gl, gl.FRAGMENT_SHADER, FRAGMENT_SHADER);
  const program = gl.createProgram();
  if (!vertex || !fragment || !program) return null;
  gl.attachShader(program, vertex);
  gl.attachShader(program, fragment);
  gl.linkProgram(program);
  gl.deleteShader(vertex);
  gl.deleteShader(fragment);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    gl.deleteProgram(program);
    return null;
  }
  gl.useProgram(program);

  const buffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
  const position = gl.getAttribLocation(program, "a_position");
  gl.enableVertexAttribArray(position);
  gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0);

  const uniform = (name: string) => gl.getUniformLocation(program, name);
  const uResolution = uniform("u_resolution");
  const uTime = uniform("u_time");
  const uAccent = uniform("u_accent");
  const uInk = uniform("u_ink");
  const uPointer = uniform("u_pointer");
  const uPointerStrength = uniform("u_pointer_strength");

  const readColors = () => {
    const styles = getComputedStyle(document.documentElement);
    gl.uniform3fv(uAccent, parseColor(styles.getPropertyValue("--accent"), [0.71, 0.33, 0.04]));
    gl.uniform3fv(uInk, parseColor(styles.getPropertyValue("--ink-faint"), [0.6, 0.56, 0.52]));
  };
  readColors();

  let width = 0;
  let height = 0;
  const resize = () => {
    const ratio = Math.min(window.devicePixelRatio || 1, MAX_PIXEL_RATIO);
    width = Math.max(1, Math.round(canvas.clientWidth * ratio));
    height = Math.max(1, Math.round(canvas.clientHeight * ratio));
    if (canvas.width !== width || canvas.height !== height) {
      canvas.width = width;
      canvas.height = height;
    }
    gl.viewport(0, 0, width, height);
    gl.uniform2f(uResolution, width, height);
  };
  resize();

  let pointerX = -1e4;
  let pointerY = -1e4;
  let pointerStrength = 0;
  let pointerTargetStrength = 0;
  const onPointerMove = (event: PointerEvent) => {
    const rect = canvas.getBoundingClientRect();
    const ratio = width / Math.max(1, rect.width);
    pointerX = (event.clientX - rect.left) * ratio;
    pointerY = (rect.bottom - event.clientY) * ratio;
    pointerTargetStrength = 1;
  };
  const onPointerLeave = () => {
    pointerTargetStrength = 0;
  };
  pointerTarget.addEventListener("pointermove", onPointerMove, { passive: true });
  pointerTarget.addEventListener("pointerleave", onPointerLeave, { passive: true });

  let visible = true;
  let frame = 0;
  let lastCallback = 0;
  let lastDraw = 0;
  // A fixed non-zero start keeps the still frame from always showing the
  // zero-time terrain; the running clock continues from here.
  let elapsed = 40;
  let animated = initiallyAnimated;
  let tooSlow = false;
  let disposed = false;
  const probe = createFrameProbe();

  const render = () => {
    gl.uniform1f(uTime, elapsed);
    gl.uniform2f(uPointer, pointerX, pointerY);
    gl.uniform1f(uPointerStrength, pointerStrength);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  };

  const shouldRun = () => animated && !tooSlow && !disposed && visible && !document.hidden;

  const draw = (now: number) => {
    frame = 0;
    if (!shouldRun()) return;
    frame = requestAnimationFrame(draw);
    if (lastCallback !== 0 && probe(now - lastCallback)) tooSlow = true;
    lastCallback = now;
    if (lastDraw !== 0 && now - lastDraw < FRAME_INTERVAL_MS) return;
    const delta = lastDraw === 0 ? FRAME_INTERVAL_MS : Math.min(now - lastDraw, 100);
    lastDraw = now;
    elapsed += delta / 1000;
    pointerStrength += (pointerTargetStrength - pointerStrength) * 0.08;
    render();
  };
  const resume = () => {
    if (frame === 0 && shouldRun()) {
      lastCallback = 0;
      lastDraw = 0;
      frame = requestAnimationFrame(draw);
    }
  };

  const intersection = new IntersectionObserver(([entry]) => {
    visible = entry?.isIntersecting ?? true;
    resume();
  });
  intersection.observe(canvas);
  const resizeObserver = new ResizeObserver(() => {
    resize();
    if (!shouldRun()) render();
  });
  resizeObserver.observe(canvas);
  const themeObserver = new MutationObserver(() => {
    readColors();
    if (!shouldRun()) render();
  });
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["class", "data-accent"],
  });
  document.addEventListener("visibilitychange", resume);
  render();
  resume();

  return {
    setAnimated: (next) => {
      animated = next;
      if (next) {
        resume();
      } else if (frame !== 0) {
        cancelAnimationFrame(frame);
        frame = 0;
      }
    },
    dispose: () => {
      disposed = true;
      if (frame !== 0) cancelAnimationFrame(frame);
      intersection.disconnect();
      resizeObserver.disconnect();
      themeObserver.disconnect();
      document.removeEventListener("visibilitychange", resume);
      pointerTarget.removeEventListener("pointermove", onPointerMove);
      pointerTarget.removeEventListener("pointerleave", onPointerLeave);
      gl.deleteBuffer(buffer);
      gl.deleteProgram(program);
      gl.getExtension("WEBGL_lose_context")?.loseContext();
    },
  };
}
