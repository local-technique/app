# Unified Time Status for Items

## Problem

Three item types (events/maintenances, incidents, projects) each have their own
date-based status computation logic, duplicated across the frontend. The logic
is inconsistent:

- Events use `common/date.ts:classifyEventStatus()` for section + `events/utils.ts:computeStatusType()` for display
- Incidents have their own `incidents/utils.ts:toIncidentStatus()`
- Projects have their own `projects/utils.ts:classifyProject()`

## Solution

A single shared function `computeTimeStatus` in a dedicated module
`common/timeStatus.ts` that all three item types call.

## Type

```typescript
export type TimeStatus = "TO_COME" | "ONGOING" | "PAST";
```

## Function signature

```typescript
export function computeTimeStatus(
  startUtc?: string | null,
  endUtc?: string | null,
  now?: Date,                // defaults to new Date()
): TimeStatus
```

## Logic

```
if start is absent:
  if end is present AND end <= now -> PAST
  else -> TO_COME

if start > now -> TO_COME

if end is absent -> ONGOING

if end <= now -> PAST

otherwise -> ONGOING
```

## Test cases

Frozen at `now = new Date("2026-06-29T12:00:00Z")`:

| # | startUtc | endUtc | Expected |
|---|----------|--------|----------|
| 1 | null | null | TO_COME |
| 2 | null | undefined | TO_COME |
| 3 | `"2026-07-01T00:00:00Z"` (future) | null | TO_COME |
| 4 | `"2026-06-29T10:00:00Z"` (today) | null | ONGOING |
| 5 | `"2026-06-28T00:00:00Z"` (past) | `"2026-06-29T10:00:00Z"` (today) | PAST |
| 6 | `"2026-06-28T00:00:00Z"` (past) | `"2026-07-01T00:00:00Z"` (future) | ONGOING |
| 7 | null | `"2026-06-28T00:00:00Z"` (past) | PAST |
| 8 | null | `"2026-06-29T15:00:00Z"` (today) | TO_COME |
| 9 | null | `"2026-07-01T00:00:00Z"` (future) | TO_COME |

## Migration

1. Create `common/timeStatus.ts` with the type and function
2. Write tests in `common/timeStatus.test.ts`
3. Update `events/utils.ts` to use `computeTimeStatus`
4. Update `incidents/utils.ts` to use `computeTimeStatus`
5. Update `projects/utils.ts` to use `computeTimeStatus`
6. Remove duplicated logic from each module

## Non-goals

- This does not change the stored `status_type` database column ("waiting"/"ongoing")
- This does not change the backend; status computation remains frontend-only
- This does not alter the UI section grouping (current/toCome/past per module)
