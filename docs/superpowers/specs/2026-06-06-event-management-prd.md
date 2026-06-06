# Incident and Maintenance Event Management PRD

## Purpose

Allow authorized co-ownership board members to create and modify incidents and maintenance events from dedicated pages, while preserving admin-only deletion and adding clear audit information on detail pages.

The feature must support editing event text in any enabled language without changing language-independent event data or deleting other translations.

## Terminology

- `Incident`: an event stored in the existing incident domain and shown in the incidents area.
- `Maintenance event`: an event stored in the existing maintenance domain and shown in the events/maintenance area.
- `Event`: generic wording in this PRD for both incidents and maintenance events when requirements apply to both.
- `CO_OWNERSHIP_BOARD`: new role code granting creation and modification rights for incidents and maintenance events.
- English role label: `Co-ownership board`.
- French role label: `Conseil syndical`.
- `ADMIN`: existing role. Deletion remains admin-only.
- `CO_OWNER`: existing role for reading co-owner content. This PRD does not replace it.
- `Category`: shared reference data used by incidents and maintenance events. A category has an immutable id, editable code, icon id, and one localized label per enabled locale.
- `Edit locale`: the language chosen in the create/edit form for localized text fields.
- `UI locale`: the current application language selected by the user.

## Goals

- Add a new assignable role for co-ownership board members.
- Treat `ADMIN`, `CO_OWNER`, and `CO_OWNERSHIP_BOARD` as read roles for incident and maintenance content.
- Let users with `CO_OWNERSHIP_BOARD` create incidents and maintenance events.
- Let users with `CO_OWNERSHIP_BOARD` modify incidents and maintenance events.
- Keep deletion restricted to `ADMIN` only.
- Add shared admin-managed categories for incidents and maintenance events.
- Let admins create, update, and delete unused categories from a dedicated admin page.
- Use dedicated create and edit routes/pages, not modals.
- Add create/edit/delete actions in the UI only when the current user has the required role.
- Add last-modified audit data to event detail responses and pages.
- Let users choose which supported language version they are editing from a drop-down list.
- Default the edit locale to the user's UI locale.
- Preserve other language versions when creating or modifying one language version.
- When a selected edit locale has no translation for a field, prefill that field with a fallback language value so the user can translate from it.

## Non-Goals

- No modal-based create or edit forms.
- No attachment upload or attachment management changes.
- No rich text editor unless the app already has one before implementation.
- No approval workflow.
- No soft-delete or recycle-bin requirement.
- No role management UI redesign beyond making the new role available through the existing admin roles mechanism.
- No inline category creation from event create/edit forms.
- No category history snapshots. Event pages display the current category code, icon, and label through normalized category joins.
- No runtime language administration. The language selector uses enabled locales already supported by the backend.
- No automatic machine translation.
- No requirement for `ADMIN` to imply create/edit rights. Admin users need `CO_OWNERSHIP_BOARD` too if they should create or modify events.

## Roles And Permissions

### Read Access

- Incident and maintenance listing/detail endpoints must allow authenticated users with any of these roles: `ADMIN`, `CO_OWNER`, or `CO_OWNERSHIP_BOARD`.
- Incident and maintenance frontend read routes must allow authenticated users with any of these roles: `ADMIN`, `CO_OWNER`, or `CO_OWNERSHIP_BOARD`.
- `CO_OWNERSHIP_BOARD` implies everything `CO_OWNER` can do for incident and maintenance read access.
- `ADMIN` implies read access for incident and maintenance content so admins can reach detail pages and perform admin-only deletion from the UI.

### Create And Modify Access

- `CO_OWNERSHIP_BOARD` is required to create incidents.
- `CO_OWNERSHIP_BOARD` is required to update incidents.
- `CO_OWNERSHIP_BOARD` is required to create maintenance events.
- `CO_OWNERSHIP_BOARD` is required to update maintenance events.
- Backend authorization is mandatory for all create/update endpoints.
- Frontend route guards and button visibility are usability protections only and must not be relied on for security.
- `ADMIN` alone must not satisfy create/update authorization unless the user also has `CO_OWNERSHIP_BOARD`.

### Delete Access

