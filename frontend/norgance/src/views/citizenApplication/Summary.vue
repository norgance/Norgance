<template>
  <div>
    <FormulateForm @submit="nextStep">
      <dl>
        <h2>{{ $t("summaryTitle") }}</h2>
        <dt>{{ $t("name") }}</dt>
        <dd>
          {{ name }}
          <router-link :to="{ name: 'CitizenApplicationNames' }">
            {{ $t("edit") }}
          </router-link>
        </dd>
        <div v-show="familyName">
          <dt>{{ $t("familyName") }}</dt>
          <dd>
            {{ familyName }}
            <router-link :to="{ name: 'CitizenApplicationNames' }">
              {{ $t("edit") }}
            </router-link>
          </dd>
        </div>
        <div v-show="birthday">
          <dt>{{ $t("birthday") }}</dt>
          <dd>
            {{ birthday | formatBirthday }}
            <router-link :to="{ name: 'CitizenApplicationBirthday' }">
              {{ $t("edit") }}
            </router-link>
          </dd>
        </div>
        <div v-show="birthplace">
          <dt>{{ $t("birthplace") }}</dt>
          <dd>
            {{ birthplace }}
            <router-link :to="{ name: 'CitizenApplicationBirthday' }">
              {{ $t("edit") }}
            </router-link>
          </dd>
        </div>
        <dt>{{ $t("identifier") }}</dt>
        <dd>
          {{ identifier }}
          <router-link :to="{ name: 'CitizenApplicationIdentifier' }">
            {{ $t("edit") }}
          </router-link>
        </dd>
        <dt>{{ $t("password") }}</dt>
        <dd>
          <code>***************</code>
          <router-link :to="{ name: 'CitizenApplicationPassword' }">
            {{ $t("edit") }}
          </router-link>
        </dd>
      </dl>
      <FormulateInput type="submit" :name="$t('continue')" />
    </FormulateForm>
  </div>
</template>
<script>
import { mapState } from 'vuex';
import { formatLocaleDate } from '../../i18n';

export default {
  name: 'CitizenApplicationSummary',
  computed: mapState('citizenApplication', [
    'name',
    'familyName',
    'birthday',
    'birthplace',
    'identifier',
    'password',
  ]),
  methods: {
    nextStep() {
      this.$router.push({ name: 'CitizenApplicationPassword' });
    },
  },
  mounted() {
    if (!this.name) {
      this.$router.push({ name: 'CitizenApplicationNames' });
    } else if (!this.identifier) {
      this.$router.push({ name: 'CitizenApplicationIdentifier' });
    } else if (!this.password) {
      this.$router.push({ name: 'CitizenApplicationPassword' });
    }
  },
  filters: {
    formatBirthday(date) {
      return formatLocaleDate(date);
    },
  },
};
</script>

<style lang="scss" scoped>
h2 {
  font-size: 1.4em;
  font-weight: 300;
}
dl {
  background: white;
  border: 1px solid hsl(0, 0, 75);
  border-radius: 3px;
  padding: 0.5em 1.5em 1.5em 1.5em;
  a {
    display: block;
    float: right;
    font-size: 0.8em;
    color: hsl(0, 0, 25);
  }
}
dt {
  font-weight: bold;
  padding-top: 0.5em;
}
</style>

<i18n lang="yaml">
en:
  identifier: Which citizen identifier do you want to use ?
  back: Back to your birthday
  continue: Continue
fr:
  summaryTitle: Récapitulatif de votre demande de citoyenneté
  name: "Nom:"
  familyName: "Nom de famille:"
  birthday: "Date de naissance:"
  birthplace: "Lieu de naissance:"
  identifier: "Identifiant:"
  password: "Mot de passe:"
  continue: Devenir citoyen
  edit: Modifier
</i18n>
