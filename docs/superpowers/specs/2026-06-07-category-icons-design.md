# Category Icons Design

## Goal

Show each item's category icon consistently across maintenance events, incidents, and projects.

## Scope

- Listing pages for events, incidents, and projects show a category visual rail to the left of each card.
- Detail pages for events, incidents, and projects show the category icon next to the category code and label.
- Create/edit pages for events, incidents, and projects show the selected category icon to the left of the category dropdown.
- Category data already includes `icon`, `code`, and `label`; no API or persistence changes are needed.

## Listing Layout

Each listing card uses a two-column layout:

- Left rail: category icon above category code.
- Small vertical demarcation between the rail and the existing card content.
- Right content: current title, dates, status, location, timeline preview, and links stay unchanged.

If an item has no category, the rail is omitted and the card keeps the existing content layout.

## Detail Layout

The current category text line becomes an inline category row:

- Category icon on the left.
- Existing `CODE - Label` text on the right.
- The row remains in the current metadata area below the title/actions.

## Edition Layout

Native select options remain text-only for browser compatibility. The selected category icon appears directly to the left of the category dropdown and updates when the selected category changes.

## Components

Reuse the existing `CategoryIcon` component. Add a small shared display component if it keeps repeated templates and CSS out of the six affected pages.

## Testing

Update frontend tests to assert that category icons render on listing, detail, and form pages for the three domains where coverage exists. Run `npm run lint`, `npm run test`, and `npm run build` in `front/`.
