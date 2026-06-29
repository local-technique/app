# Unified Time Status Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a single shared `computeTimeStatus` function that replaces duplicated date-based status logic across events, incidents, and projects.

**Architecture:** Pure function in `front/src/common/timeStatus.ts` returning a `TimeStatus` enum value. Each module maps the result to its own section type.

**Tech Stack:** TypeScript, Vitest

---

### Task 1: Create `common/timeStatus.ts` + tests

**Files:**
- Create: `front/src/common/timeStatus.ts`
- Create: `front/src/common/timeStatus.test.ts`

- [ ] **Step 1: Write the failing tests**

Write to `front/src/common/timeStatus.test.ts`:

```typescript
import { describe, expect, it } from "vitest";
import { computeTimeStatus } from "./timeStatus";

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
```

- [ ] **Step 2: Run tests to confirm they fail**

Run: `npm run test -- --run front/src/common/timeStatus.test.ts` from `front/`
Expected: FAIL — module not found / function not exported

- [ ] **Step 3: Write minimal implementation**

Write to `front/src/common/timeStatus.ts`:

```typescript
export type TimeStatus = "TO_COME" | "ONGOING" | "PAST";

export function computeTimeStatus(
  startUtc?: string | null,
  endUtc?: string | null,
  now = new Date(),
): TimeStatus {
  const nowMs = now.getTime();
  const startMs = startUtc ? Date.parse(startUtc) : null;
  const endMs = endUtc ? Date.parse(endUtc) : null;

  if (startMs === null) {
    if (endMs !== null && endMs <= nowMs) return "PAST";
    return "TO_COME";
  }

  if (startMs > nowMs) return "TO_COME";

  if (endMs === null) return "ONGOING";

  if (endMs <= nowMs) return "PAST";

  return "ONGOING";
}
```

- [ ] **Step 4: Run tests to confirm they pass**

Run: `npm run test -- --run front/src/common/timeStatus.test.ts` from `front/`
Expected: PASS (all 7 test cases)

---

### Task 2: Update `events/utils.ts` to use `computeTimeStatus`

**Files:**
- Modify: `front/src/events/utils.ts`
- Possibly: `front/src/events/ListingPage.vue` (if `groupByStatus` key mapping needed)

- [ ] **Step 1: Replace `classifyEventStatus` with `computeTimeStatus`**

In `front/src/events/utils.ts`:
- Remove `import { classifyEventStatus, ... }` — change to `import { computeTimeStatus, formatLocalDate, formatLocalDateTime, parseUtc } from "../common/date";`
  - Actually, `classifyEventStatus` is from `../common/date`, but `computeTimeStatus` will be from `../common/timeStatus`. So add a new import: `import { computeTimeStatus } from "../common/timeStatus";`
  - Keep the existing import from `../common/date` but remove `classifyEventStatus` from it.

- In `toEventViewModel`, replace:
  ```typescript
  status: classifyEventStatus({ startUtc: event.startUtc, endUtc: event.endUtc }),
  ```
  with:
  ```typescript
  status: toSection(computeTimeStatus(event.startUtc, event.endUtc)),
  ```

- Add a mapping function before `toEventViewModel`:
  ```typescript
  function toSection(status: TimeStatus): EventStatusSection {
    switch (status) {
      case "TO_COME": return "toCome";
      case "ONGOING": return "current";
      case "PAST": return "past";
    }
  }
  ```
  
  And add the import:
  ```typescript
  import type { TimeStatus } from "../common/timeStatus";
  ```

- [ ] **Step 2: Run existing tests**

Run: `npm run test` from `front/`
Expected: PASS — no test changes needed since the behavior is equivalent

---

### Task 3: Update `incidents/utils.ts` to use `computeTimeStatus`

**Files:**
- Modify: `front/src/incidents/utils.ts`

- [ ] **Step 1: Replace `toIncidentStatus` with `computeTimeStatus`**

In `front/src/incidents/utils.ts`:
- Add imports:
  ```typescript
  import { computeTimeStatus } from "../common/timeStatus";
  import type { TimeStatus } from "../common/timeStatus";
  ```

- Remove the `toIncidentStatus` function (lines 58-65).

- In `toIncidentViewModel`, replace:
  ```typescript
  status: toIncidentStatus(incident),
  ```
  with:
  ```typescript
  status: toSection(computeTimeStatus(incident.startUtc, incident.endUtc)),
  ```

- Add a mapping function:
  ```typescript
  function toSection(status: TimeStatus): IncidentStatusSection {
    switch (status) {
      case "TO_COME":
      case "ONGOING": return "current";
      case "PAST": return "past";
    }
  }
  ```

- _Note: `TO_COME` maps to `"current"` because `IncidentStatusSection` is `"current" | "past"` and has no `"toCome"` variant._

- [ ] **Step 2: Run existing tests**

Run: `npm run test` from `front/`
Expected: PASS

---

### Task 4: Update `projects/utils.ts` to use `computeTimeStatus`

**Files:**
- Modify: `front/src/projects/utils.ts`

- [ ] **Step 1: Replace `classifyProject`'s date logic with `computeTimeStatus`**

In `front/src/projects/utils.ts`:
- Add imports:
  ```typescript
  import { computeTimeStatus } from "../common/timeStatus";
  import type { TimeStatus } from "../common/timeStatus";
  ```

- Replace the `classifyProject` function (lines 48-60) with a simpler version that only handles the `statusType` mapping based on `stored` status:

  ```typescript
  function classifyProject(project: ProjectItem, now = new Date()): { section: ProjectStatusSection; statusType: ProjectDisplayStatus } {
    const timeStatus = computeTimeStatus(project.startUtc, project.endUtc, now);

    let section: ProjectStatusSection;
    switch (timeStatus) {
      case "TO_COME": section = "toCome"; break;
      case "ONGOING": section = "ongoing"; break;
      case "PAST": section = "finished"; break;
    }

    let statusType: ProjectDisplayStatus;
    if (timeStatus === "PAST") {
      statusType = "finished";
    } else if (project.statusType === "ongoing" && timeStatus === "TO_COME") {
      statusType = "planned";
    } else {
      statusType = project.statusType;
    }

    return { section, statusType };
  }
  ```

- [ ] **Step 2: Run existing tests**

Run: `npm run test` from `front/`
Expected: PASS

---

### Task 5: Run full test suite and lint

- [ ] **Step 1: Run all tests**

Run: `npm run test` from `front/`
Expected: All tests pass

- [ ] **Step 2: Run lint**

Run: `npm run lint` from `front/`
Expected: No errors
