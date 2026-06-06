import { fireEvent, render, screen, waitFor } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../common/i18n";
import IncidentsListingPage from "./ListingPage.vue";

describe("Incidents listing", () => {
  it("shows current/past sections and applies search visibility rules", async () => {
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [{ path: "/incidents", component: IncidentsListingPage }],
    });
    await router.push("/incidents");
    await router.isReady();

    render(IncidentsListingPage, { global: { plugins: [router, createAppI18n("en")] } });

    expect(await screen.findByRole("heading", { name: "Current" })).not.toBeNull();
    expect(screen.getByRole("heading", { name: "Past" })).not.toBeNull();
    expect(screen.queryByText("No incidents match your search.")).toBeNull();
    expect(screen.getByText(/INC-001/)).not.toBeNull();
    expect(screen.getByText("Awaiting valve replacement")).not.toBeNull();
    expect(screen.getByText("Awaiting valve replacement").closest(".latest-timeline-entry")?.classList.contains("latest-timeline-entry-stretched")).toBe(true);

    const input = screen.getByPlaceholderText("Search incidents");
    await fireEvent.update(input, "generator");

    await waitFor(() => {
      expect(screen.queryByRole("heading", { name: "Current" })).toBeNull();
      expect(screen.getByRole("heading", { name: "Past" })).not.toBeNull();
    });

    await fireEvent.update(input, "no-match-value");

    await waitFor(() => {
      expect(screen.queryByRole("heading", { name: "Current" })).toBeNull();
      expect(screen.queryByRole("heading", { name: "Past" })).toBeNull();
      expect(screen.getByText("No incidents match your search.")).not.toBeNull();
    });
  });
});
