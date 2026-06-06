import { describe, expect, it } from "vitest";
import { toDateTimeLocalInput, toUtcFromDateTimeLocalInput } from "./dateInput";

describe("date input UTC conversion", () => {
  it("round-trips UTC datetime-local values without timezone shifts", () => {
    const input = toDateTimeLocalInput("2026-02-01T10:30:00Z");

    expect(input).toBe("2026-02-01T10:30");
    expect(toUtcFromDateTimeLocalInput(input)).toBe("2026-02-01T10:30:00.000Z");
  });
});
