# Incident Timeline Visual Design

## Goal

Rework incident detail timeline entries into an actual vertical timeline while supporting pending entries without a date.

## Approved Design

- Use a line-left timeline layout on incident detail pages.
- Use the app's existing blue accent family for the vertical line and dots.
- Render the timeline date/time in a left gutter, the line/dot beside it, and a compact card to the right.
- Right-side entry cards should feel like smaller versions of existing listing/detail cards.
- Timeline entries with `at_utc = null` are pending entries.
- Pending entries render first and show a bright orange `Pending` badge in the date gutter.
- Dated entries show formatted date/time and prefix the entry title with the Lucide `CircleCheck` icon.
- Timeline ordering is `at_utc DESC NULLS FIRST`, then `sort_order` for stable ordering.
- Incident create/edit forms allow timeline entries without a date and save them as `at_utc: null`.

## Scope

- Backend migration makes `incident_timeline.at_utc` nullable.
- Backend models, save paths, detail/edit responses, and ordering support nullable timeline dates.
- Frontend incident types, repositories, forms, view model, and detail page support nullable dates.
- Add tests for pending timeline rendering and mobile-safe item count/state where relevant.

## Non-Goals

- No new workflow/status field beyond date-null pending behavior.
- No timeline changes for maintenance events.
- No alternating desktop timeline layout.
