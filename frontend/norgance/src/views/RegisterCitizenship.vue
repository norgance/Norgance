<template>
  <div>
    <h1>{{ $t("hello") }}</h1>
    <p>{{ $t("congratulations") }}</p>
    <hr />
    <p v-if="step" class="step-counter">{{ $t("step", { step, total: totalStep }) }}</p>
    <router-view>
      canard
    </router-view>
  </div>
</template>

<script>

export default {
  name: 'RegisterCitizenship',
  computed: {
    step() {
      switch (this.$route.name) {
        case 'CitizenApplicationNames':
          return '1';
        case 'CitizenApplicationBirthday':
          return '2';
        case 'CitizenApplicationIdentifier':
          return '3';
        case 'CitizenApplicationPassword':
          return '4';
        case 'CitizenApplicationSummary':
          return '5';
        default:
          return undefined;
      }
    },
    totalStep() {
      return '5';
    },
  },
};
</script>

<style lang="scss" scoped>
.step-counter {
  text-align: right;
  font-size: 0.9em;
}
</style>

<i18n lang="yaml">
en:
  hello: "hello world!"
  congratulations: Congratulations for applying as a Norgance citizen !
  become-citizen: Become a citizen
  step: Step {step}/{total}.
fr:
  hello: Bonjour
  congratulations: Félicitations pour votre application en tant que citoyen de Norgance !
  step: Étape {step} sur {total}.
</i18n>
