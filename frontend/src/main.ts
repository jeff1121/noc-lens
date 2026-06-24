import { createApp } from "vue";
import { createPinia } from "pinia";
import VueVirtualScroller from "vue-virtual-scroller";
import "vue-virtual-scroller/dist/vue-virtual-scroller.css";
import VueApexCharts from "vue3-apexcharts";

import App from "./App.vue";
import { router } from "./router";
import "./styles/main.css";

createApp(App)
  .use(createPinia())
  .use(router)
  .use(VueVirtualScroller)
  .component("apexchart", VueApexCharts)
  .mount("#app");
