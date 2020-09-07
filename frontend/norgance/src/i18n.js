import Vue from 'vue';
import VueI18n from 'vue-i18n';
import { en, es, fr } from '@braid/vue-formulate-i18n';

Vue.use(VueI18n);

const DEFAULT_LOCALE = 'en';
const LOCALES = ['en', 'es', 'fr'];

function loadLocaleMessages() {
  return Object.fromEntries(LOCALES.map((locale) => ([locale, {}])));
}

function findBestLocale() {
  return (window.navigator.languages || [window.navigator.language])
    .map((locale) => locale.match(/^[^-]+/)[0].toLocaleLowerCase())
    .find((locale) => LOCALES.includes(locale))
    || DEFAULT_LOCALE;
}

const formulateI18n = [en, es, fr];
export {
  formulateI18n,
};

export default new VueI18n({
  locale: findBestLocale(),
  fallbackLocale: DEFAULT_LOCALE,
  messages: loadLocaleMessages(),
});