- `ADMIN` is required to delete incidents.
- `ADMIN` is required to delete maintenance events.
- `CO_OWNERSHIP_BOARD` must not be able to delete events unless the user also has `ADMIN`.
- Delete actions must be hidden for non-admin users.
- Direct API delete calls by non-admin users must return forbidden according to existing API conventions.

### Role Catalog

- Add `CO_OWNERSHIP_BOARD` to the backend role catalog.
- Add it to assignable non-admin roles.
- Return it from the existing available roles endpoint.
- Add frontend i18n labels for `roles.coOwnershipBoard` in English and French.
- The existing admin user roles UI must be able to assign and remove this role through the existing non-admin role editing flow.

## Category Management

### Category Model

Categories are shared by incidents and maintenance events.

Each category has:

- `id`: immutable stable identifier used as the database primary key and event foreign key.
- `code`: editable unique business/display code.
- `icon`: non-empty string intended to match a Lucide icon name.
- `labels`: one required localized display label for every enabled backend locale.

Requirements:

- Category `id` is required on create and cannot be edited after creation.
- Category `code` is required, unique, and editable.
- Category `icon` is required and editable.
- Category labels are required for all enabled backend locales.
- Category labels have one field only: display label. No category description/help text is included.
- Backend icon validation only checks that the value is non-empty after trimming.
- Backend must not validate the icon against a Lucide allow-list.
- Frontend resolves the icon string against the icon library it uses and falls back to a default icon when the icon cannot be resolved.
- Category admin UI shows helper text linking to `https://lucide.dev/icons/` so admins can find valid Lucide icon names.

### Category Admin Page

- Add a dedicated admin-only category management page at `/admin/categories`.
- Add an admin navigation entry for category management.
- Only users with `ADMIN` can access category CRUD UI and APIs.
- Category management is separate from event create/edit forms.
- Event create/edit forms only select existing categories.
- If a needed category is missing, users must ask an admin to create it.

Category admin page supports:

- Listing categories.
- Creating categories.
- Editing category `code`, `icon`, and localized labels.
- Deleting categories that are not referenced by any incident or maintenance event.
- Showing a clear error when deletion is blocked because a category is in use.

### Category Deletion

- Category deletion is hard deletion.
- A category can be deleted only when no incident and no maintenance event references it.
- Backend must reject deletion of a referenced category.
- No soft-delete or disabled/archive state is required.

### Category Localization

- Category labels are complete reference data and must exist for every enabled backend locale.
- If a new backend locale is enabled later, existing category labels for that locale are backfilled from the English label.
- Event translations are not backfilled when a new locale is enabled; event edit pages continue to use fallback prefill until a user saves that locale.

### Category Display

- Event list and detail responses should include category display data from normalized category joins, not denormalized event snapshots.
- If a category code, label, or icon changes, existing events display the current category values.
- Historical preservation of old category values is out of scope.
- Category labels in event responses follow the requested display locale with fallback behavior.

## User Experience

### Listing Pages

- Incident listing page shows a create action when the current user has `CO_OWNERSHIP_BOARD`.
- Maintenance listing page shows a create action when the current user has `CO_OWNERSHIP_BOARD`.
- Create actions navigate to dedicated pages:
  - Incidents: `/incidents/new`
  - Maintenance events: `/events/new`
- Listing pages must not show create actions while roles are still loading.
- Listing pages must not show create actions for users without `CO_OWNERSHIP_BOARD`.

### Detail Pages

- Incident detail page shows an edit action when the current user has `CO_OWNERSHIP_BOARD`.
- Maintenance detail page shows an edit action when the current user has `CO_OWNERSHIP_BOARD`.
- Incident detail page shows a delete action when the current user has `ADMIN`.
- Maintenance detail page shows a delete action when the current user has `ADMIN`.
- Edit actions navigate to dedicated pages:
  - Incidents: `/incidents/:id/edit`
  - Maintenance events: `/events/:id/edit`
- Delete may use a small confirmation UI, but the create/edit forms themselves must not be modals.
- Detail pages display audit text when event data is available.

Audit display format:

