import { describe, expect, it, vi } from "vitest";
import type { ProjectItem } from "./types";
import { groupByStatus, renderProjectMarkdown, toProjectViewModel, matchesProjectQuery } from "./utils";

function project(overrides: Partial<ProjectItem> = {}): ProjectItem {
  return {
    id: "PRJ-001",
    categoryCode: "ELV",
    category: { id: "ELV", code: "ELV", icon: "arrow-up-down", color: "#0366d6", label: "Elevator" },
    title: { en: "Elevator modernization" },
    description: { en: "Replace controller" },
    startUtc: "2026-01-10T10:00:00Z",
    endUtc: undefined,
    statusType: "waiting",
    statusText: { en: "Awaiting quote" },
    attachments: [],
    ...overrides,
  };
}

describe("project utilities", () => {
  it("classifies finished, ongoing, and to-come projects", () => {
    vi.setSystemTime(new Date("2026-06-01T12:00:00Z"));

    const finished = toProjectViewModel(project({ endUtc: "2026-05-01T12:00:00Z" }), "en");
    const ongoingWithPastStart = toProjectViewModel(project({ startUtc: "2026-05-30T12:00:00Z", statusType: "ongoing" }), "en");
    const ongoingByStatus = toProjectViewModel(project({ startUtc: undefined, statusType: "ongoing" }), "en");
    const toCome = toProjectViewModel(project({ startUtc: "2026-07-01T12:00:00Z", statusType: "waiting" }), "en");

    expect(finished.status).toBe("finished");
    expect(ongoingWithPastStart.status).toBe("ongoing");
    expect(ongoingByStatus.status).toBe("ongoing");
    expect(toCome.status).toBe("toCome");
    expect(groupByStatus([finished, ongoingWithPastStart, ongoingByStatus, toCome])).toMatchObject({
      ongoing: [ongoingWithPastStart, ongoingByStatus],
      toCome: [toCome],
      finished: [finished],
    });

    vi.useRealTimers();
  });

  it("keeps waiting projects in to-come even when their start date has passed", () => {
    vi.setSystemTime(new Date("2026-06-01T12:00:00Z"));

    const waiting = toProjectViewModel(project({ startUtc: "2026-05-30T12:00:00Z", statusType: "waiting" }), "en");

    expect(waiting.status).toBe("toCome");
    expect(waiting.displayStatus).toBe("waiting");
    vi.useRealTimers();
  });

  it("formats missing project dates gracefully", () => {
    expect(toProjectViewModel(project({ startUtc: undefined, endUtc: undefined }), "en").dateLabel).toBe("Dates to be confirmed");
    expect(toProjectViewModel(project({ startUtc: "2026-01-10T10:00:00Z", endUtc: undefined }), "en").dateLabel).toContain("Jan");
    expect(toProjectViewModel(project({ startUtc: undefined, endUtc: "2026-02-10T10:00:00Z" }), "en").dateLabel).toContain("until");
  });

  it("renders conservative markdown while escaping raw html", () => {
    const html = renderProjectMarkdown("# ignored\n\nHello **board** and <script>alert(1)</script>\n\n- [site](https://example.com)\n- `safe`");

    expect(html).toContain("<strong>board</strong>");
    expect(html).toContain("&lt;script&gt;alert(1)&lt;/script&gt;");
    expect(html).toContain('<a href="https://example.com"');
    expect(html).toContain("<code>safe</code>");
    expect(html).not.toContain("<script>");
  });

  it("renders headings and pipe tables", () => {
    const html = renderProjectMarkdown("# Abri velo\n\nPour proteger les velos.\n\n| type | nombre |\n|-----|------:|\n| vtt | 17 |\n| long-tail | 43 |");

    expect(html).toContain("<h1>Abri velo</h1>");
    expect(html).toContain("<table>");
    expect(html).toContain("<th>type</th>");
    expect(html).toContain('<th class="align-right">nombre</th>');
    expect(html).toContain("<td>vtt</td>");
    expect(html).toContain('<td class="align-right">43</td>');
  });

  it("uses localized status text for display and search", () => {
    const item = project({ statusType: "ongoing", statusText: { en: "Installing insulation" }, description: { en: "Roof phase" } });

    expect(toProjectViewModel(item, "en").statusText).toBe("Installing insulation");
    expect(matchesProjectQuery(item, "installing", "en")).toBe(true);
    expect(matchesProjectQuery(item, "ongoing", "en")).toBe(true);
    expect(matchesProjectQuery(item, "plumbing", "en")).toBe(false);
  });

  it("uses generic finished text when end date is past", () => {
    vi.setSystemTime(new Date("2026-06-01T12:00:00Z"));
    const item = project({ endUtc: "2026-05-01T12:00:00Z", statusText: { en: "Installing insulation" } });

    expect(toProjectViewModel(item, "en").statusText).toBe("Finished");
    expect(matchesProjectQuery(item, "finished", "en")).toBe(true);
    vi.useRealTimers();
  });
});
