<template>
  <div>
    <FormulateForm @submit="nextStep">
      <FormulateInput
        name="identifier"
        :label="$t('identifier')"
        :help="$t('identifierHelp')"
        v-model="identifier"
        validation="matches:/^[^\s]*$/"
        :validation-messages="{ matches: $t('noSpaces') }"
        error-behavior="live"
        required
      />
      <p v-if="specialCharacters" class="special-characters">
        {{ $t('specialCharacters')}}
      </p>
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
  name: 'CitizenApplicationIdentifier',
  computed: {
    identifier: {
      get() {
        return this.$store.state.citizenApplication.identifier;
      },
      set(identifier) {
        this.$store.commit('citizenApplication/updateIdentifier', identifier);
      },
    },
    specialCharacters() {
      return /\s/.test(this.identifier) ? false
        : !/^[a-zA-Z0-9.\-_@]*$/.test(this.identifier);
    },
  },
  methods: {
    nextStep() {
      this.$router.push({ name: 'CitizenApplicationPassword' });
    },
  },
};
</script>

<style lang="scss" scoped>
.special-characters {
  font-size: 0.9em;
  color: #f44336;
}
</style>

<i18n lang="yaml">
en:
  identifier: Which citizen identifier do you want to use ?
  back: Back to your birthday
  continue: Continue
  specialCharacters: |
    Your identifier contains special characters.
    Some people may have difficulties to write correctly
    your identifier.
  noSpaces: Your identifier must not contain spaces.
fr:
  identifier: Quel identifiant de citoyen souhaitez-vous utiliser ?
  identifierHelp: |
    Votre identiant est unique et personnel.
    Il n'est pas secret mais Norgance ne le connait pas
    (Norgance utilise une signature numérique de votre identifiant).
    Vous pouvez partager votre identifiant à des personnes de confiance,
    notamment à votre futur épous·e ou vos enfants.
  specialCharacters: |
    Votre identifant contient des caractères spéciaux.
    Certaines personnes peuvent avoir des difficultés
    pour saisir votre identifiant correctement.
  noSpaces: Votre identifiant ne doit pas contenir d'espaces.
  back: Retour à vos informations de naissance.
  continue: Continuer
</i18n>
