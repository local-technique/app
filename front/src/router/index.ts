import { createRouter, createWebHashHistory } from "vue-router";
import { sanitizeRedirectPath } from "../auth/redirect";
import { ensureAuthenticated, ensureCurrentUserRoles, hasRole, hasNoRoles } from "../auth/session";

const LoginPage = () => import("../auth/LoginPage.vue");
const OAuthCallbackPage = () => import("../auth/OAuthCallbackPage.vue");
const EventsPage = () => import("../events/ListingPage.vue");
const EventDetailPage = () => import("../events/DetailPage.vue");
const IncidentsPage = () => import("../incidents/ListingPage.vue");
const IncidentDetailPage = () => import("../incidents/DetailPage.vue");
const AdminUsersPage = () => import("../admin/UsersPage.vue");
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
    { path: "/events", component: EventsPage, meta: { requiresAuth: true, requiredRole: "CO_OWNER" } },
    { path: "/events/:id", component: EventDetailPage, meta: { requiresAuth: true, requiredRole: "CO_OWNER" } },
    { path: "/incidents", component: IncidentsPage, meta: { requiresAuth: true, requiredRole: "CO_OWNER" } },
    { path: "/incidents/:id", component: IncidentDetailPage, meta: { requiresAuth: true, requiredRole: "CO_OWNER" } },
    { path: "/admin/users", component: AdminUsersPage, meta: { requiresAuth: true, requiredRole: "ADMIN" } },
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
    return true;
  }

  if (hasNoRoles() && to.path !== "/access-pending") {
    return { path: "/access-pending" };
  }

  return { path: "/access-denied" };
});

export default router;
