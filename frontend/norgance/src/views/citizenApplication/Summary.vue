<template>
  <div>
    <FormulateForm @submit="nextStep">
      <FormulateInput
        name="identifier"
        :label="$t('identifier')"
        :help="$t('identifierHelp')"
        v-model="identifier"
        required
      />
      <router-link
        :to="{ name: 'CitizenApplicationBirthday' }"
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
  name: 'CitizenApplicationSummary',
  computed: {
    identifier: {
      get() {
        return this.$store.state.citizenApplication.identifier;
      },
      set(identifier) {
        this.$store.commit('citizenApplication/updateIdentifier', identifier);
      },
    },
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
  back: Retour à vos informations de naissance.
  continue: Continuer
</i18n>
