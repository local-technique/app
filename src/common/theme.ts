export type ThemeMode = "system" | "light" | "dark";

const STORAGE_KEY = "copro-theme";

export function getStoredTheme(storage: Pick<Storage, "getItem"> = localStorage): ThemeMode {
  const value = storage.getItem(STORAGE_KEY);
  if (value === "light" || value === "dark" || value === "system") {
    return value;
  }
  return "system";
}

export function setStoredTheme(mode: ThemeMode, storage: Pick<Storage, "setItem"> = localStorage): void {
  storage.setItem(STORAGE_KEY, mode);
}

export function prefersDarkMode(): boolean {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return false;
  }
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

export function resolveTheme(mode: ThemeMode, prefersDark: boolean): "light" | "dark" {
  if (mode === "system") {
    return prefersDark ? "dark" : "light";
  }
  return mode;
}

export function applyTheme(
  mode: ThemeMode,
  options: {
    documentElement?: HTMLElement;
    prefersDark?: boolean;
  } = {},
): "light" | "dark" {
  const element = options.documentElement ?? document.documentElement;
  const resolved = resolveTheme(mode, options.prefersDark ?? prefersDarkMode());

  element.dataset.theme = mode;
  element.dataset.resolvedTheme = resolved;
  return resolved;
}
