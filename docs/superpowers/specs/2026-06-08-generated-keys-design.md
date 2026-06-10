# Generated Public Keys Implementation Design

## Goal

Implement `docs/superpowers/specs/2026-06-08-generated-keys-prd.md` as a coordinated breaking change. The application will use `id` only for internal UUID identifiers and `key` only for user-facing readable identifiers in URLs, labels, search, and API payloads.

## Approach

Use one database migration plus matching backend and frontend changes. Do not add compatibility aliases for old `code` fields or create payload public IDs. This keeps the API shape explicit and matches the PRD scope.

## Database Design

Add migration `0013_generated_public_keys.sql`.

The migration will rename public identifier columns from `code` to `key` on `maintenances`, `incidents`, `projects`, and `event_categories`. It will preserve existing public values and keep unique constraints on the public key columns.

`event_categories.id` will become an internal UUID primary key. Existing category text IDs such as `HEA` and `ELV` will become `event_categories.key`. The migration will update `incidents.category_code`, `maintenances.category_code`, and `projects.category_code` to category UUID values, then rename those foreign key columns to `category_id`.

The migration will create table-local PostgreSQL sequences named `maintenance_key_seq`, `incident_key_seq`, and `project_key_seq`. Each sequence will start at `10` unless existing keys with the relevant prefix already contain a numeric suffix of `10` or greater; in that case the sequence starts above the highest preserved suffix.

## Backend Design

Backend models will expose public identifiers as `key`. Internal UUID primary keys remain `id` only where they are intentionally returned, such as categories.

Maintenance, incident, and project create request models will not include a public key. Repositories will generate the key atomically in the `INSERT` using `nextval(...)`, and create handlers will return `{ "key": "..." }`. Update, detail, edit, translation, and delete handlers will continue to use route params as public keys and resolve them against the database key columns.

Category create requests will include a user-entered `key` but not an `id`; the backend will generate a UUID. Category update requests will identify the category by UUID route param and allow changing the key after validation. Category references in maintenance, incident, and project rows will use category UUIDs.

Category key validation will trim whitespace, uppercase values, allow only ASCII alphanumeric characters, and require 3 to 5 characters.

## Frontend Design

Frontend API types and mappers will use `key` for public identifiers. Maintenance events, incidents, and projects will keep route params as public keys, but create forms will not render public key inputs. After a successful create, repositories will return the generated key and form pages will route to the detail page using it.

Category frontend types will rename `code` to `key`. Category admin will remove the ID input, show a key input, display the shortened UUID in the ID column, and continue showing category keys in dropdowns and badges.

Labels that currently say `ID`, `Event ID`, or `Project ID` for public identifiers will become `Key`, `Event key`, or `Project key` where applicable.

## Testing And Verification

Backend tests will cover category key normalization and validation, plus create request behavior where existing seams make that practical. Frontend tests will cover removal of public key inputs, routing after create using returned keys, category admin form/list behavior, and API mapper naming.

After code changes, run the required checks from `AGENTS.md`: frontend `npm run lint`, `npm run test`, `npm run build`; backend `cargo clippy --all-features -- -D warnings`, `cargo test --all-features`, and `cargo build --all-features`.

## Non-Goals

Do not renumber existing public keys. Do not add old `code` API aliases. Do not change route shapes beyond continuing to use public keys in maintenance, incident, and project routes.
