# Projects PRD

## Goal

Add a third top-level section named `Projects`, alongside maintenance/events and incidents, to track co-ownership projects through planning, active execution, and completion.

Projects must be visible in their own listing and detail pages, editable by the same roles that can manage maintenance/incidents, and readable by the same roles that can view maintenance/incidents.

## Users And Permissions

Readable by:

- `ADMIN`
- `CO_OWNER`
- `CO_OWNERSHIP_BOARD`

Creatable, editable, and deletable by:

- `ADMIN`
- `CO_OWNERSHIP_BOARD`

This matches the current permission matrix for maintenance and incidents.

## Navigation

Add a `Projects` navigation entry.

Routes:

- `/projects`
- `/projects/new`
- `/projects/:id`
- `/projects/:id/edit`

## Data Model

Each project has:

| Field | Requirement |
| --- | --- |
| `id` | Required, user-entered unique project ID, like current event/incident IDs. |
| `category_id` / `category_code` | Required, reuses existing admin-managed categories. |
| `title` | Required, localized. |
| `description` | Required, localized markdown source. |
| `start_utc` | Optional. |
| `end_utc` | Optional. |
| `status` | Required stored enum: `waiting` or `ongoing`. |
| `last_modified_at` | Backend-maintained audit timestamp. |
| `last_modified_by` | Backend-maintained audit user. |
| `attachments` | Returned as an empty list/block for now; upload and management are out of scope. |

No stored `finished` status. A project is considered finished when `end_utc` exists and is in the past.

## Listing Sections

The Projects listing is split into three sections, in this order:

1. `Ongoing Projects`: projects that are not finished and are currently active.
2. `Projects To Come`: projects that are not finished and are not yet active, including projects with no dates yet.
3. `Finished Projects`: projects whose `end_utc` is in the past.

Classification rules:

| Case | Section |
| --- | --- |
| `end_utc` exists and is before now | Finished |
| `start_utc` exists and is before or equal to now and project is not finished | Ongoing |
| Stored `status = ongoing` and project is not finished | Ongoing |
| Otherwise | To Come |

This supports projects with no dates yet while allowing editors to manually mark a date-less project as ongoing.

## Listing Card Content

Each project card shows:

| Content | Notes |
| --- | --- |
| ID and category code | Same pattern as incident listing. |
| Title | Links to detail page. |
| Date label | Shows available start/end dates and handles missing dates gracefully. |
| Status icon and text | Similar position and weight to the latest incident timeline entry. |

Project descriptions are not shown on listing cards in v1.

Status display values:

| Display state | Source | Suggested icon |
| --- | --- | --- |
| Waiting | Stored `waiting`, not finished. | Hourglass. |
| Ongoing | Stored or date-derived ongoing, not finished. | Activity. |
| Finished | Derived from `end_utc`. | Check-circle. |

## Card Visual Treatment

Finished project cards use the same faded visual treatment as past incident/maintenance cards for coherence.

Non-finished cards get distinct color accents:

| Section | Suggested treatment |
| --- | --- |
| Ongoing | Blue/teal accent, active but not alarming. |
| To Come | Amber/purple accent, planned/upcoming. |
| Finished | Existing faded/past treatment. |

The final colors should fit the current theme variables and remain readable in light and dark modes.

## Detail View

Project detail page shows:

| Content | Notes |
| --- | --- |
| Back link | Back to projects, preserving search query if applicable. |
| ID/category | Same style as existing detail pages. |
| Title | Localized. |
| Date label | Supports missing start/end. |
| Status icon/text | Uses the same display mapping as listing. |
| Description | Markdown rendered to safe HTML. |
| Attachments block | Always visible; displays `No attachments available` for now. |
| Last modified audit | Same pattern as event/incident detail. |
| Edit/delete actions | Only for allowed roles. |

Markdown scope:

- The edit form stores markdown source in `description`.
- The detail view renders sanitized HTML.
- The listing view does not render markdown.

## Create/Edit Form

Fields:

| Field | Requirement |
| --- | --- |
| Edit language | Same localized edit pattern as current forms. |
| Project ID | Required on create, disabled on edit. |
| Category | Required, existing categories. |
| Start | Optional datetime. |
| End | Optional datetime. |
| Status | Required select: `waiting` or `ongoing`. |
| Title | Required. |
| Description | Required markdown textarea. |

No markdown preview in v1.

Validation:

| Rule | Behavior |
| --- | --- |
| ID missing | Client and backend reject. |
| Category missing | Client and backend reject. |
| Title missing | Client and backend reject. |
| Description missing | Client and backend reject. |
| Status not `waiting`/`ongoing` | Backend rejects. |
| End before start when both present | Backend rejects. |
| Unknown category | Backend rejects via existing category relation. |

## Backend API

Add endpoints:

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/projects?locale=&q=` | List/search projects. |
| `POST` | `/projects` | Create project. |
| `GET` | `/projects/:id` | Project detail. |
| `PUT` | `/projects/:id` | Update project. |
| `DELETE` | `/projects/:id` | Delete project. |
| `GET` | `/projects/:id/edit?locale=` | Edit data with fallbacks. |
| `GET` | `/projects/:id/translations` | Translation matrix. |
| `POST` | `/projects/:id/translations/replace` | Replace translations. |

Follow the existing incidents/maintenance repository, service, and HTTP structure.

## Search

Project search matches:

- ID
- Title
- Description
- Category code
- Category localized label
- Status text

## Internationalization

Add English and French labels for:

- Projects navigation
- Create project
- Edit project
- Project not found/load failed
- Search projects
- No projects match
- Ongoing projects
- Projects to come
- Finished projects
- Project ID
- Project status
- Waiting
- Ongoing
- Finished
- Description

Existing generic labels like category, start, end, save, delete, attachments, and last modified should be reused.

## Out Of Scope

- Attachment upload/management
- Markdown editor toolbar
- Markdown preview in edit form
- Comments/discussions
- Project timeline/milestones
- Custom project categories separate from existing categories

## Testing

Frontend checks:

- `npm run lint`
- `npm run test`
- `npm run build`

Backend checks:

- `cargo clippy --all-features -- -D warnings`
- `cargo test --all-features`
- `cargo build --all-features`

Test coverage should include:

| Area | Tests |
| --- | --- |
| Project classification | Optional dates, finished derivation, status fallback. |
| Listing | Three sections, status visible, past fading. |
| Detail | Markdown rendered safely, attachments empty block. |
| Permissions | Routes and actions match maintenance/incidents. |
| Backend validation | Status enum, required fields, date ordering. |
| CRUD | Create/list/detail/update/delete. |