- English: `Last modified at {date} by {user}`.
- French: `Derniere modification le {date} par {user}`.
- Dates use the existing frontend date formatting utilities.
- `{user}` is the modifying user's email when available.
- If historical data has no recorded modifying user, show a localized unknown-user label rather than omitting the audit line.

### Create Pages

- Create pages are dedicated route pages, not modals.
- Create pages include a language drop-down for the edit locale.
- The edit locale defaults to the current UI locale.
- The language drop-down lists enabled backend locales supported by the event translation model.
- The language drop-down uses all enabled backend locales and is not coupled to the static frontend UI translation list.
- The form contains language-independent fields and localized text fields.
- Saving creates the event and writes localized fields for the selected edit locale.
- After successful save, navigate to the new event detail page.
- The detail page should be opened in the same UI locale as the current app unless the user changes the global language.

### Edit Pages

- Edit pages are dedicated route pages, not modals.
- Edit pages include a language drop-down for the edit locale.
- The edit locale defaults to the current UI locale.
- Changing the edit locale changes which localized field values are loaded into the form.
- Changing the edit locale must not mutate the event by itself.
- Changing the edit locale must not change the event id, category, timestamps, status grouping, or any canonical event language, because events do not have a separate canonical language in the current model.
- Saving writes language-independent fields and localized fields for the selected edit locale only.
- Saving must preserve translations for all other locales.
- After successful save, navigate back to the event detail page.

### Missing Translation Prefill

- For each localized field, if the selected edit locale has an exact translation, show that exact translation in the input.
- If the selected edit locale has no exact translation for a field, show the best fallback value from another locale in the input.
- The fallback order should match existing backend locale fallback behavior unless a field-specific exact/fallback response provides a clearer source.
- The UI must indicate when an input is prefilled from another language, for example with helper text such as `Prefilled from English` or `Prefilled from French`.
- Saving writes the visible input values as translations for the selected edit locale.
- If a user clears an optional localized field and saves, the selected locale value for that field should be removed or omitted according to the backend persistence design, while other locales remain unchanged.
- Required localized fields must not save as empty strings.

## Form Fields

### Shared Language-Independent Fields

Incident and maintenance forms include:

- `id` / code.
- `category_id` selected from the shared category catalog.
- `start_utc`.
- `end_utc`.

Requirements:

- `id` is required on create.
- `id` is stable after creation and must not be editable on edit pages.
- `category_id` is required and editable after event creation.
- Event forms show enabled/current categories from the category catalog in a drop-down.
- Event forms must not allow inline category creation or editing.
- `start_utc` is required.
- `end_utc` is optional.
- If `end_utc` is provided, it must not be earlier than `start_utc`.
- Date/time inputs should use native date/time controls or existing app patterns, converting to RFC3339 UTC payload values for the API.

### Maintenance-Specific Fields

Maintenance forms include:

- `notified_at_utc`, optional.
- `title`, localized and required.
- `short_description`, localized and required.
- `long_description`, localized and required.
- `warning`, localized and optional.
- `location`, localized and optional.

### Incident-Specific Fields

Incident forms include:

- `title`, localized and required.
- `short_description`, localized and required.
- `long_description`, localized and required.
- `location`, localized and optional.
- Timeline entries.

### Incident Timeline Fields

- Incident edit pages must support editing existing timeline entries because timeline text is part of incident detail data.
- Incident create pages may start with zero timeline entries and allow adding entries.
- Timeline entries include:
  - `id`, stable identifier.
  - `at_utc`, required.
  - `sort_order`, required or derived from displayed order.
  - `title`, localized and required.
  - `details`, localized and optional.
- Users can add and remove timeline entries while creating or editing an incident.
- Removing a timeline entry during edit removes that entry across all languages because timeline entries are language-independent objects with localized fields.
- Editing a timeline entry's localized fields writes only the selected edit locale for that entry.
- Editing a timeline entry's `at_utc` or order updates language-independent data for that entry.

## Backend Requirements

### Authorization

