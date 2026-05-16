import { createRouter, createWebHashHistory } from "vue-router";
import { sanitizeRedirectPath } from "../auth/redirect";
import { ensureAuthenticated } from "../auth/session";

const LoginPage = () => import("../auth/LoginPage.vue");
const OAuthCallbackPage = () => import("../auth/OAuthCallbackPage.vue");
const EventsPage = () => import("../events/ListingPage.vue");
const EventDetailPage = () => import("../events/DetailPage.vue");
const IncidentsPage = () => import("../incidents/ListingPage.vue");
const IncidentDetailPage = () => import("../incidents/DetailPage.vue");
const NotFoundPage = () => import("../common/NotFoundPage.vue");

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/events" },
    { path: "/login", component: LoginPage },
    { path: "/auth/callback", component: OAuthCallbackPage },
    { path: "/events", component: EventsPage, meta: { requiresAuth: true } },
    { path: "/events/:id", component: EventDetailPage, meta: { requiresAuth: true } },
    { path: "/incidents", component: IncidentsPage, meta: { requiresAuth: true } },
    { path: "/incidents/:id", component: IncidentDetailPage, meta: { requiresAuth: true } },
    { path: "/:pathMatch(.*)*", component: NotFoundPage },
  ],
});

router.beforeEach(async (to) => {
  const requiresAuth = to.matched.some((entry) => entry.meta.requiresAuth === true);
  if (!requiresAuth) {
    return true;
  }

  const ok = await ensureAuthenticated();
  if (ok) {
    return true;
  }

  return {
    path: "/login",
    query: {
      redirect: sanitizeRedirectPath(to.fullPath),
    },
  };
});

export default router;
