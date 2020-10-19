<template>
  <div>
    <h2>{{ $t("finalTitle") }}</h2>
    <ul>
      <transition-group name="component-fade">
        <li key="started" v-show="started">{{$t('started')}}</li>
        <li key="accessKey" v-show="accessKey">{{$t('accessKey')}}</li>
        <li key="symmetricKey" v-show="symmetricKey">{{$t('symmetricKey')}}</li>
        <li key="asymmetricKeys" v-show="asymmetricKeys">{{$t('asymmetricKeys')}}</li>
        <li key="done" v-show="done">{{$t('done')}}</li>
      </transition-group>
    </ul>
  </div>
</template>

<script>
import { mapState } from 'vuex';

export default {
  name: 'CitizenApplicationFinal',
  computed: {
    ...mapState('citizenApplication', [
      'name',
      'identifier',
      'password',
    ]),
    ...mapState('citizenApplication/registerCitizenship', [
      'started',
      'accessKey',
      'symmetricKey',
      'asymmetricKeys',
      'done',
    ]),
  },
  async mounted() {
    if (!this.name) {
      this.$router.push({ name: 'CitizenApplicationNames' });
    } else if (!this.identifier) {
      this.$router.push({ name: 'CitizenApplicationIdentifier' });
    } else if (!this.password) {
      this.$router.push({ name: 'CitizenApplicationPassword' });
    }
    await this.$store.dispatch('citizenApplication/registerCitizenship/register');
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
