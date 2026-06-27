<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { formatLocalDateTime } from "../common/date";
import { listRoles, listUsers, updateUserRoles, updateUserNames, type AdminUser, type AdminUsersQuery, type RoleDescriptor } from "./api";

const PAGE_SIZE = 30;
const ADMIN_ROLE = "ADMIN";
const NO_ROLE_FILTER = "__NO_ROLE__";

const { t, locale } = useI18n();
const roles = ref<RoleDescriptor[]>([]);
const users = ref<AdminUser[]>([]);
const total = ref(0);
const loadingUsers = ref(false);
const loadingRoles = ref(false);
const usersError = ref(false);
const rolesError = ref(false);
const saveError = ref(false);
const saving = ref(false);
const editingUser = ref<AdminUser | null>(null);
const selectedRoles = ref<string[]>([]);
const editFirstName = ref("");
const editLastName = ref("");
const query = reactive<AdminUsersQuery>({
  offset: 0,
  searchEmail: "",
  role: "",
  sort: "id",
  direction: "asc",
});

let loadVersion = 0;

const pageStart = computed(() => (total.value === 0 ? 0 : query.offset + 1));
const pageEnd = computed(() => Math.min(query.offset + PAGE_SIZE, total.value));
const canGoPrevious = computed(() => query.offset > 0);
const canGoNext = computed(() => query.offset + PAGE_SIZE < total.value);

function roleLabel(roleCode: string): string {
  const role = roles.value.find((value) => value.code === roleCode);
  return role ? t(role.label_key) : roleCode;
}

function formatDate(value: string | null): string {
  return value ? formatLocalDateTime(value, locale.value) : t("labels.never");
}

function displayUserId(id: string): string {
  return id.slice(0, 7);
}

async function loadRoles(): Promise<void> {
  loadingRoles.value = true;
  rolesError.value = false;
  try {
    roles.value = await listRoles();
  } catch {
    rolesError.value = true;
  } finally {
    loadingRoles.value = false;
  }
}

async function loadUserPage(): Promise<void> {
  const requestVersion = ++loadVersion;
  loadingUsers.value = true;
  usersError.value = false;
  try {
    const response = await listUsers(query);
    if (requestVersion === loadVersion) {
      users.value = response.items;
      total.value = response.total;
    }
  } catch {
    if (requestVersion === loadVersion) {
      usersError.value = true;
    }
  } finally {
    if (requestVersion === loadVersion) {
      loadingUsers.value = false;
    }
  }
}

function setSort(sort: AdminUsersQuery["sort"]): void {
  if (query.sort === sort) {
    query.direction = query.direction === "asc" ? "desc" : "asc";
  } else {
    query.sort = sort;
    query.direction = "asc";
  }
}

function openEditor(user: AdminUser): void {
  editingUser.value = user;
  selectedRoles.value = user.roles.filter((role) => role !== ADMIN_ROLE);
  editFirstName.value = user.first_name ?? "";
  editLastName.value = user.last_name ?? "";
  saveError.value = false;
}

function closeEditor(): void {
  if (!saving.value) {
    editingUser.value = null;
  }
}

function toggleRole(roleCode: string, checked: boolean): void {
  if (roleCode === ADMIN_ROLE) {
    return;
  }
  selectedRoles.value = checked
    ? [...new Set([...selectedRoles.value, roleCode])]
    : selectedRoles.value.filter((role) => role !== roleCode);
}

async function saveRoles(): Promise<void> {
  if (!editingUser.value) {
    return;
  }

  const user = editingUser.value;
  saving.value = true;
  saveError.value = false;
  try {
    const firstName = editFirstName.value.trim() || null;
    const lastName = editLastName.value.trim() || null;
    await Promise.all([updateUserRoles(user.id, selectedRoles.value), updateUserNames(user.id, firstName, lastName)]);
    editingUser.value = null;
    await loadUserPage();
  } catch {
    saveError.value = true;
  } finally {
    saving.value = false;
  }
}

function nextPage(): void {
  if (canGoNext.value) {
    query.offset += PAGE_SIZE;
  }
}

function previousPage(): void {
  if (canGoPrevious.value) {
    query.offset = Math.max(0, query.offset - PAGE_SIZE);
  }
}

