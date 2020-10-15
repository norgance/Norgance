<template>
  <div>
    <FormulateForm @submit="nextStep" :class="{loading}">
      <FormulateInput
        class="identifier-input"
        name="identifier"
        :label="$t('identifier')"
        :help="$t('identifierHelp')"
        v-model="identifier"
        validation="matches:/^[^\s]*$/"
        :pattern="regexPattern"
        :validation-messages="{ matches: $t('noSpaces') }"
        error-behavior="live"
        required
        :disabled="loading"
      />
      <p v-if="specialCharacters" class="special-characters">
        {{ $t("specialCharacters") }}
      </p>
      <p v-if="confusingCharacters" class="confusing-characters">
        {{ $t("confusingCharacters") }}
      </p>
      <p v-if="error" class="error">
        {{ $t("error") }}
      </p>
      <p v-if="alreadyUsed" class="already-used">
        {{ $t("alreadyUsed") }}
      </p>
      <FormulateInput type="submit">
        {{ $t("continue") }}
        <Spinner v-if="loading" />
      </FormulateInput>
      <router-link :to="{ name: 'CitizenApplicationBirthday' }">{{
        $t("back")
      }}</router-link>
    </FormulateForm>
  </div>
</template>
<script>
import escapeRegExp from 'lodash.escaperegexp';
import Spinner from '../../components/Spinner.vue';

export default {
  name: 'CitizenApplicationIdentifier',
  components: {
    Spinner,
  },
  data() {
    return {
      identifier: this.$store.state.citizenApplication.identifier || '',
      loading: false,
      error: false,
      alreadyUsed: !this.$store.state.citizenApplication.identifierIsAvailable,
    };
  },
  computed: {
    specialCharacters() {
      return /\s/.test(this.identifier)
        ? false
        : !/^[a-zA-Z0-9.\-_@]*$/.test(this.identifier);
    },
    confusingCharacters() {
      return /(I.*l)|(l.*I)/.test(this.identifier);
    },
    identifierIsAvailable() {
      return this.$store.state.citizenApplication.identifierIsAvailable;
    },
    regexPattern() {
      // If the identifier is already used,
      // We build a regex to reject it.
      if (this.alreadyUsed) {
        return `(?!${escapeRegExp(this.identifier)})`;
      }
      return '[^\\s]*';
    },
  },
  methods: {
    async nextStep() {
      if (this.loading) return;

      this.loading = true;
      const timeoutId = setTimeout(() => {
        this.fail();
      }, 30_000);

      try {
        console.time('identifier');
        await this.$store.dispatch('citizenApplication/setIdentifier', this.identifier);
        await this.$store.dispatch('citizenApplication/checkIdentifierAvailability');
        this.alreadyUsed = !this.identifierIsAvailable;
        if (this.loading && this.identifierIsAvailable) {
          this.$router.push({ name: 'CitizenApplicationPassword' });
        }
        console.timeEnd('identifier');
        this.error = false;
      } catch (error) {
        this.error = true;
        console.error(error);
      } finally {
        clearTimeout(timeoutId);
        this.loading = false;
      }
    },
    fail() {
      this.loading = false;
      this.error = true;
      this.alreadyUsed = false;
    },
  },
  mounted() {
    if (!this.$store.state.citizenApplication.name) {
      this.$router.push({ name: 'CitizenApplicationNames' });
    }
  },
  watch: {
    identifierIsAvailable(newAvailability) {
      console.log('lol', newAvailability);
      this.alreadyUsed = !newAvailability;
    },
    identifier() {
      this.alreadyUsed = false;
    },
  },
};
</script>

<style lang="scss" scoped>
.special-characters,
.confusing-characters,
.error,
.already-used {
  font-size: 0.9em;
  color: #f44336;
}
/deep/ form.loading button[type="submit"] {
  animation-duration: 60s;
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
  confusingCharacters: |
    Your identifier contains characters that are sometimes difficult to distinguish,
    such as a lower case L and an uppercase i (I and l).
    Try to use only uppercase or lowercase letters in your identifier.
  noSpaces: Your identifier must not contain spaces.
fr:
  identifier: Quel identifiant de citoyen souhaitez-vous utiliser ?
  identifierHelp: |
    Votre identiant est unique et personnel.
    Il n'est pas secret mais Norgance ne le connait pas
    (Norgance utilise une signature numérique de votre identifiant).
    Vous ne devez pas le perdre et il est conseillé de
    le conserver précieusement.
    Certaines procédures vous demanderont de partager
    votre identifiant à des personnes de confiance,
    par exemple à votre épous·e ou à vos enfants.
  specialCharacters: |
    Votre identifant contient des caractères spéciaux.
    Certaines personnes peuvent avoir des difficultés
    pour saisir votre identifiant correctement.
  confusingCharacters: |
    Votre identifiant contient des caractères parfois difficiles à déterminer,
    tel qu'un L minuscule et un i majuscule (I et l).
    Essayer d'utiliser que des lettres minuscules et majuscules dans votre identifiant.
  alreadyUsed: |
    Désolé mais votre identifiant n'est pas disponible.
    Vous devez utiliser un autre identifiant de citoyen.
  noSpaces: Votre identifiant ne doit pas contenir d'espaces.
  back: Retour à vos informations de naissance.
  continue: Continuer
  error: Une erreur est survenue. Merci de réessayer ou de signaler le problème.
</i18n>
