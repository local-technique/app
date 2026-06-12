import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../i18n";
import MobileBottomNav from "./MobileBottomNav.vue";

describe("Mobile bottom nav", () => {
  it("shows at most 6 items (5 nav + ...)", async () => {
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [{ path: "/events", component: { template: "<div />" } }],
    });
    await router.push("/events");
    await router.isReady();

    const { container } = render(MobileBottomNav, {
      props: { showCoOwnerLinks: true, showAdminLink: true },
      global: { plugins: [router, createAppI18n("en")] },
    });

    expect(container.querySelectorAll(".nav-item")).toHaveLength(6);
  });

  it("sticks the more button at the rightmost slot", async () => {
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [{ path: "/settings", component: { template: "<div />" } }],
    });
    await router.push("/settings");
    await router.isReady();

    const { container } = render(MobileBottomNav, {
      props: { showCoOwnerLinks: false, showAdminLink: false },
      global: { plugins: [router, createAppI18n("en")] },
    });

    const items = container.querySelectorAll(".nav-item");
    expect(items).toHaveLength(6);
    expect(items[1].classList.contains("nav-item-spacer")).toBe(true);
    expect(items[5].classList.contains("nav-item-more")).toBe(true);
  });

  it("marks events active", async () => {
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [{ path: "/events", component: { template: "<div />" } }],
    });
    await router.push("/events");
    await router.isReady();

    render(MobileBottomNav, {
      props: { showCoOwnerLinks: true, showAdminLink: true },
      global: { plugins: [router, createAppI18n("en")] },
    });

    expect(screen.getByRole("link", { name: "Events & Maintenance" }).classList.contains("active")).toBe(true);
  });

  it("marks projects active", async () => {
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [{ path: "/projects", component: { template: "<div />" } }],
    });
    await router.push("/projects");
    await router.isReady();

    render(MobileBottomNav, {
      props: { showCoOwnerLinks: true, showAdminLink: true },
      global: { plugins: [router, createAppI18n("en")] },
    });

    expect(screen.getByRole("link", { name: "Projects" }).classList.contains("active")).toBe(true);
  });
});
