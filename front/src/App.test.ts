import { describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/vue";
import { createRouter, createWebHashHistory } from "vue-router";
import App from "./App.vue";
import { createAppI18n } from "./common/i18n";

async function renderApp() {
  localStorage.setItem("copro-locale", "en");
  const router = createRouter({
    history: createWebHashHistory(),
    routes: [
      { path: "/events", component: { template: "<div>events</div>" } },
      { path: "/incidents", component: { template: "<div>incidents</div>" } },
    ],
  });
  await router.push("/events");
  await router.isReady();

  return render(App, { global: { plugins: [router, createAppI18n("en")] } });
}

async function renderAppInEnglish() {
  await renderApp();

  const localeSelect = document.getElementById("app-locale");
  if (!(localeSelect instanceof HTMLSelectElement)) {
    throw new Error("Locale select not found");
  }

  if (localeSelect.value !== "en") {
    await fireEvent.update(localeSelect, "en");
  }
}

describe("app shell", () => {
  it("renders sidebar entries and controls", async () => {
    await renderAppInEnglish();

    expect(screen.getAllByText("Events & Maintenance").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Incidents").length).toBeGreaterThan(0);
    expect(screen.getByLabelText("Language")).toBeTruthy();
    expect(screen.getByLabelText("Theme")).toBeTruthy();
  });

  it("opens mobile menu from bottom nav more button", async () => {
    await renderAppInEnglish();

    expect(screen.queryByRole("navigation", { name: "Mobile menu" })).toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Open more" }));

    expect(screen.getByRole("navigation", { name: "Mobile menu" })).toBeTruthy();
  });

  it("persists locale and theme selections", async () => {
    localStorage.clear();
    localStorage.setItem("copro-locale", "en");
    await renderAppInEnglish();

    await fireEvent.update(screen.getByLabelText("Theme"), "dark");
    await fireEvent.update(screen.getByLabelText("Language"), "fr");

    expect(localStorage.getItem("copro-locale")).toBe("fr");
    expect(localStorage.getItem("copro-theme")).toBe("dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
  });
});
