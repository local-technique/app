import { createRouter, createWebHashHistory } from "vue-router";

const EventsPage = () => import("../events/ListingPage.vue");
const EventDetailPage = () => import("../events/DetailPage.vue");
const IncidentsPage = () => import("../incidents/ListingPage.vue");
const IncidentDetailPage = () => import("../incidents/DetailPage.vue");
const NotFoundPage = () => import("../common/NotFoundPage.vue");

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/events" },
    { path: "/events", component: EventsPage },
    { path: "/events/:id", component: EventDetailPage },
    { path: "/incidents", component: IncidentsPage },
    { path: "/incidents/:id", component: IncidentDetailPage },
    { path: "/:pathMatch(.*)*", component: NotFoundPage },
  ],
});

export default router;
