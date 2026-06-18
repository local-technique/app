# Co-ownership Board Ops Role Design

## Purpose

Add a new composable role `CO_OWNERSHIP_BOARD_OPS` that extends the capabilities of `CO_OWNERSHIP_BOARD` with delete permissions on incidents, maintenances, and projects, plus full CRUD access to categories.

## Role Definition

- **Code**: `CO_OWNERSHIP_BOARD_OPS`
- **English label**: Co-ownership board ops
- **French label**: Conseil syndical ops
- **Assignable**: Yes (in admin user management UI, same as `CO_OWNER` and `CO_OWNERSHIP_BOARD`)
- **Relationship**: Independent, composable role. A user can have `CO_OWNERSHIP_BOARD`, `CO_OWNERSHIP_BOARD_OPS`, both, or neither.

## Permissions

| Capability | `CO_OWNERSHIP_BOARD` | `CO_OWNERSHIP_BOARD_OPS` |
|---|---|---|
| View incidents/maintenances/projects | ✅ | ✅ |
| Create incidents/maintenances/projects | ✅ | ✅ |
| Edit incidents/maintenances/projects | ✅ | ✅ |
| Delete incidents/maintenances/projects | ❌ | ✅ |
| List categories (for dropdowns) | ✅ | ✅ |
| Create/edit/delete categories | ❌ | ✅ |

Admins retain all existing permissions and are unaffected.

## Changes

### Backend

- `common/role.rs`: Add `CoOwnershipBoardOps` variant, add to `ALL` and `ASSIGNABLE` arrays.
- `incidents/http.rs`: Delete handler changes `ensure_role(Admin)` → `ensure_any_role([Admin, CoOwnershipBoardOps])`. Create/update handlers also include the new role.
- `maintenances/http.rs`: Same pattern as incidents.
- `projects/http.rs`: Same pattern as incidents.
- `categories/http.rs`: Public list includes new role. Admin CRUD endpoints change from `ensure_role(Admin)` → `ensure_any_role([Admin, CoOwnershipBoardOps])`.

### Frontend

- `common/i18n.ts`: Add translation keys for role label.
- `incidents/DetailPage.vue`, `events/DetailPage.vue`, `projects/DetailPage.vue`: `canDelete` includes `CO_OWNERSHIP_BOARD_OPS`.
- `router/index.ts`: Add `CO_OWNERSHIP_BOARD_OPS` to route guard arrays where `CO_OWNERSHIP_BOARD` is present.
- `App.vue` / nav components: Include `CO_OWNERSHIP_BOARD_OPS` in link visibility checks.
