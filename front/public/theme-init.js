(() => {
  const storageKey = "copro-theme";
  const mode = localStorage.getItem(storageKey);
  const prefersDark =
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches;

  let resolved = "light";
  if (mode === "dark" || mode === "light") {
    resolved = mode;
  } else if (mode === "system" || mode === null) {
    resolved = prefersDark ? "dark" : "light";
  }

  document.documentElement.dataset.theme = mode === "dark" || mode === "light" || mode === "system" ? mode : "system";
  document.documentElement.dataset.resolvedTheme = resolved;
})();
