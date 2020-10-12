<template>
  <div>
    <h2>{{ $t("finalTitle") }}</h2>
    <ul>
      <transition-group name="component-fade">
        <li key="a" v-show="a">{{$t('canard')}}</li>
        <li key="b" v-show="b">{{$t('canard')}}</li>
      </transition-group>
    </ul>
  </div>
</template>

<script>
import { mapState } from 'vuex';

export default {
  name: 'CitizenApplicationFinal',
  data() {
    return {
      a: false,
      b: false,
    };
  },
  computed: mapState('citizenApplication', [
    'name',
    'identifier',
    'password',
  ]),
  async mounted() {
    if (!this.name) {
      this.$router.push({ name: 'CitizenApplicationNames' });
    } else if (!this.identifier) {
      this.$router.push({ name: 'CitizenApplicationIdentifier' });
    } else if (!this.password) {
      this.$router.push({ name: 'CitizenApplicationPassword' });
    }
    await this.$store.dispatch('citizenApplication/registerCitizenship');
  },
};
</script>

<style lang="scss" scoped>
h2 {
  font-size: 1.4em;
  font-weight: 300;
}
</style>

<i18n lang="yaml">
en:
  finalTitle: Registering your citizenship
fr:
  finalTitle: Enregistrement de votre citoyenneté
</i18n>
