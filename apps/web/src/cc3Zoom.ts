// CC3: Ctrl +/-/0 zooms the whole GUI; terminals handle the same keys themselves (font size).
const STORAGE_KEY = "cc3:gui-zoom";
const STEP = 0.1;
const MIN = 0.5;
const MAX = 2;

function clamp(value: number): number {
  return Math.min(MAX, Math.max(MIN, Math.round(value * 10) / 10));
}

function apply(zoom: number): void {
  document.documentElement.style.zoom = zoom === 1 ? "" : String(zoom);
  localStorage.setItem(STORAGE_KEY, String(zoom));
}

export function installGuiZoom(): void {
  const stored = Number(localStorage.getItem(STORAGE_KEY));
  let zoom = Number.isFinite(stored) && stored > 0 ? clamp(stored) : 1;
  apply(zoom);
  window.addEventListener("keydown", (event) => {
    if (event.defaultPrevented || !event.ctrlKey || event.altKey || event.metaKey) return;
    const next =
      event.key === "+" || event.key === "="
        ? zoom + STEP
        : event.key === "-" || event.key === "_"
          ? zoom - STEP
          : event.key === "0"
            ? 1
            : null;
    if (next === null) return;
    event.preventDefault();
    zoom = clamp(next);
    apply(zoom);
  });
}