watch(
  () => [query.searchEmail, query.role, query.sort, query.direction],
  () => {
    query.offset = 0;
    void loadUserPage();
  },
);

watch(
  () => query.offset,
  () => {
    void loadUserPage();
  },
);

void loadRoles();
void loadUserPage();
</script>

<template>
  <main class="page-wrap admin-page">
    <h1 class="page-title">{{ t("labels.adminUsersTitle") }}</h1>

    <section class="admin-filters" :aria-label="t('labels.userFilters')">
      <label>
        <span>{{ t("labels.searchUsersByEmail") }}</span>
        <input v-model="query.searchEmail" type="search" :placeholder="t('labels.searchUsersByEmailPlaceholder')" />
      </label>

      <label>
        <span>{{ t("labels.filterByRole") }}</span>
        <select v-model="query.role" :disabled="loadingRoles || rolesError">
          <option value="">{{ t("labels.allRoles") }}</option>
          <option :value="NO_ROLE_FILTER">{{ t("labels.noRole") }}</option>
          <option v-for="role in roles" :key="role.code" :value="role.code">{{ roleLabel(role.code) }}</option>
        </select>
      </label>
    </section>

    <p v-if="rolesError" class="empty-state">{{ t("labels.rolesLoadFailed") }}</p>
    <p v-if="loadingUsers" class="empty-state">{{ t("labels.loadingUsers") }}</p>
    <p v-else-if="usersError" class="empty-state">{{ t("labels.usersLoadFailed") }}</p>
    <p v-else-if="users.length === 0" class="empty-state">{{ t("labels.noUsersMatch") }}</p>

    <div v-else class="admin-table-wrap">
      <table class="admin-table">
        <thead>
          <tr>
            <th><button type="button" @click="setSort('id')">{{ t("labels.userId") }}</button></th>
            <th><button type="button" @click="setSort('email')">{{ t("labels.email") }}</button></th>
            <th>{{ t("labels.firstName") }}</th>
            <th>{{ t("labels.lastName") }}</th>
            <th><button type="button" @click="setSort('created_at')">{{ t("labels.createdAt") }}</button></th>
            <th><button type="button" @click="setSort('last_login_at')">{{ t("labels.lastLoginAt") }}</button></th>
            <th>{{ t("labels.roles") }}</th>
            <th>{{ t("labels.actions") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in users" :key="user.id">
            <td :title="user.id">{{ displayUserId(user.id) }}</td>
            <td>{{ user.email }}</td>
            <td>{{ user.first_name ?? "" }}</td>
            <td>{{ user.last_name ?? "" }}</td>
            <td>{{ formatDate(user.created_at) }}</td>
            <td>{{ formatDate(user.last_login_at) }}</td>
            <td>
              <span v-for="role in user.roles" :key="role" class="role-badge">{{ roleLabel(role) }}</span>
            </td>
            <td>
              <button class="secondary-button" type="button" @click="openEditor(user)">
                {{ t("labels.edit") }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <nav class="pagination" :aria-label="t('labels.pagination')" v-if="!usersError && total > 0">
      <button type="button" :disabled="!canGoPrevious" @click="previousPage">{{ t("labels.previousPage") }}</button>
      <span>{{ t("labels.userResults", { start: pageStart, end: pageEnd, total }) }}</span>
      <button type="button" :disabled="!canGoNext" @click="nextPage">{{ t("labels.nextPage") }}</button>
    </nav>

    <div v-if="editingUser" class="modal-backdrop" role="presentation">
      <section class="user-modal" role="dialog" aria-modal="true" :aria-label="t('labels.editRoles')">
        <h2>{{ t("labels.editUserTitle", { email: editingUser.email }) }}</h2>
        <div class="name-fields">
          <label>
            <span>{{ t("labels.firstName") }}</span>
            <input v-model="editFirstName" type="text" :disabled="saving" maxlength="100" />
          </label>
          <label>
            <span>{{ t("labels.lastName") }}</span>
            <input v-model="editLastName" type="text" :disabled="saving" maxlength="100" />
          </label>
        </div>
        <hr />
        <div class="role-options">
          <label v-for="role in roles" :key="role.code">
            <input
              type="checkbox"
              :checked="role.code === ADMIN_ROLE ? editingUser.roles.includes(ADMIN_ROLE) : selectedRoles.includes(role.code)"
              :disabled="role.code === ADMIN_ROLE || saving"
              @change="toggleRole(role.code, ($event.target as HTMLInputElement).checked)"
            />
            <span>{{ roleLabel(role.code) }}</span>
          </label>
        </div>
        <p v-if="saveError" class="modal-error">{{ t("labels.saveUserFailed") }}</p>
        <footer>
          <button class="secondary-button" type="button" :disabled="saving" @click="closeEditor">
            {{ t("labels.cancel") }}
          </button>
          <button class="primary-button" type="button" :disabled="saving" @click="saveRoles">
            {{ saving ? t("labels.saving") : t("labels.save") }}
          </button>
        </footer>
      </section>
    </div>
  </main>
</template>

<style scoped>
.admin-page {
  width: 100%;
  max-width: none;
}

.admin-filters {
  display: flex;
  flex-wrap: wrap;
  gap: 0.8rem;
  margin: 1rem 0;
}

.admin-filters label {
  display: grid;
  gap: 0.35rem;
  min-width: min(280px, 100%);
  color: var(--muted-fg);
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.07em;
}

.admin-filters input,
.admin-filters select {
  border: 1px solid var(--control-border);
  border-radius: 0.72rem;
  min-height: 2.35rem;
  padding: 0 0.7rem;
  background: var(--control-bg);
  color: var(--control-fg);
}

.admin-table-wrap {
  overflow-x: auto;
  border: 1px solid var(--border-color);
  border-radius: 0.9rem;
  background: var(--panel-bg);
}

.admin-table {
  width: 100%;
  border-collapse: collapse;
  min-width: 880px;
}

.admin-table th,
.admin-table td {
  padding: 0.72rem;
  border-bottom: 1px solid var(--border-color);
  text-align: left;
  vertical-align: top;
}

.admin-table th button {
  border: 0;
  padding: 0;
  background: transparent;
  color: inherit;
  cursor: pointer;
  font: inherit;
  font-weight: 700;
}

.role-badge {
  display: inline-flex;
  margin: 0 0.35rem 0.35rem 0;
  border-radius: 999px;
  padding: 0.18rem 0.5rem;
  background: rgba(72, 144, 255, 0.18);
  color: var(--page-fg);
  font-size: 0.82rem;
  font-weight: 700;
}

.secondary-button,
.primary-button,
.pagination button {
  border: 1px solid var(--control-border);
  border-radius: 0.55rem;
  padding: 0.45rem 0.7rem;
  background: var(--control-bg);
  color: var(--control-fg);
  cursor: pointer;
}

.primary-button {
  border-color: rgba(72, 144, 255, 0.7);
  background: rgba(72, 144, 255, 0.22);
}

button:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.pagination {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.7rem;
  margin-top: 0.9rem;
  color: var(--muted-fg);
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 120;
  display: grid;
  place-items: center;
  padding: 1rem;
  background: var(--overlay-bg);
}

.user-modal {
  width: min(480px, 100%);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  padding: 1rem;
  background: var(--panel-bg);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.28);
}

.user-modal h2 {
  margin: 0 0 0.9rem;
  font-size: 1.2rem;
}

.user-modal hr {
  border: none;
  border-top: 1px solid var(--border-color);
  margin: 0.9rem 0;
}

.name-fields {
  display: grid;
  gap: 0.55rem;
  margin-bottom: 0.5rem;
}

.name-fields label {
  display: grid;
  gap: 0.25rem;
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--muted-fg);
}

.name-fields input {
  border: 1px solid var(--control-border);
  border-radius: 0.55rem;
  padding: 0.45rem 0.7rem;
  background: var(--control-bg);
  color: var(--control-fg);
  font-size: 1rem;
  text-transform: none;
  letter-spacing: normal;
}

.role-options {
  display: grid;
  gap: 0.55rem;
}

.role-options label {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.modal-error {
  color: #f35a67;
  font-weight: 700;
}

.user-modal footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.6rem;
  margin-top: 1rem;
}
</style>
