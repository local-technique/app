# Detail Page Header UX Redesign

## Summary

Consolidate the title, ID, and category lines on incident, maintenance (event), and project detail pages into a single header line: category icon (colored) + item ID + title.

## Current State

Each detail page shows three separate lines after the back-navigation link:

1. `<h1>` page title
2. `ID: <model.id>` 
3. Category badge (icon + key code) + label text
4. Edit/delete actions

This is redundant — the ID and category are shown as standalone lines when they could be integrated into the title row.

## Design

Replace lines 1–3 with a single `<h1 class="page-title">` heading that contains:

```
[CategoryIcon colored]  [model.id]  [model.title]
```

- **CategoryIcon**: rendered inline, colored via `--category-color` CSS variable, no label/code text
- **Item ID**: `model.id` displayed in a muted/monospace style as a secondary identifier
- **Title**: the main heading text

The edit/delete action buttons remain but move up directly below the new title line (replacing the position of the former ID line).

## Files to Change

- `front/src/incidents/DetailPage.vue` — template + scoped styles
- `front/src/projects/DetailPage.vue` — template + scoped styles
- `front/src/events/DetailPage.vue` — template + scoped styles

No changes needed to:
- ViewModel utilities (the data passed to template remains the same)
- CategoryBadge or CategoryIcon components (CategoryIcon is reused directly)
- Router, types, or stores

## Implementation Notes

- Import `CategoryIcon` instead of (or in addition to) `CategoryBadge` in each DetailPage
- Apply the category color directly to the `CategoryIcon` wrapper via `color: var(--category-color)`
- Use `display: inline-flex` with `gap` on the title line to lay out icon + ID + title
- Keep the same `page-title` class to reuse global heading styles
- Remove `ID:` label text — the raw ID value is enough