- `GET /incidents` requires one of `ADMIN`, `CO_OWNER`, or `CO_OWNERSHIP_BOARD`.
- `GET /incidents/{id}` requires one of `ADMIN`, `CO_OWNER`, or `CO_OWNERSHIP_BOARD`.
- `POST /incidents` requires `CO_OWNERSHIP_BOARD`.
- `PUT /incidents/{id}` requires `CO_OWNERSHIP_BOARD`.
- `DELETE /incidents/{id}` requires `ADMIN`.
- `GET /maintenances` requires one of `ADMIN`, `CO_OWNER`, or `CO_OWNERSHIP_BOARD`.
- `GET /maintenances/{id}` requires one of `ADMIN`, `CO_OWNER`, or `CO_OWNERSHIP_BOARD`.
- `POST /maintenances` requires `CO_OWNERSHIP_BOARD`.
- `PUT /maintenances/{id}` requires `CO_OWNERSHIP_BOARD`.
- `DELETE /maintenances/{id}` requires `ADMIN`.
- Admin category CRUD endpoints require `ADMIN`.
- Category list endpoints used by event forms require one of `ADMIN` or `CO_OWNERSHIP_BOARD`.
- Any edit-helper endpoints that expose raw translations for event editing require `CO_OWNERSHIP_BOARD` or `ADMIN`.
- Existing admin translation endpoints may remain admin-only, but the new event edit flow must not depend on an admin-only endpoint.

### Role Enforcement Helpers

- Backend authorization must support OR checks for endpoints that allow multiple roles.
- Read endpoints use `ADMIN OR CO_OWNER OR CO_OWNERSHIP_BOARD`.
- Category list endpoints for event forms use `ADMIN OR CO_OWNERSHIP_BOARD` unless also needed by read pages.
- Create/update endpoints use `CO_OWNERSHIP_BOARD`.
- Delete endpoints use `ADMIN`.

### Current Principal

- Backend request principal data must include enough user identity to write audit fields.
- At minimum, mutation handlers need the authenticated user's id.
- Detail responses need the last modifying user's display value, preferably email.

### Audit Persistence

- Add audit columns or equivalent persisted fields for incidents and maintenances:
  - `last_modified_at TIMESTAMPTZ`.
  - `last_modified_by_user_id UUID REFERENCES users(id)`.
- New create/update operations set `last_modified_at` to now.
- New create/update operations set `last_modified_by_user_id` to the authenticated user id.
- Updating only translations still updates audit fields.
- Updating only language-independent fields still updates audit fields.
- Updating incident timeline entries still updates the parent incident audit fields.
- Existing rows should be backfilled with `last_modified_at = updated_at` where possible.
- Existing rows with unknown modifying user may keep `last_modified_by_user_id = NULL`.
- Deletion does not need to preserve audit data unless the implementation introduces soft delete, which is out of scope.

### Detail Responses

Incident detail responses include:

```json
{
  "category": {
    "id": "HEA",
    "code": "HEA",
    "icon": "flame",
    "label": "Heating"
  },
  "last_modified_at": "2026-06-06T12:00:00Z",
  "last_modified_by": {
    "id": "user-id",
    "email": "user@example.com"
  }
}
```

Maintenance detail responses include the same audit fields.

Requirements:

- List responses should also include category display data, or enough fields for the frontend to display category code/icon/label without hardcoding category metadata.
- `last_modified_at` may be null only for legacy rows that cannot be backfilled.
- `last_modified_by` may be null for legacy rows with unknown modifying user.
- If `last_modified_by` is null, frontend displays a localized unknown-user label.
- API responses must not expose more user data than id and email for this audit purpose.

### Category APIs

Provide backend endpoints for category selection and admin CRUD.

Recommended contracts:

```http
GET /categories?locale=fr
GET /admin/categories?locale=fr
POST /admin/categories
PUT /admin/categories/{id}
DELETE /admin/categories/{id}
```

Category list response item:

```json
{
  "id": "HEA",
  "code": "HEA",
  "icon": "flame",
  "label": "Chauffage",
  "labels": {
    "en": "Heating",
    "fr": "Chauffage"
  }
}
```

Requirements:

- Public unauthenticated category access is not required.
- `/categories` is for authorized app usage, especially event forms and event display support.
- `/admin/categories` returns full label maps for admin editing.
- Category create requires `id`, `code`, `icon`, and labels for all enabled locales.
- Category update allows changing `code`, `icon`, and labels, but not `id`.
- Category delete hard-deletes only unreferenced categories.
- Category delete returns a validation/conflict error when the category is referenced by any incident or maintenance event.

