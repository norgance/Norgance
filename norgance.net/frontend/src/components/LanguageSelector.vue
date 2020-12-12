<template>
  <ul>
    <li
      v-for="[name, locale] in locales"
      :key="locale"
      v-show="locale !== currentLocale"
    >
      <button
        @click.prevent="changeLocale(locale)"
      >
      {{ name }}
      </button>
    </li>
  </ul>
</template>

<script>
export default {
  name: 'LanguageSelector',
  data() {
    return {
      locales: [
        ['English', 'en'],
        ['Español', 'es'],
        ['Français', 'fr'],
        ['Norsk', 'no'],
      ],
    };
  },
  computed: {
    currentLocale() {
      return this.$i18n.locale;
    },
  },
  methods: {
    changeLocale(newLocale) {
      this.$i18n.locale = newLocale;
    },
  },
};
</script>

<style lang="scss" scoped>
ul {
  list-style: none;
  text-align: right;
  margin: 0;
}
li {
  display: inline-block;
  //padding: 0.6em;

  button {
    background: none;
    border: none;
    text-decoration: underline;
    cursor: pointer;
  }
}
</style>
