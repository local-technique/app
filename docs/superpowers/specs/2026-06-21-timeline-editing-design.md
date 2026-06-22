# Timeline Editing — Design

## Goal

Add inline editing of timeline entries for incidents, projects, and maintenances (events) — only for users with role-based edit permission. Lightweight integration on both detail pages and form pages, using the same component.

## Approach A — Dedicated Timeline CRUD Endpoints

New backend endpoints per entity, frontend `EditableTimelineList` component, integrated into both detail and form pages.

## Permission Gate

All timeline mutation endpoints enforce the same role set as item editing:

```
Admin, CoOwnershipBoard, CoOwnershipBoardOps
```

- **Backend**: `principal.ensure_any_role(...)` in each HTTP handler
- **Frontend**: UI controls hidden unless `canEdit` is true (derived from same role check)

## Backend — New API Endpoints

### For each entity: incidents, projects, maintenances

| Method | Path | Action | Response |
|--------|------|--------|----------|
| `POST` | `/{entity}/{id}/timeline` | Create entry | 201 + created entry (with id) |
| `PUT` | `/{entity}/{id}/timeline/{entryId}` | Update entry | 200 + updated entry |
| `DELETE` | `/{entity}/{id}/timeline/{entryId}` | Delete entry | 204 No Content |

### Request body (POST/PUT)

```json
{
  "at_utc": "2026-06-21T14:30:00Z",
  "sort_order": 0,
  "fields": {
    "title": "Entry title",
    "details": "Optional details"
  }
}
```

Fields are stored for the **principal's current locale** (from the JWT session). This matches how the existing `save_partial` handler handles timeline data.

### Response body (POST/PUT)

```json
{
  "id": "uuid",
  "at_utc": "2026-06-21T14:30:00Z",
  "title": "Entry title (locale-resolved)",
  "details": "Optional details (locale-resolved)"
}
```

### Changes per Rust module

- **`http.rs`** — 3 new handler functions per entity, each calling `principal.ensure_any_role` then delegating to service/repo
- **`repository.rs`** — 3 new SQL methods per entity: `create_timeline_entry`, `update_timeline_entry`, `delete_timeline_entry`
- **`service.rs`** — reuse existing timeline validation (field keys, non-empty title)
- **`model.rs`** — new request/response structs if needed (can reuse existing `TimelineSaveItem` / `TimelineItem` patterns)
- **`router.rs`** — 9 new routes (3 entities × 3 endpoints)

## Frontend — EditableTimelineList Component

### Props

```typescript
{
  entries: TimelineEntryViewModel[]   // read-only display data
  canEdit: boolean                    // show/hide controls
  onAdd(entry: {...}): Promise<void>  // detail mode: calls API
  onUpdate(entry: {...}): Promise<void> // detail mode: calls API
  onDelete(id: string): Promise<void> // detail mode: calls API
  // For form mode, parent listens to @change events and collects timeline state
}
```

### Rendering

**Read-only** (same as current `TimelineList`): date column + axis dot + card with title/details.

**When `canEdit` is true, additionally:**

- **New-entry form** at the top of the timeline — visually identical structure to a timeline card but all fields are inputs (date/time picker defaulted to current time, title input, details textarea). Save icon + X icon on the right.

- **Each existing entry** gets a trash-2 icon (red background) and a pencil icon on the right. Pencil toggles the entry into edit mode — same layout as read-only but fields become inputs matching their current values.

- **Edit mode layout** mirrors read-only as closely as possible: same card structure, same spacing, same typography, but `<input>`/`<textarea>` replace plain text, and the right-side icons swap to save icon + X icon.

### Save / Cancel Behavior

| Action | Result |
|--------|--------|
| Click **save icon** | Persists the entry (API call or local state) |
| Click **X icon** | Discards changes, reverts to read-only |
| **Trash icon** | Deletes immediately (confirmation not required — lightweight) |

### Mode-Specific Behavior

- **Detail mode**: Each add/edit/delete calls the new API endpoint immediately. The component manages its own loading/error state inline.
- **Form mode**: The component mutates local state and emits change events. The parent FormPage collects the final timeline array and includes it in the main save payload with `replace_timeline: true`.

## Integration

| Page | Current | After |
|------|---------|-------|
| Incident DetailPage | Inline duplicated HTML rendering read-only timeline | Replace with `<EditableTimelineList mode="detail">` |
| Project DetailPage | `<TimelineList>` | Replace with `<EditableTimelineList mode="detail">` |
| Event DetailPage | `<TimelineList>` | Replace with `<EditableTimelineList mode="detail">` |
| Incident FormPage | `<TimelineList>` read-only, timeline sent as `[]` | Replace with `<EditableTimelineList mode="form">`, form save includes timeline array |
| Project FormPage | `<TimelineList>` read-only, timeline sent as `[]` | Same pattern |
| Event FormPage | `<TimelineList>` read-only, timeline sent as `[]` | Same pattern |

## Edge Cases

- **Empty title on save**: For a **new** entry — don't create (save button disabled when title empty). For an **existing** entry being edited — keep the previous value (don't overwrite with empty). Save button disabled when title is empty.
- **Pending entries** (at_utc = null): Show with orange badge as currently; can still edit date/time to set a real date, or leave as pending
- **Concurrent edits**: Not handled at this level — last-write-wins is acceptable for the expected usage
- **Loading states**: Buttons show a spinner or disabled state during API calls
- **Error handling**: API errors shown as inline toasts near the affected entry
