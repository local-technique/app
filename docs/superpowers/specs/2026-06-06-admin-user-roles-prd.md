# Admin User Roles PRD

## Purpose

Create an admin-only user management view that lets authorized administrators list users, inspect their roles, and update non-admin roles.

This feature must be protected in both frontend and backend. Frontend visibility is only a usability measure; backend authorization is mandatory for every admin endpoint.

## Terminology

- Product and implementation wording: roles.
- Existing backend storage already uses `roles`.
- `ADMIN` is the privileged admin role already used by backend authorization.
- `CO_OWNER` is a new role granting access to co-owner content: incidents and maintenance.

## Goals

- Add an admin-only frontend view consistent with the existing responsive app layout.
- Hide the admin menu entry until the current user's roles are known.
- Create the `CO_OWNER` role.
- Hide incidents and maintenance menu entries from users without `CO_OWNER`.
- Show a no-access blank state for authenticated users who have no role at all.
- List users in a paginated data table with filtering, search, and sorting.
- Show each user's `id`, `email`, `creation date`, `last login date`, and roles, in that order.
- Allow admins to update a user's composable non-admin roles through a modal.
- Fetch available roles from the backend. The frontend must not hardcode the role list.
- Preserve `ADMIN` when editing roles. The modal submits the full selected non-admin role list; the backend replaces non-admin roles and keeps any existing `ADMIN` role unchanged.
- Ensure all new frontend wording is translated through the existing i18n mechanism.

## Non-Goals

- No user name field. Users do not have names and this feature must not introduce one.
- No account status field. This project does not currently define account states such as active, disabled, locked, pending, suspended, or deleted; therefore account status is out of scope.
- No frontend-only security. All restricted data and mutations must be protected by backend authorization.
- No editing of the `ADMIN` role from this modal.
- No hardcoded frontend role catalog.

## Roles

### Admin Access

- A user with `ADMIN` can access the admin user roles view.
- A user without `ADMIN` must not see the admin menu entry.
- A user without `ADMIN` must not be able to access admin data or mutate roles through direct API calls.

### Co-Owner Access

- `CO_OWNER` grants access to the incident and maintenance areas.
- A user without `CO_OWNER` must not see the incident menu entry.
- A user without `CO_OWNER` must not see the maintenance menu entry.
- Backend endpoints serving incident and maintenance data that should only be available to co-owners must require authenticated `CO_OWNER` authorization, unless an endpoint is intentionally public by existing product design.
- Existing admin-only incident and maintenance mutation endpoints must continue to require `ADMIN`.
- If an admin also needs to see incident and maintenance menus, that admin user must have `CO_OWNER` too. `ADMIN` alone does not imply `CO_OWNER` for menu visibility.

### No-Role Access

- An authenticated user with an empty role list must see a blank access-pending page instead of the incident, maintenance, or admin areas.
- Suggested English wording: `Your account does not have access yet. Please contact the co-ownership board to request access.`
- Suggested French wording: `Votre compte n'a pas encore d'accès. Veuillez contacter le conseil de copropriété pour demander l'accès.`
- This page must use the existing app layout enough to keep language/theme controls available if those controls are normally global.
- This page must not expose incident, maintenance, admin, or user-management data.

### Role Composition

- A user can have zero, one, or several roles.
- Roles are composable.
- `ADMIN` can coexist with `CO_OWNER` and other roles.
- `CO_OWNER` can coexist with future non-admin roles.

### Editing Rules

- The roles modal displays all available roles.
- `ADMIN` is visible in the modal but disabled.
- The table displays `ADMIN` when a user has it.
- The admin can check/uncheck non-admin roles, including `CO_OWNER`.
- On save, the frontend submits the complete selected non-admin role set for that user.
- The backend replaces the user's non-admin roles with the submitted set.
- The backend preserves the user's existing `ADMIN` value unchanged, whether present or absent.
- Because `ADMIN` is preserved by the backend and disabled in the frontend, an admin cannot remove their own admin role through this feature.

