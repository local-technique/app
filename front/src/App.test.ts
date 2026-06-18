import { describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/vue";
import { createRouter, createWebHashHistory } from "vue-router";
import App from "./App.vue";
import { createAppI18n } from "./common/i18n";
import { currentUserRoles } from "./auth/session";

async function renderApp() {
  localStorage.setItem("copro-locale", "en");
  currentUserRoles.loaded = true;
  currentUserRoles.roles = ["ADMIN", "CO_OWNER"];
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

describe("app shell", () => {
  it("renders sidebar entries", async () => {
    await renderApp();

    expect(screen.getAllByText("Events & Maintenance").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Incidents").length).toBeGreaterThan(0);
  });

  it("opens mobile menu from bottom nav more button", async () => {
    await renderApp();

    expect(screen.queryByRole("navigation", { name: "Mobile menu" })).toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Open more" }));

    expect(screen.getByRole("navigation", { name: "Mobile menu" })).toBeTruthy();
  });

  it("persists theme via system preference change", async () => {
    localStorage.clear();
    localStorage.setItem("copro-locale", "en");
    localStorage.setItem("copro-theme", "dark");
    await renderApp();

    expect(document.documentElement.dataset.theme).toBe("dark");
  });
});
