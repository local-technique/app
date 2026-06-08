# Category Colors Design

## Goal

Add a persisted color to each category and use it consistently wherever category badges and listing card category borders appear.

## Data Model

- Add `color TEXT NOT NULL DEFAULT '#9aaab1'` to `event_categories`.
- Existing categories receive seeded colors in a migration.
- Category API list/create/update payloads include `color`.
- Backend validation accepts only `#RRGGBB`, normalizes to lowercase, and rejects invalid values.

## Admin Experience

The category create/edit form adds one color input with a GitHub pull-request-label style experience:

- A color swatch preview.
- A hex text input.
- A popover with default color choices.
- A `refresh-ccw` icon button that sets a random color.

The categories table shows the category color as a swatch plus hex value.

## Display

- `CategoryBadge` accepts `color`.
- The badge icon, code, and rail divider use the category color.
- Listing cards receive `--category-color` from their category and use it for the existing left border/stripe.
- Detail pages pass category color to the inline category badge.
- Edition pages pass the selected category color to the badge beside the category dropdown.

## Scope

Apply category colors to maintenance events, incidents, and projects. Status icons and text remain semantic; only the category badge and the left card category border use category color.

## Testing

- Backend tests cover valid/invalid color validation and normalized payloads.
- Frontend tests cover admin color input rendering, random color button behavior, category color display in the admin table, and category color propagation to listing/detail/form badges.
- Run frontend checks: `npm run lint`, `npm run test`, `npm run build`.
- Run backend checks: `cargo clippy --all-features -- -D warnings`, `cargo test --all-features`, `cargo build --all-features`.
