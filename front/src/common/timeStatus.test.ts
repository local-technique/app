import { describe, expect, it } from "vitest";
import { computeDisplayStatus, computeTimeStatus } from "./timeStatus";

const NOW = new Date("2026-06-29T12:00:00Z");

describe("computeTimeStatus", () => {
  it("no dates -> TO_COME", () => {
    expect(computeTimeStatus(null, null, NOW)).toBe("TO_COME");
    expect(computeTimeStatus(undefined, undefined, NOW)).toBe("TO_COME");
  });

  it("start in future -> TO_COME", () => {
    expect(computeTimeStatus("2026-07-01T00:00:00Z", null, NOW)).toBe("TO_COME");
  });

  it("start today/past, no end -> ONGOING", () => {
    expect(computeTimeStatus("2026-06-29T10:00:00Z", null, NOW)).toBe("ONGOING");
    expect(computeTimeStatus("2026-06-28T00:00:00Z", null, NOW)).toBe("ONGOING");
  });

  it("start today/past, end today/past -> PAST", () => {
    expect(computeTimeStatus("2026-06-28T00:00:00Z", "2026-06-29T10:00:00Z", NOW)).toBe("PAST");
    expect(computeTimeStatus("2026-06-28T00:00:00Z", "2026-06-28T23:59:00Z", NOW)).toBe("PAST");
  });

  it("start today/past, end in future -> ONGOING", () => {
    expect(computeTimeStatus("2026-06-28T00:00:00Z", "2026-07-01T00:00:00Z", NOW)).toBe("ONGOING");
  });

  it("start absent, end in past -> PAST", () => {
    expect(computeTimeStatus(null, "2026-06-28T00:00:00Z", NOW)).toBe("PAST");
  });

  it("start absent, end today/future -> TO_COME", () => {
    expect(computeTimeStatus(null, "2026-06-29T15:00:00Z", NOW)).toBe("TO_COME");
    expect(computeTimeStatus(null, "2026-07-01T00:00:00Z", NOW)).toBe("TO_COME");
  });
});

describe("computeDisplayStatus", () => {
  it("PAST -> finished regardless of stored status", () => {
    expect(computeDisplayStatus("waiting", "PAST")).toBe("finished");
    expect(computeDisplayStatus("ongoing", "PAST")).toBe("finished");
  });

  it("ongoing + TO_COME -> planned", () => {
    expect(computeDisplayStatus("ongoing", "TO_COME")).toBe("planned");
  });

  it("waiting + TO_COME -> waiting", () => {
    expect(computeDisplayStatus("waiting", "TO_COME")).toBe("waiting");
  });

  it("ongoing + ONGOING -> ongoing", () => {
    expect(computeDisplayStatus("ongoing", "ONGOING")).toBe("ongoing");
  });

  it("waiting + ONGOING -> waiting", () => {
    expect(computeDisplayStatus("waiting", "ONGOING")).toBe("waiting");
  });
});