## Last Login Definition

- `last_login_at` represents the latest successful login or successful refresh-token use.
- Failed login attempts must not update it.
- Ordinary authenticated API calls must not update it unless they perform a refresh-token flow.
- If a user has never successfully logged in or refreshed after tracking is introduced, `last_login_at` may be `null`.

## Frontend Requirements

### Navigation Visibility

- The app must fetch the current user's roles from the backend.
- The admin menu entry must remain hidden until the role fetch succeeds and confirms `ADMIN`.
- The incident menu entry must remain hidden until the role fetch succeeds and confirms `CO_OWNER`.
- The maintenance menu entry must remain hidden until the role fetch succeeds and confirms `CO_OWNER`.
- There must be no flash of unauthorized UI while roles are loading.
- If the current user is unauthenticated or role fetching fails with an auth error, protected menu entries remain hidden.
- If the current user is authenticated and has no roles, show the access-pending blank page.

### Route Protection

- The admin route must require `ADMIN` in frontend route guards or equivalent route-level logic.
- Incident and maintenance routes must require `CO_OWNER` in frontend route guards or equivalent route-level logic.
- If a non-admin user reaches the admin route manually, display an access-denied state or redirect according to the existing app pattern.
- If a user without `CO_OWNER` reaches incident or maintenance routes manually, display an access-denied state, redirect, or the no-role page according to the existing app pattern and the user's role state.
- This frontend behavior does not replace backend authorization.

### Admin Users Table

The table columns must appear in this order:

1. `id`
2. `email`
3. `creation date`
4. `last login date`
5. `roles`

The table must support:

- Offset-based pagination.
- Fixed page size of 30.
- Search on email.
- Filter by role.
- Sorting by `id`, `email`, `creation date`, or `last login date`.
- Default sorting by `id`.
- Responsive behavior consistent with the rest of the app.

### Role Filter

- Role filter options come from the backend role list endpoint.
- `ADMIN` may be included as a filter option so admins can find admin users.
- `CO_OWNER` must be included as a filter option.
- Filtering by a role returns users having that role among their composed role set.

### Edit Roles Modal

- The modal opens from a user row action.
- The modal title/context identifies the selected user by email and/or id.
- The modal lists available roles from the backend as checkboxes.
- `ADMIN` is visible but disabled.
- Existing roles are preselected.
- Non-admin roles are editable.
- Save submits the complete selected non-admin role list.
- Cancel closes the modal without changes.
- After a successful save, refresh the affected row or current table page so the displayed role list is current.
- Save errors must be shown in the modal or adjacent to the save action using existing app error patterns.

### Empty, Loading, and Error States

- Show a loading state while the users table is loading.
- Show an empty state when no users match search/filter criteria.
- Show an error state if the user list cannot be loaded.
- Show a separate loading/error state for available roles if needed by the filter or modal.

### Internationalization

- All new UI wording must use the existing Vue i18n setup in `front/src/common/i18n.ts` or the current replacement if the app i18n structure evolves before implementation.
- New keys must be provided for both supported locales, `en` and `fr`.
- No new user-facing admin, role, no-access, table, filter, modal, loading, empty, or error text may be hardcoded directly in Vue templates or TypeScript.
- If role labels are returned by the backend, they must either be localized by the backend according to the requested locale or returned as stable label keys that the frontend translates.
- The implementation must remain consistent with the backend translation key/value mechanism where applicable, especially for admin-managed UI copy.

## Backend Requirements

### Authorization

- Every admin users endpoint must require authenticated `ADMIN` authorization on the backend.
- Incident and maintenance read endpoints that expose co-owner content must require authenticated `CO_OWNER` authorization, unless explicitly public by existing product design.
- Existing incident and maintenance mutation endpoints must continue to require authenticated `ADMIN` authorization.
- Authorization must be enforced server-side even if frontend route/menu checks are present.
- Non-authorized callers receive a forbidden response.
- Unauthenticated callers receive an authentication error according to existing API conventions.

