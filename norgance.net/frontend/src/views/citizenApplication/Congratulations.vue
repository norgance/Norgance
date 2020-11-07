<template>
  <div class="done">
    <div>
      <h2>{{ $t("done") }}</h2>
      <router-link :to="{ name: 'Home' }" class="rainbow-button" tag="button">{{
        $t("home")
      }}</router-link>
      <br />
      <br />
      <button ref="confetti-button" @click.prevent="confetti" class="confetti-button">🎉</button>
    </div>
  </div>
</template>

<script>
import { fancyConfettiFromElement } from '../../libs/confetti';

export default {
  name: 'Congratulations',
  computed: {
    done() {
      return this.$store.state.citizenApplication.registerCitizenship.done;
    },
  },
  mounted() {
    if (!this.done) {
      this.$router.push({ name: 'CitizenApplicationStart' });
    } else {
      this.confetti();
    }
  },
  methods: {
    confetti() {
      fancyConfettiFromElement(this.$refs['confetti-button']);
    },
  },
};
</script>

<style lang="scss" scoped>
.done {
  text-align: center;
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  max-width: 30em;
}
h2 {
  font-size: 2.5rem;
  font-weight: 200;
  background-image: linear-gradient(
    to left,
    hsl(320, 75, 50),
    hsl(200, 100, 30)
  );
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
}
.confetti-button {
  height: 2.6em;
  width: 2.6em;
  text-align: center;
  border: none;
}
</style>

<i18n lang="yaml">
fr:
  done: Félicitations ! Vous êtes désormais un citoyen de Norgance !
  home: Tableau de bord
  confetti: Plus de confetti
</i18n>
