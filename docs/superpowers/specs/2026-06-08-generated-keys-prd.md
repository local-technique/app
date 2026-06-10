# Generated Public Keys PRD

## Purpose

Remove user-entered public identifiers for maintenance events, incidents, and projects. Generate their readable public keys atomically in PostgreSQL so users cannot break naming conventions or create avoidable collisions.

Keep the clean readable structure already used by the application:

- Maintenance events: `EVT-X`
- Incidents: `INC-X`
- Projects: `PRJ-X`

No zero padding is used.

Clarify naming across the codebase:

- `id` means an internal technical identifier, normally a UUID primary key used for joins.
- `key` means a user-facing readable identifier used in URLs, labels, search, and display.

## Scope

In scope:

- Maintenance event public key generation.
- Incident public key generation.
- Project public key generation.
- Category naming cleanup from `code` to `key`.
- Category internal ID migration to UUIDs.
- Frontend create form updates that remove public key inputs.
- API payload and response updates that expose `key` for public identifiers and keep UUID `id` for technical identifiers.

Out of scope:

- Renumbering existing maintenance, incident, or project public keys.
- Changing incident timeline entry UUID behavior.
- Adding human-friendly slugs beyond the required public keys.
- Backward-compatible API aliases such as accepting both `code` and `key`.
- Changing route shapes beyond keeping detail/edit/update/delete URLs keyed by the public key.

## Terminology

- `Internal ID`: a technical UUID primary key used by the database and joins. It is not meant for end-user workflows.
- `Public key`: a clean readable identifier exposed to users and used in URLs.
- `Category key`: the user-chosen 3 to 5 character category identifier, replacing current category `code` semantics.
- `Maintenance event`: the current maintenance domain shown in the frontend as events/maintenance.

## Chosen Approach

Use table-local PostgreSQL sequences for generated public keys.

Sequences:

- `maintenance_key_seq` for maintenance events.
- `incident_key_seq` for incidents.
- `project_key_seq` for projects.

Each sequence starts at `10`, unless existing preserved keys require a higher next value.

Generated values:

- `EVT-` plus `nextval('maintenance_key_seq')`.
- `INC-` plus `nextval('incident_key_seq')`.
- `PRJ-` plus `nextval('project_key_seq')`.

This approach is atomic, keeps SQL simple, avoids race-prone `max + 1` calculations, and does not block future repository code from using PostgreSQL upserts where appropriate.

## Alternatives Considered

### Insert-Time Max Suffix Plus One

The insert could compute the next number from the current highest matching key. This avoids sequences but is harder to make race-safe, requires locking or retry behavior, and makes insert SQL more complex.

### Backend Pre-Allocation

The backend could request a number before insert, then build and insert the key. This can still be safe with database sequences, but it spreads key generation across service code and creates a less direct create flow than returning the inserted row.

## Data Model

### Maintenance Events

- `maintenances.id` remains the internal UUID primary key.
- `maintenances.code` is renamed or migrated to `maintenances.key`.
- `maintenances.key` is unique, immutable, and user-facing.
- New maintenance keys are generated as `EVT-10`, `EVT-11`, and so on.

### Incidents

- `incidents.id` remains the internal UUID primary key.
- `incidents.code` is renamed or migrated to `incidents.key`.
- `incidents.key` is unique, immutable, and user-facing.
- New incident keys are generated as `INC-10`, `INC-11`, and so on.

### Projects

- `projects.id` remains the internal UUID primary key.
- `projects.code` is renamed or migrated to `projects.key`.
- `projects.key` is unique, immutable, and user-facing.
- New project keys are generated as `PRJ-10`, `PRJ-11`, and so on.

### Categories

- `event_categories.id` becomes an internal UUID primary key.
- `event_categories.code` is renamed or migrated to `event_categories.key`.
- `event_categories.key` is user-chosen, unique, and displayed to users.
- Category keys are 3 to 5 ASCII alphanumeric characters.
- Existing category values such as `HEA` and `ELV` become category keys.
- Incidents, maintenances, and projects reference categories by internal UUID ID, not by category key.

## Routes And API Shape

### Public Routes

Maintenance event, incident, and project detail/edit/update/delete URLs use public keys, not internal UUIDs.

Examples:

- `/events/EVT-10`
- `/incidents/INC-10`
- `/projects/PRJ-10`

Backend handlers resolve route keys to internal UUID records before joining translation and timeline tables.

### Create Requests