### Edit Data Responses

The edit UI needs exact selected-locale values and fallback metadata. The implementation may use one of these approaches:

- Add dedicated edit-data endpoints.
- Extend existing detail endpoints with an `include_translations=true` or `mode=edit` query.
- Reuse existing translation matrix endpoints after changing authorization and response shape as needed.

Recommended dedicated endpoints:

```http
GET /incidents/{id}/edit?locale=fr
GET /maintenances/{id}/edit?locale=fr
```

Each edit-data response must include:

- Language-independent fields.
- Selected edit locale.
- Enabled locales available for editing.
- For each localized field:
  - `field_key`.
  - `value`, the exact selected-locale value if present, otherwise fallback value.
  - `exact_value`, nullable exact selected-locale value.
  - `fallback_locale`, nullable locale used when exact value is missing.
  - `fallback_value`, nullable fallback value used when exact value is missing.
- For incidents, timeline entries with the same selected-locale/fallback metadata for timeline localized fields.

Example field shape:

```json
{
  "field_key": "title",
  "value": "Maintenance chauffage en cours",
  "exact_value": null,
  "fallback_locale": "fr",
  "fallback_value": "Maintenance chauffage en cours"
}
```

### Create And Update Payloads

Create/update payloads must identify the selected edit locale separately from the event data.

Recommended maintenance save payload:

```json
{
  "id": "HEA-001",
  "category_id": "HEA",
  "start_utc": "2026-06-06T08:00:00Z",
  "end_utc": null,
  "notified_at_utc": null,
  "locale": "fr",
  "fields": {
    "title": "Maintenance chauffage",
    "short_description": "Intervention planifiee",
    "long_description": "Details de l'intervention",
    "warning": "",
    "location": "Batiment B"
  }
}
```

Recommended incident save payload:

```json
{
  "id": "INC-001",
  "category_id": "HEA",
  "start_utc": "2026-06-06T08:00:00Z",
  "end_utc": null,
  "locale": "fr",
  "fields": {
    "title": "Panne chauffage",
    "short_description": "Baisse de pression",
    "long_description": "Details de l'incident",
    "location": "Chaufferie"
  },
  "timeline": [
    {
      "id": "timeline-entry-id",
      "at_utc": "2026-06-06T08:15:00Z",
      "sort_order": 1,
      "fields": {
        "title": "Incident detecte",
        "details": "Signalement recu"
      }
    }
  ]
}
```

Requirements:

- The backend validates the locale is enabled.
- The backend validates that `category_id` exists.
- The backend validates field keys are supported for the event type.
- The backend writes only the provided selected-locale field values for localized event fields.
- The backend preserves all non-selected locale values.
- The backend does not delete all translations during an ordinary edit.
- The backend may delete selected-locale optional field translations when submitted as empty, but must not delete other locales.
- The backend must reject invalid timestamps, unknown category ids, invalid ids, and unsupported locales.

### Compatibility With Existing Upsert Code

- Current full-replace upsert behavior is not acceptable for the new edit flow if it deletes translations for other locales.
- Implementations may replace the current upsert contract or add new create/update service functions.
- If legacy full-translation upsert endpoints remain for internal/admin use, they must not be used by the new co-ownership board edit pages unless they are made safe for partial language edits.

## Frontend Requirements

### Routes

- Existing read routes `/events`, `/events/:id`, `/incidents`, and `/incidents/:id` require one of `ADMIN`, `CO_OWNER`, or `CO_OWNERSHIP_BOARD`.
- Add `/events/new`, requiring `CO_OWNERSHIP_BOARD`.
- Add `/events/:id/edit`, requiring `CO_OWNERSHIP_BOARD`.
- Add `/incidents/new`, requiring `CO_OWNERSHIP_BOARD`.
- Add `/incidents/:id/edit`, requiring `CO_OWNERSHIP_BOARD`.
- Add `/admin/categories`, requiring `ADMIN`.
- Existing detail routes remain unchanged.
- If a user without `CO_OWNERSHIP_BOARD` navigates directly to create/edit routes, use the existing access-denied pattern.

