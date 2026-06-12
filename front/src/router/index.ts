import { createRouter, createWebHashHistory } from "vue-router";
import { sanitizeRedirectPath } from "../auth/redirect";
import { ensureAuthenticated, ensureCurrentUserRoles, hasAnyRole, hasRole, hasNoRoles } from "../auth/session";

const LoginPage = () => import("../auth/LoginPage.vue");
const OAuthCallbackPage = () => import("../auth/OAuthCallbackPage.vue");
const EventsPage = () => import("../events/ListingPage.vue");
const EventDetailPage = () => import("../events/DetailPage.vue");
const EventFormPage = () => import("../events/FormPage.vue");
const IncidentsPage = () => import("../incidents/ListingPage.vue");
const IncidentDetailPage = () => import("../incidents/DetailPage.vue");
const IncidentFormPage = () => import("../incidents/FormPage.vue");
const ProjectsPage = () => import("../projects/ListingPage.vue");
const ProjectDetailPage = () => import("../projects/DetailPage.vue");
const ProjectFormPage = () => import("../projects/FormPage.vue");
const SettingsPage = () => import("../views/settings/SettingsPage.vue");
const AdminUsersPage = () => import("../admin/UsersPage.vue");
const AdminCategoriesPage = () => import("../admin/CategoriesPage.vue");
const AccessPendingPage = () => import("../common/AccessPendingPage.vue");
const AccessDeniedPage = () => import("../common/AccessDeniedPage.vue");
const NotFoundPage = () => import("../common/NotFoundPage.vue");

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/events" },
    { path: "/login", component: LoginPage },
    { path: "/auth/callback", component: OAuthCallbackPage },
    { path: "/access-pending", component: AccessPendingPage, meta: { requiresAuth: true } },
    { path: "/access-denied", component: AccessDeniedPage, meta: { requiresAuth: true } },
    { path: "/events", component: EventsPage, meta: { requiresAuth: true, requiredRoles: ["ADMIN", "CO_OWNER", "CO_OWNERSHIP_BOARD"] } },
    { path: "/events/new", component: EventFormPage, meta: { requiresAuth: true, requiredRoles: ["ADMIN", "CO_OWNERSHIP_BOARD"] } },
    { path: "/events/:id", component: EventDetailPage, meta: { requiresAuth: true, requiredRoles: ["ADMIN", "CO_OWNER", "CO_OWNERSHIP_BOARD"] } },
    { path: "/events/:id/edit", component: EventFormPage, meta: { requiresAuth: true, requiredRoles: ["ADMIN", "CO_OWNERSHIP_BOARD"] } },
    { path: "/incidents", component: IncidentsPage, meta: { requiresAuth: true, requiredRoles: ["ADMIN", "CO_OWNER", "CO_OWNERSHIP_BOARD"] } },
    { path: "/incidents/new", component: IncidentFormPage, meta: { requiresAuth: true, requiredRoles: ["ADMIN", "CO_OWNERSHIP_BOARD"] } },
    { path: "/incidents/:id", component: IncidentDetailPage, meta: { requiresAuth: true, requiredRoles: ["ADMIN", "CO_OWNER", "CO_OWNERSHIP_BOARD"] } },
    { path: "/incidents/:id/edit", component: IncidentFormPage, meta: { requiresAuth: true, requiredRoles: ["ADMIN", "CO_OWNERSHIP_BOARD"] } },
    { path: "/projects", component: ProjectsPage, meta: { requiresAuth: true, requiredRoles: ["ADMIN", "CO_OWNER", "CO_OWNERSHIP_BOARD"] } },
    { path: "/projects/new", component: ProjectFormPage, meta: { requiresAuth: true, requiredRoles: ["ADMIN", "CO_OWNERSHIP_BOARD"] } },
    { path: "/projects/:id", component: ProjectDetailPage, meta: { requiresAuth: true, requiredRoles: ["ADMIN", "CO_OWNER", "CO_OWNERSHIP_BOARD"] } },
    { path: "/projects/:id/edit", component: ProjectFormPage, meta: { requiresAuth: true, requiredRoles: ["ADMIN", "CO_OWNERSHIP_BOARD"] } },
    { path: "/admin/users", component: AdminUsersPage, meta: { requiresAuth: true, requiredRole: "ADMIN" } },
    { path: "/admin/categories", component: AdminCategoriesPage, meta: { requiresAuth: true, requiredRole: "ADMIN" } },
    { path: "/settings", component: SettingsPage, meta: { requiresAuth: true } },
    { path: "/:pathMatch(.*)*", component: NotFoundPage },
  ],
});

router.beforeEach(async (to) => {
  const requiresAuth = to.matched.some((entry) => entry.meta.requiresAuth === true);
  if (!requiresAuth) {
    return true;
  }

  const ok = await ensureAuthenticated();
  if (!ok) {
    return {
      path: "/login",
      query: {
        redirect: sanitizeRedirectPath(to.fullPath),
      },
    };
  }

  const rolesOk = await ensureCurrentUserRoles();
  if (!rolesOk) {
    return {
      path: "/login",
      query: {
        redirect: sanitizeRedirectPath(to.fullPath),
      },
    };
  }

  const requiredRole = to.matched.find((entry) => typeof entry.meta.requiredRole === "string")?.meta.requiredRole;
  if (typeof requiredRole !== "string" || hasRole(requiredRole)) {
    const requiredRoles = to.matched.find((entry) => Array.isArray(entry.meta.requiredRoles))?.meta.requiredRoles;
    if (!Array.isArray(requiredRoles) || hasAnyRole(requiredRoles.filter((role): role is string => typeof role === "string"))) {
      return true;
    }
  }

  if (hasNoRoles() && to.path !== "/access-pending") {
    return { path: "/access-pending" };
  }

  return { path: "/access-denied" };
});

export default router;