### Current User Roles Endpoint

Provide an endpoint for the frontend to list the current authenticated user's roles.

Recommended contract:

```http
GET /me/roles
```

Response:

```json
{
  "roles": ["ADMIN", "CO_OWNER"]
}
```

Notes:

- If an existing `/me` response already returns roles and is appropriate for this use, the implementation may reuse it rather than add a redundant endpoint.
- The frontend must rely on backend data for current-user roles.

### Available Roles Endpoint

Provide an endpoint that returns all roles the frontend may display.

Recommended contract:

```http
GET /admin/roles
```

Response:

```json
{
  "roles": [
    {
      "code": "ADMIN",
      "label_key": "roles.admin"
    },
    {
      "code": "CO_OWNER",
      "label_key": "roles.coOwner"
    }
  ]
}
```

Requirements:

- Protected by backend `ADMIN` authorization unless there is a product need to expose non-admin role metadata elsewhere.
- Frontend must not hardcode role codes for display/editing except for the special `ADMIN` edit-disabled behavior and route/menu checks for `ADMIN` and `CO_OWNER`.
- `ADMIN` must be included so it can be visible but disabled in the modal and usable in table/filter display.
- `CO_OWNER` must be included so it can be assigned by admins and used in filters.
- Role display names must support `en` and `fr` through either backend-localized labels or frontend i18n label keys.

### Admin User List Endpoint

Provide an endpoint to list users with roles and last login date.

Recommended contract:

```http
GET /admin/users?offset=0&limit=30&search_email=foo&role=CO_OWNER&sort=id&direction=asc
```

Response:

```json
{
  "items": [
    {
      "id": "user-id",
      "email": "user@example.com",
      "created_at": "2026-06-06T00:00:00Z",
      "last_login_at": "2026-06-06T00:00:00Z",
      "roles": ["ADMIN", "CO_OWNER"]
    }
  ],
  "total": 1,
  "offset": 0,
  "limit": 30
}
```

Query requirements:

- `offset`: zero-based offset, default `0`.
- `limit`: fixed to `30` for this view. Backend may reject other values or clamp to `30`.
- `search_email`: optional email substring search.
- `role`: optional exact role filter.
- `sort`: one of `id`, `email`, `created_at`, `last_login_at`; default `id`.
- `direction`: `asc` or `desc`; default can follow existing backend conventions, otherwise `asc`.
- Sorting and filtering must be implemented server-side.

### Update User Roles Endpoint

Provide an endpoint to replace a user's non-admin roles while preserving `ADMIN`.

Recommended contract:

```http
PUT /admin/users/{user_id}/roles
```

Request:

```json
{
  "roles": ["CO_OWNER", "ANOTHER_ROLE"]
}
```

Response:

```json
{
  "id": "user-id",
  "roles": ["ADMIN", "CO_OWNER", "ANOTHER_ROLE"]
}
```

Requirements:

- Protected by backend `ADMIN` authorization.
- Request roles represent the complete desired non-admin role set.
- If `ADMIN` is included in the request, the backend must ignore it for mutation purposes or reject the request with a validation error. Prefer rejecting clearly for stricter behavior.
- Backend validates that submitted role codes exist and are assignable.
- Backend preserves the target user's existing `ADMIN` role unchanged.
- Backend returns the resulting complete role set including `ADMIN` if present.

## Data Requirements

### User Fields

- `id`: existing user identifier.
- `email`: existing normalized user email.
- `created_at`: existing user creation timestamp.
- `last_login_at`: timestamp of latest successful login or refresh.
- `roles`: composed role codes.

### Last Login Tracking

- If not already persisted, add a nullable timestamp column or equivalent persisted field for `last_login_at`.
- Update it on successful OAuth login.
- Update it on successful refresh-token use.
- Do not update it on failed auth attempts.

### Role Catalog

