<template>
  <header>
    <rust-check class="rust-check"/>
    <section class="norgance-brand">
      <h1 class="title">
        <a href="/"
          ><img alt="Norgance" src="../assets/norgance.svg" height="40"
        /></a>
        <img
          alt=""
          :aria-label="$t('norgance-flag-aria')"
          src="../assets/flag-norgance.svg"
          class="flag"
          height="40"
        />
      </h1>
    </section>
    <router-link tag="button" :to="{ name: 'Home' }" exact>{{
      $t("dashboard")
    }}</router-link>
    <router-link
      tag="button"
      :to="{
        name: 'CitizenApplicationStart',
      }"
    >
      {{ $t("citizen-application") }}</router-link
    >
    <router-link tag="button" to="/about1" class="w98">About</router-link>
    <router-link tag="button" to="/about2" class="w98">About</router-link>
    <router-link tag="button" to="/about2" class="w98">About</router-link>
    <router-link tag="button" to="/about3" class="w98">About</router-link>
    <router-link tag="button" to="/about4" class="w98">About</router-link>
    <router-link tag="button" to="/about5" class="w98">About</router-link>
    <router-link tag="button" to="/about6" class="w98">About</router-link>
    <router-link tag="button" to="/about8" class="w98">About</router-link>
    <router-link tag="button" to="/about7" class="w98">About</router-link>
    <router-link tag="button" to="/about9" class="w98">About</router-link>
    <router-link tag="button" to="/about10" class="w98">About</router-link>
    <router-link tag="button" to="/about11" class="w98">About</router-link>
    <a
      href="https://github.com/norgance/Norgance/issues/new/choose"
      target="_blank"
      >{{ $t("report-issue") }}</a
    >

    <hr class="rainbow" />
    <language-selector />
  </header>
</template>

<script>
import RustCheck from './RustCheck.vue';
import LanguageSelector from './LanguageSelector.vue';

export default {
  name: 'Header',
  components: { RustCheck, LanguageSelector },
};
</script>

<style lang="scss" scoped>
header {
  background: white;
}
hr.rainbow {
  height: 10px;
  border: none;
  margin: 0;
  background: linear-gradient(
    124deg,
    #ff2400,
    #e81d1d,
    #90710f,
    #e3e81d,
    #1de840,
    #1ddde8,
    #2b1de8,
    purple,
    pink,
    #ff2400,
    #e81d1d,
    #e8b71d,
    #e3e81d,
    #1de840,
    #1ddde8,
    #2b1de8,
    lightcoral,
    pink
  );
  background-size: 1000% 100%;
  animation: rainbow 120s linear infinite;
}
.rust-check {
  float: right;
}
</style>
<i18n lang="yaml">
en:
  dashboard: Dashboard
  citizen-application: Become a citizen
  report-issue: Report an issue
fr:
  dashboard: Tableau de bord
  report-issue: Signaler un problème
  citizen-application: Demande de citoyenneté
</i18n>
