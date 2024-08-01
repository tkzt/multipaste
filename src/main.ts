import { createApp } from "vue";
import App from "./App.vue";
import 'virtual:uno.css'
import "./styles.css";
import router from "./router";
import register from "./shortcut";

createApp(App)
  .use(register)
  .use(router)
  .mount("#app");