Maintenance event, incident, and project create payloads do not include a public key. The backend generates the key during insert.

Category create payloads include a user-chosen `key` and do not include a user-entered `id`.

### Create Responses

Maintenance event, incident, and project create endpoints return the generated public key needed by the frontend.

Examples:

```json
{ "key": "EVT-10" }
```

```json
{ "key": "INC-10" }
```

```json
{ "key": "PRJ-10" }
```

Category create endpoints return the created category item, including its generated internal UUID `id` and user-chosen public `key`, so the admin list can refresh without guessing either value.

### Update Requests

Maintenance event, incident, and project update endpoints use the route key to identify the record. Payloads must not allow changing the key.

Category update endpoints identify the category by internal UUID ID and may update the category key, subject to uniqueness and format validation. This is safe because event/project/incident references point to the internal category UUID.

## Validation

Generated public keys must match these patterns:

- Maintenance events: `EVT-[1-9][0-9]*`
- Incidents: `INC-[1-9][0-9]*`
- Projects: `PRJ-[1-9][0-9]*`

Category keys:

- Trim surrounding whitespace.
- Normalize to uppercase.
- Allow only ASCII alphanumeric characters.
- Require length between 3 and 5 characters inclusive.
- Require uniqueness.

Backend validation is the source of truth. Frontend validation can provide immediate feedback but must not be relied on for correctness.

## Migration Requirements

- Preserve existing maintenance, incident, and project readable values by migrating current `code` values to `key` values.
- Do not renumber existing maintenance, incident, or project records.
- Rename or migrate category `code` values to category `key` values.
- Replace current text category IDs with generated UUID internal IDs.
- Update `incidents`, `maintenances`, and `projects` category foreign keys to reference the new UUID category IDs.
- Keep existing category keys such as `HEA` and `ELV` as the user-facing values.
- Create table-local sequences for maintenance, incident, and project key generation.
- Initialize each sequence at `10`, unless existing preserved keys in that table already include the generated prefix with a suffix of `10` or greater. In that case, initialize the sequence above the highest existing suffix for that table.
- Maintain unique constraints on public keys for maintenance events, incidents, and projects.
- Maintain a unique constraint on category keys.

## Frontend UX

### Maintenance Event, Incident, And Project Forms

- Remove public ID/key inputs entirely from create forms.
- Do not show a generated-key hint.
- On successful create, use the returned `key` to route to the detail page.
- Edit forms can display the public key as non-editable metadata if the page already has an appropriate display location, but must not allow editing it.

### Lists, Details, Search, And Labels

- Display generated public keys where the UI currently displays user-facing IDs.
- Replace user-facing labels such as `ID` and `Project ID` with `Key` and `Project key` where applicable.
- Search continues matching maintenance, incident, and project public keys.
- Search and category UI display category keys, not category internal UUIDs.

### Category Admin

- Remove the category ID input from the admin category form.
- Replace the current category code field with a category key field.
- Display category key anywhere users currently see category code.
- Show the internal category UUID only in the admin category listing, shortened to the first 7 characters, matching the user listing pattern.
- Allow editing a category key even when the category is referenced, because references use internal UUID IDs.

## Backend Acceptance Criteria

- Maintenance create generates `EVT-10`, then `EVT-11`, when no higher existing suffix exists.
- Incident create generates `INC-10`, then `INC-11`, when no higher existing suffix exists.
- Project create generates `PRJ-10`, then `PRJ-11`, when no higher existing suffix exists.
- Create responses return `{ "key": ... }` for maintenance events, incidents, and projects.
- Create requests for maintenance events, incidents, and projects do not require user-provided public keys.
- Update, detail, edit, and delete operations by route key resolve the correct internal UUID record.
- Category create generates or stores an internal UUID ID and stores the normalized user-provided key.
- Category key validation rejects values outside 3 to 5 ASCII alphanumeric characters.
- Category key can be updated while referenced.
- Category foreign keys use internal UUID IDs after migration.
- Sequence migration starts at `10` unless existing preserved keys require a higher next value.

## Frontend Acceptance Criteria

- Maintenance event, incident, and project create forms do not render public key inputs.
- Create save flows route to detail URLs using the returned public key.
- Category admin form does not render an ID input.
- Category admin listing displays the shortened internal UUID and category key.
- Category dropdowns and badges display category key.
- API repository payload mappers use `key` naming instead of `code` for public identifiers.
- Event, incident, and project route params continue representing public keys.

## Open Decisions

No open decisions remain for this PRD.
