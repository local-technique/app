import { describe, expect, it } from "vitest";
import type { IncidentItem } from "./types";
import { toIncidentViewModel } from "./utils";

describe("incident view model", () => {
  it("classifies incidents without end date as current after their start date", () => {
    const incident: IncidentItem = {
      id: "INC-OPEN",
      categoryCode: "HEA",
      title: { en: "Open incident" },
      description: { en: "Open" },
      startUtc: "2020-01-01T00:00:00Z",
      statusType: "ongoing",
      statusText: { en: "In progress" },
      timeline: [],
      attachments: [],
    };

    expect(toIncidentViewModel(incident, "en").status).toBe("current");
  });
});