- The backend owns the list of available roles.
- The initial catalog must include `ADMIN` and `CO_OWNER`.
- The list may be static in backend code if roles are application-defined, or database-backed if the project needs runtime-configurable roles.
- The frontend must consume the backend-provided list.

## Security Requirements

- Backend admin endpoints must call the existing role guard or equivalent centralized authorization logic.
- Backend co-owner endpoints must call the existing role guard or equivalent centralized authorization logic for `CO_OWNER` where the content is restricted.
- Backend must never trust frontend-disabled controls for protecting `ADMIN`.
- Role update must be validated against the backend role catalog.
- Unknown role codes must not be stored.
- The implementation must avoid allowing a non-admin user to infer the full user list or role assignments.
- API responses should not include fields outside this PRD unless required by existing shared response models.

## UX Requirements

- Reuse existing app layout, typography, spacing, controls, and responsive patterns.
- Dates should use the existing frontend date formatting conventions.
- Roles should be readable in the table, using existing badge/tag patterns if present.
- On narrow screens, the table should remain usable using the existing responsive table/card/scroll pattern used elsewhere in the app.
- The no-role page should be calm and direct, with the message: `Your account does not have access yet. Please contact the co-ownership board to request access.`

## Translation Requirements

At minimum, add translated wording for:

- Admin menu label.
- Admin users page title.
- User table column labels: id, email, creation date, last login date, roles.
- Search by email input label and placeholder.
- Role filter label and empty/default option.
- Pagination controls and result count text if not already shared.
- Edit roles action label.
- Edit roles modal title, save action, cancel action, loading state, and error state.
- Role labels for `ADMIN` and `CO_OWNER`.
- Access denied wording.
- No-role blank page wording.

Suggested translations for the no-role blank page:

- `en`: `Your account does not have access yet. Please contact the co-ownership board to request access.`
- `fr`: `Votre compte n'a pas encore d'accès. Veuillez contacter le conseil de copropriété pour demander l'accès.`

## Acceptance Criteria

- Non-admin users do not see the admin menu item after roles finish loading.
- Admin users see the admin menu item after roles finish loading.
- Users without `CO_OWNER` do not see incident or maintenance menu items after roles finish loading.
- Users with `CO_OWNER` see incident and maintenance menu items after roles finish loading.
- Authenticated users with no roles see the no-role blank page.
- Direct API calls to admin endpoints fail for non-admin users.
- Direct API calls to restricted incident and maintenance endpoints fail for users without `CO_OWNER`, except endpoints intentionally public by existing product design.
- Direct navigation to the admin route by a non-admin user does not expose admin data.
- Direct navigation to incident or maintenance routes by a user without `CO_OWNER` does not expose restricted data.
- Admin users can list users with page size 30.
- Admin users can search users by email.
- Admin users can filter users by role.
- Admin users can sort users by `id`, `email`, `created_at`, and `last_login_at`.
- User rows display `id`, `email`, creation date, last login date, and roles in that order.
- Available roles are fetched from the backend.
- The edit modal displays `ADMIN` when available but disables it.
- The edit modal allows assigning and removing `CO_OWNER`.
- Saving roles replaces the target user's non-admin roles.
- Saving roles preserves the target user's existing `ADMIN` role unchanged.
- Last login updates on successful login and successful refresh-token use.
- Failed login attempts do not update last login.
- All new user-facing wording is available in both `en` and `fr` through the existing i18n mechanism.

## Open Questions

No blocking product gaps remain from the current requirements.

Implementation-time choices still need to follow existing project conventions:

- Whether to add `GET /me/roles` or reuse an existing `/me` response if it already exposes current-user roles cleanly.
- Whether invalid `ADMIN` in an update request should be ignored or rejected. Rejection is stricter and preferred. In both cases, `ADMIN` must not be mutated by this endpoint.
- Whether the backend role catalog should be static application code or persisted in the database. Static backend-owned catalog is sufficient unless runtime role management is required later.
- Whether role labels should be backend-localized from the translation tables or returned as frontend i18n keys. Either is acceptable if no user-facing role label is hardcoded.
