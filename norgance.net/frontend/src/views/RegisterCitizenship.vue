<template>
  <div>
    <h1 v-if="step !== '6'">{{ $t("title") }}</h1>
    <p v-if="step && step !== '6'" class="step-counter">
      {{ $t("step", { step, total: totalStep }) }}
    </p>
    <transition name="component-fade" mode="out-in"
      >>
      <router-view class="registration-view"></router-view>
    </transition>
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
        case 'CitizenApplicationCongratulations':
          return '6';
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
  margin-top:0;
  color: #666;
}
.registration-view {
  max-width: 20em;
  margin: 2em auto;
}
h1 {
  margin-top: 2rem;
  text-align: center;
  font-weight: 300;
  background-image: linear-gradient(
    to left,
    hsl(320, 75, 50),
    hsl(200, 100, 30),
    hsl(320, 75, 50),
  );
  background-size: 20em 100%;
  -webkit-background-clip: text;
  -moz-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
  -moz-text-fill-color: transparent;
}
</style>

<i18n lang="yaml">
en:
  title: Citizenship Application
  step: Step {step}/{total}.
fr:
  title: Demande de citoyenneté
  step: Étape {step} sur {total}.
</i18n>
