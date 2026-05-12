import { createApp } from "vue";
import App from "./App.vue";
import { createAppI18n, getStoredLocale } from "./common/i18n";
import router from "./router";

createApp(App).use(createAppI18n(getStoredLocale())).use(router).mount("#app");