### Route Guards

- Frontend route authorization must support OR role requirements.
- Read routes use `ADMIN OR CO_OWNER OR CO_OWNERSHIP_BOARD`.
- Create/edit routes use `CO_OWNERSHIP_BOARD`.
- Admin category routes use `ADMIN`.

### Repositories

- Extend events and incidents repositories with create/update/delete methods.
- Extend repositories with edit-data loading methods or equivalent calls.
- Add a category repository/client for category list and admin CRUD operations.
- Repository methods must include authorization headers consistently with existing API calls.
- Mock repositories used in tests must support the new methods enough for component and route tests.

### Forms

- Prefer shared form components or composables only where they reduce duplication without obscuring incident-specific timeline behavior.
- Event forms load categories from the backend and display them in a drop-down using the current edit/display locale label, current code, and icon where appropriate.
- Event forms save `category_id`, not category code.
- Category admin forms require labels for all enabled backend locales.
- Category admin icon input includes help text linking to `https://lucide.dev/icons/`.
- All user-facing labels, helper text, buttons, errors, and empty states use the existing Vue i18n system.
- Form errors should be shown near the relevant field when validation is local.
- API save errors should be shown at form level using existing error-state styling.
- Save buttons disable while saving.
- Cancel/back actions return to the relevant listing or detail page without saving.

### Delete Flow

- Delete action is available from detail pages for `ADMIN` users.
- Delete action must ask for confirmation before calling the API.
- After successful delete, navigate to the relevant listing page.
- Delete failure shows a localized error on the detail page.

### Icon Rendering

- Category icons are stored as strings intended to match Lucide icon names.
- Frontend attempts to resolve the stored icon string through the icon library already used by the app.
- If an icon cannot be resolved, frontend shows a default fallback icon.
- Unknown icon strings must not break list, detail, form, or admin category pages.

## Internationalization Requirements

Add English and French translations for at least:

- New role label `roles.coOwnershipBoard`.
- Create event action.
- Create incident action.
- Edit action.
- Delete action.
- Delete confirmation title/message.
- Delete success/failure messages if displayed.
- Save, saving, cancel, and back labels if not already reusable.
- Form labels for all fields.
- Language selector label for edit locale.
- Missing-translation helper text.
- Unknown user audit label.
- Last-modified audit sentence.
- Create/update load and save error messages.
- Add/remove timeline entry labels.
- Admin category navigation label.
- Category admin page labels, create/edit/delete actions, validation errors, and referenced-category delete error.
- Category icon helper text with the Lucide icon documentation link.
- Category field labels: id, code, icon, and localized labels.

## Data Migration Requirements

- Add a migration for `CO_OWNERSHIP_BOARD` only if roles become database-backed. If roles remain static backend code, no role migration is required.
- Add shared category tables for category metadata and category i18n labels.
- Seed categories for existing incident and maintenance category values.
- Existing category values such as `HEA`, `ELV`, and `PLB` should become initial category ids and initial category codes to preserve existing event references during migration.
- Future categories may use generated stable ids; ids do not need to match editable codes after creation.
- Migrate incidents and maintenances to reference category ids.
- Event queries should join categories for current category code, icon, and localized label display. Do not denormalize category display fields into event rows.
- If a new locale is enabled later, existing category labels for that locale are backfilled from the English label.
- Do not backfill event translations for newly enabled locales.
- Add incident audit columns.
- Add maintenance audit columns.
- Backfill `last_modified_at` from existing `updated_at` for incidents and maintenances.
- Keep `last_modified_by_user_id` null for historical rows where the user is unknowable.
- Add indexes for audit user joins only if query plans require them; detail pages fetch by event code and can join by primary key.

## Security Requirements

