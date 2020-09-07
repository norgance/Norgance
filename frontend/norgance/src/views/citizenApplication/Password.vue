<template>
  <div>
    <FormulateForm @submit="nextStep">
      <FormulateInput
        name="password"
        type="password"
        :label="$t('password')"
        :help="$t('passwordHelp')"
        v-model="password"
        required
      />
      <FormulateInput
        name="password"
        type="password"
        :label="$t('passwordConfirm')"
        :help="$t('passwordConfirmHelp')"
        v-model="passwordCopy"
        required
      />
      <router-link
        :to="{ name: 'CitizenApplicationIdentifier' }"
        tag="button"
        type="button"
        class="back-button"
        >{{ $t("back") }}</router-link
      >
      <FormulateInput type="submit" :name="$t('continue')" />
    </FormulateForm>
  </div>
</template>
<script>
export default {
  name: 'CitizenApplicationPassword',
  data() {
    return {
      password: '',
      passwordCopy: '',
    };
  },
  methods: {
    nextStep() {
      this.$router.push({ name: 'CitizenApplicationPassword' });
    },
  },
};
</script>

<i18n lang="yaml">
en:
  identifier: Which citizen identifier do you want to use ?
  back: Back to your birthday
  continue: Continue
fr:
  identifier: Quel identifiant de citoyen souhaitez-vous utiliser ?
  identifierHelp: |
    Votre identiant est unique et personnel.
    Il n'est pas secret mais Norgance ne le connait pas
    (Norgance utilise la signature numérique de votre identifiant).
    Vous pouvez partager votre identifiant à des personnes de confiance,
    notamment à votre futur épous·e ou vos enfants.
  back: Retour à votre identiant.
  continue: Continuer
</i18n>
