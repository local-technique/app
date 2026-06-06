import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../common/i18n";
import EventsDetailPage from "./DetailPage.vue";

describe("Event detail", () => {
  it("shows maintenance warning with listing warning style", async () => {
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [
        { path: "/events/:id", component: EventsDetailPage },
        { path: "/events", component: { template: "<div />" } },
      ],
    });
    await router.push("/events/HEA-001");
    await router.isReady();

    render(EventsDetailPage, { global: { plugins: [router, createAppI18n("en")] } });

    const warning = await screen.findByText("⚠️ no hot water between 9h30 & 17h00");
    expect(warning.classList.contains("timeline-warning")).toBe(true);
  });
});
