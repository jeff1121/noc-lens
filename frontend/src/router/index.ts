import { createRouter, createWebHashHistory } from "vue-router";

const routes = [
  { path: "/", redirect: "/devices" },
  {
    path: "/devices",
    name: "devices",
    component: () => import("../views/DeviceList.vue"),
  },
  {
    path: "/devices/:id",
    name: "device-detail",
    component: () => import("../views/DeviceDetail.vue"),
  },
  {
    path: "/groups",
    name: "groups",
    component: () => import("../views/Groups.vue"),
  },
  {
    path: "/schedules",
    name: "schedules",
    component: () => import("../views/Schedules.vue"),
  },
  {
    path: "/reports",
    name: "reports",
    component: () => import("../views/Reports.vue"),
  },
  {
    path: "/settings",
    name: "settings",
    component: () => import("../views/Settings.vue"),
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
