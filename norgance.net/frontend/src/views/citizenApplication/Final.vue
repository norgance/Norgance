<template>
  <div>
    <h2>{{ $t("finalTitle") }}</h2>
    <ul>
      <transition-group name="component-fade">
        <li key="started" v-show="started">{{ $t("started") }}</li>
        <li key="accessKey" v-show="accessKey">{{ $t("accessKey") }}</li>
        <li key="symmetricKey" v-show="symmetricKey">
          {{ $t("symmetricKey") }}
        </li>
        <li key="asymmetricKeys" v-show="asymmetricKeys">
          {{ $t("asymmetricKeys") }}
        </li>
        <li key="registering" v-show="registering">{{ $t("registering") }}</li>
        <li class="error" key="error" v-show="error">
          <p>{{ $t("error") }}</p>
          <router-link :to="{ name: 'CitizenApplicationNames' }">
            {{ $t("try-again") }}
          </router-link>
        </li>
        <li class="spinner-area" key="spinner" v-show="!done && !error">
          <Spinner class="large rainbow" />
        </li>
      </transition-group>
    </ul>
  </div>
</template>

<script>
import { mapState } from 'vuex';

import Spinner from '../../components/Spinner.vue';

export default {
  name: 'CitizenApplicationFinal',
  components: {
    Spinner,
  },
  computed: {
    ...mapState('citizenApplication', [
      'name',
      'identifier',
      'identifierHash',
      'password',
    ]),
    ...mapState('citizenApplication/registerCitizenship', [
      'started',
      'accessKey',
      'symmetricKey',
      'asymmetricKeys',
      'registering',
      'done',
      'error',
    ]),
  },
  beforeMount() {
    this.$store.commit('citizenApplication/registerCitizenship/reset');
  },
  watch: {
    done(value) {
      if (value) {
        this.$router.push({ name: 'CitizenApplicationCongratulations' });
      }
    },
  },
  async mounted() {
    if (!this.name) {
      this.$router.push({ name: 'CitizenApplicationNames' });
    } else if (!this.identifier || !this.identifierHash) {
      this.$router.push({ name: 'CitizenApplicationIdentifier' });
    } else if (!this.password) {
      this.$router.push({ name: 'CitizenApplicationPassword' });
    }
    await this.$store.dispatch(
      'citizenApplication/registerCitizenship/register',
    );
  },
};
</script>

<style lang="scss" scoped>
h2 {
  font-size: 1.4em;
  font-weight: 300;
}
ul {
  margin: 0;
  padding: 0;
  font-style: italic;
  //font-family: monospace, monospace;
}
li {
  list-style: none;
}
.spinner-area,
.error,
.done {
  font-style: normal;
  margin-top: 1em;
  list-style: none;
  text-align: center;
}
.error {
  color: #f44336;
}

</style>

<i18n lang="yaml">
en:
  finalTitle: Registering your citizenship
  started: Chargement des données
  accessKey: Création de votre clef d'accés
  symmetricKey: Création de votre clef de coffre fort
  asymmetricKey: Création de vos signatures numériques
  registering: Enregistrement en ligne
fr:
  finalTitle: Enregistrement de votre citoyenneté
  started: Chargement des données
  accessKey: Création de votre clef d'accés
  symmetricKey: Création de votre coffre fort numérique
  asymmetricKeys: Création de vos signatures numériques
  registering: Enregistrement en ligne
  error: Désolé, une erreur est survenue.
  try-again: Ré-essayer
</i18n>
