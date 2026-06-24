import { createApp } from "vue";
import { createPinia } from "pinia";
import VueVirtualScroller from "vue-virtual-scroller";
import "vue-virtual-scroller/dist/vue-virtual-scroller.css";

import App from "./App.vue";
import { router } from "./router";
import "./styles/main.css";

createApp(App)
  .use(createPinia())
  .use(router)
  .use(VueVirtualScroller)
  .mount("#app");
