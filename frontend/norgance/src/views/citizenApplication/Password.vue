<template>
  <div>
    <FormulateForm @submit="nextStep">
      <p class="warning">{{$t('passwordWarning')}}</p>
      <p class="warning-following">{{$t('passwordWarningFollowing')}}</p>
      <FormulateInput
        name="password"
        type="password"
        :label="$t('password')"
        :help="$t('passwordHelp')"
        v-model="password"
        required
      />
      <FormulateInput
        name="passwordConfirm"
        type="password"
        :label="$t('passwordConfirm')"
        :help="$t('passwordConfirmHelp')"
        v-model="passwordConfirm"
        required
      />
      <FormulateInput type="submit" :name="$t('continue')" />
      <router-link
        :to="{ name: 'CitizenApplicationIdentifier' }"
        >{{ $t("back") }}</router-link
      >
    </FormulateForm>
  </div>
</template>
<script>
export default {
  name: 'CitizenApplicationPassword',
  data() {
    return {
      password: '',
      passwordConfirm: '',
    };
  },
  methods: {
    nextStep() {
      this.$router.push({ name: 'CitizenApplicationPassword' });
    },
  },
  mounted() {
    if (!this.$store.state.citizenApplication.identifier) {
      this.$router.push({ name: 'CitizenApplicationIdentifier' });
    } else if (!this.$store.state.citizenApplication.name) {
      this.$router.push({ name: 'CitizenApplicationNames' });
    }
  },
};
</script>

<style lang="scss" scoped>
p.warning {
  font-size: 0.9em;
  font-weight: bold;
  color: black;
  margin-bottom: 0em;
}
p.warning-following {
  font-size: 0.9em;
  color: #6d6d6d;
  margin-top: 0.2em;
}
</style>

<i18n lang="yaml">
en:
  identifier: Which citizen identifier do you want to use ?
  back: Back to your birthday
  continue: Continue
fr:
  password: Votre mot de passe
  passwordConfirm: Votre mot de passe une seconde fois
  passwordHelp: |
    Votre mot de passe doit être de bonne qualité,
    difficile à deviner pour d'autres mais facile à retenir pour vous.
    Idéalement votre mot de passe ne doit pas être utilisé ailleurs que sur Norgance.
  passwordWarning: |
    Votre mot de passe est garant de votre identité.
  passwordWarningFollowing: |
    Vous ne pouvez pas en obtenir un nouveau si vous l'oubliez car
    votre mot de passe est utilisé pour chiffrer vos données sur Norgance.
    Si vous pensez qu'il est possible que vous l'oubliez avec le temps,
    vous pouvez le stocker dans un gestionnaire de mot de passe
    ou tout simplement l'écrire sur papier.
  passwordConfirmHelp: |
    Recopier votre mot de passe permet de vérifier
    que vous n'avez pas fait d'erreur de saisie.
  back: Retour à votre identiant.
  continue: Continuer
</i18n>
