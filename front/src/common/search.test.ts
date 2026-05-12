import { describe, expect, it } from "vitest";
import { fuzzyMatch } from "./search";

describe("fuzzyMatch", () => {
  it("matches tokenized partial words in order", () => {
    expect(fuzzyMatch("heat maint", "Heating maintenance in block A")).toBe(true);
  });

  it("returns false when one token has no match", () => {
    expect(fuzzyMatch("heat leak", "Heating maintenance in block A")).toBe(false);
  });

  it("matches case-insensitively and ignores extra whitespace", () => {
    expect(fuzzyMatch("  HEAT   MAI  ", "Heating maintenance in block A")).toBe(true);
  });

  it("matches empty query by default", () => {
    expect(fuzzyMatch("", "Any text")).toBe(true);
  });
});
