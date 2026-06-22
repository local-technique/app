import { describe, expect, it } from "vitest";
import { toDateTimeLocalInput, toUtcFromDateTimeLocalInput, toDateLocalInput, toUtcFromDateLocalInput, todayDateInput } from "./dateInput";

describe("date input UTC conversion", () => {
  it("round-trips UTC datetime-local values without timezone shifts", () => {
    const input = toDateTimeLocalInput("2026-02-01T10:30:00Z");

    expect(input).toBe("2026-02-01T10:30");
    expect(toUtcFromDateTimeLocalInput(input)).toBe("2026-02-01T10:30:00.000Z");
  });
});

describe("date-only input conversion", () => {
  it("converts UTC ISO to date-only input value", () => {
    expect(toDateLocalInput("2026-02-01T10:30:00.000Z")).toBe("2026-02-01");
    expect(toDateLocalInput("2026-06-21T14:00:00.000Z")).toBe("2026-06-21");
    expect(toDateLocalInput(null)).toBe("");
    expect(toDateLocalInput(undefined)).toBe("");
    expect(toDateLocalInput("")).toBe("");
  });

  it("converts date-only value to UTC with current time", () => {
    const now = new Date(2026, 5, 21, 14, 30, 0);
    const result = toUtcFromDateLocalInput("2026-02-01", now);
    expect(result).toBe("2026-02-01T14:30:00.000Z");
  });

  it("returns null for empty date input", () => {
    expect(toUtcFromDateLocalInput("")).toBeNull();
  });
});

describe("todayDateInput", () => {
  it("returns today in YYYY-MM-DD format", () => {
    const result = todayDateInput();
    expect(result).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });
});
