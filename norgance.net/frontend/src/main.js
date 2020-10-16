import Vue from 'vue';
import VueFormulate from '@braid/vue-formulate';

import App from './App.vue';
import router from './router';
import store from './store';
import i18n, { formulateI18n } from './i18n';
import entropy from './entropy';

Vue.config.productionTip = true; // false;

Vue.use(VueFormulate, {
  plugins: [...formulateI18n],
  /* classes: {
    input: 'w98',
  }, */
});

new Vue({
  router,
  store,
  i18n,
  render: (h) => h(App),
}).$mount('#app');

window.entropy = entropy();
window.entropy.start();