- Backend authorization checks are required for every create, update, delete, edit-data, and raw translation endpoint used by this feature.
- Frontend role checks must never be the only protection.
- Users without `CO_OWNERSHIP_BOARD` cannot create or update events through direct API calls.
- Users without `ADMIN` cannot delete events through direct API calls.
- Users without `ADMIN` cannot create, update, or delete categories through direct API calls.
- Category deletion must be blocked when referenced by any incident or maintenance event.
- Event create/update must reject unknown category ids.
- Mutation endpoints must use authenticated user identity from the validated session, never from request payloads, for audit fields.
- The backend must not accept arbitrary `last_modified_by` or `last_modified_at` values from clients.
- The backend must not expose full user records in event audit responses.

## Acceptance Criteria

- Admins can assign `CO_OWNERSHIP_BOARD` from the existing admin user roles view.
- Users with `ADMIN`, `CO_OWNER`, or `CO_OWNERSHIP_BOARD` can access incident and maintenance read routes and read endpoints.
- Users without all three read roles cannot access incident and maintenance read routes or read endpoints.
- Users with `CO_OWNERSHIP_BOARD` see create actions on incident and maintenance listing pages after roles load.
- Users without `CO_OWNERSHIP_BOARD` do not see create actions.
- Users with `CO_OWNERSHIP_BOARD` see edit actions on incident and maintenance detail pages after roles load.
- Users without `CO_OWNERSHIP_BOARD` do not see edit actions.
- Users with `ADMIN` see delete actions on incident and maintenance detail pages.
- Users without `ADMIN` do not see delete actions.
- Direct create/update API calls fail for users without `CO_OWNERSHIP_BOARD`.
- Direct delete API calls fail for users without `ADMIN`.
- Admins can access `/admin/categories`.
- Non-admin users cannot access `/admin/categories` or category admin APIs.
- Admins can create categories with id, editable code, icon, and labels for every enabled locale.
- Admins can edit category code, icon, and labels, but not category id.
- Admins can hard-delete categories only when they are unused.
- Deleting a referenced category fails with a clear error.
- Event create/edit forms load category options from the backend.
- Event create/edit forms save category id and reject unknown category ids.
- Event list/detail pages display current category code, label, and icon from category joins.
- Unknown category icon strings render a default fallback icon.
- Create and edit use dedicated pages, not modals.
- Create defaults the edit locale to the current UI locale.
- Edit defaults the edit locale to the current UI locale.
- The language drop-down lists enabled supported locales.
- Editing in another locale does not change language-independent event data until save.
- Changing the edit locale does not persist anything by itself.
- Saving an edit in one locale preserves translations in other locales.
- Missing selected-locale fields are prefilled from a fallback locale.
- Prefilled fallback fields are visibly identified as fallback values.
- Saving prefilled fields creates selected-locale translations with the visible values.
- Category labels are required for all enabled backend locales.
- Event language selectors use all enabled backend locales and are not coupled to the current static frontend UI locale list.
- Required localized fields cannot be saved empty.
- Incident timeline entries can be added, edited, ordered, and removed from incident create/edit pages.
- Detail pages display `Last modified at ... by ...` or the French equivalent.
- Create and update operations record the authenticated modifying user.
- Existing historical events display an unknown-user audit fallback when no modifying user is known.
- All new user-facing text is translated in English and French through the existing i18n mechanism.
- Frontend lint, tests, and build pass after implementation.
- Backend clippy, tests, and build pass after implementation.

## Implementation Notes

- Existing routes already expose create/update/delete handlers, but create/update are currently admin-gated and use full upsert payloads. The implementation must adjust permissions and avoid losing translations during selected-locale edits.
- Existing detail APIs return fallback-resolved strings only. The edit form needs exact-versus-fallback metadata, so additional edit-data support is required.
- Existing `Principal` currently exposes roles only. Audit requires adding user id, and likely email, to principal or fetching it in mutation/detail paths.
- Existing enabled locales are stored in the `locales` table. The edit language drop-down should use enabled locales rather than a hardcoded frontend list if a suitable endpoint exists or is added.
- Existing single-role authorization helpers need OR-role support for read endpoints and frontend route guards.
- Existing `category_code` event storage can migrate by using current codes as initial category ids and category codes, then joining category metadata for display.

## Open Questions

No blocking product gaps remain. The PRD fixes defaults for role code, OR-role read authorization, audit fallback, edit locale behavior, missing translation behavior, partial translation persistence, category CRUD, category icon handling, and normalized category display.
