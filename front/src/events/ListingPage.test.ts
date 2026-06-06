import { fireEvent, render, screen, waitFor } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../common/i18n";
import { currentUserRoles } from "../auth/session";
import EventsListingPage from "./ListingPage.vue";

describe("Events listing", () => {
  it("shows create action for admins", async () => {
    currentUserRoles.loaded = true;
    currentUserRoles.roles = ["ADMIN"];
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [
        { path: "/events", component: EventsListingPage },
        { path: "/events/new", component: { template: "<div />" } },
      ],
    });
    await router.push("/events");
    await router.isReady();

    render(EventsListingPage, { global: { plugins: [router, createAppI18n("en")] } });

    expect(await screen.findByRole("link", { name: "Create event" })).not.toBeNull();
  });

  it("shows sections and applies search visibility rules", async () => {
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [{ path: "/events", component: EventsListingPage }],
    });
    await router.push("/events");
    await router.isReady();

    render(EventsListingPage, { global: { plugins: [router, createAppI18n("en")] } });

    expect(await screen.findByRole("heading", { name: "Current" })).not.toBeNull();
    expect(screen.getByRole("heading", { name: "To come" })).not.toBeNull();
    expect(screen.getByRole("heading", { name: "Past" })).not.toBeNull();
    expect(screen.queryByText("No events match your search.")).toBeNull();
    expect(screen.getByText("⚠️ no hot water between 9h30 & 17h00")).not.toBeNull();

    expect(screen.queryByText("HEA-001")).toBeNull();

    const input = screen.getByPlaceholderText("Search events");
    await fireEvent.update(input, "property");

    await waitFor(() => {
      expect(screen.queryByRole("heading", { name: "Current" })).toBeNull();
      expect(screen.getByRole("heading", { name: "To come" })).not.toBeNull();
      expect(screen.queryByRole("heading", { name: "Past" })).toBeNull();
    });

    await fireEvent.update(input, "no-match-value");

    await waitFor(() => {
      expect(screen.queryByRole("heading", { name: "Current" })).toBeNull();
      expect(screen.queryByRole("heading", { name: "To come" })).toBeNull();
      expect(screen.queryByRole("heading", { name: "Past" })).toBeNull();
      expect(screen.getByText("No events match your search.")).not.toBeNull();
    });
  });
});
