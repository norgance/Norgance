<template>
  <div>
    <FormulateForm @submit="nextStep" :class="{ loading }">
      <p class="warning">{{ $t("passwordWarning") }}</p>
      <p class="warning-following">{{ $t("passwordWarningFollowing") }}</p>
      <FormulateInput
        name="password"
        type="password"
        :label="$t('password')"
        :help="$t('passwordHelp')"
        v-model="password"
        required
        :validationName="$t('passwordValidationName')"
        validation="min:6,length|validationNoName|validationNoIdentifier"
        :validation-rules="{ validationNoIdentifier, validationNoName }"
        :validation-messages="{
          validationNoName: $t('validationNoName'),
          validationNoIdentifier: $t('validationNoIdentifier'),
        }"
        :disabled="loading"
      />
      <FormulateInput
        name="passwordConfirm"
        type="password"
        :label="$t('passwordConfirm')"
        :help="$t('passwordConfirmHelp')"
        v-model="passwordConfirm"
        required
        validation="validationCopy"
        :validation-rules="{ validationCopy }"
        :validation-messages="{
          validationCopy: $t('validationCopy'),
        }"
        :disabled="loading"
      />
      <p v-if="worstPassword" class="worst-password">
        {{ $t("worstPassword") }}
      </p>
      <p v-if="terriblePassword" class="terrible-password">
        {{ $t("terriblePassword") }}
      </p>
      <p v-if="badPassword" class="bad-password">
        {{ $t("badPassword") }}
      </p>
      <p v-if="rarePassword" class="rare-password">
        {{ $t("rarePassword") }}
      </p>
      <p v-if="error" class="error">
        {{ $t("error") }}
      </p>
      <FormulateInput
        v-model="doNotCareAboutThePassword"
        v-if="badPassword || terriblePassword"
        required
        type="checkbox"
        :label="$t('doNotCareAboutThePassword')"
      />
      <FormulateInput
        v-model="noRegrets"
        v-if="doNotCareAboutThePassword"
        required
        type="checkbox"
        :label="$t('noRegrets')"
      />
      <FormulateInput type="submit">
        {{ $t("continue") }}
        <Spinner v-if="loading" />
      </FormulateInput>
      <router-link :to="{ name: 'CitizenApplicationIdentifier' }">{{
        $t("back")
      }}</router-link>
    </FormulateForm>
  </div>
</template>
<script>
import Spinner from '../../components/Spinner.vue';

export default {
  name: 'CitizenApplicationPassword',
  components: {
    Spinner,
  },
  data() {
    return {
      password: this.$store.state.citizenApplication.password || '',
      passwordConfirm: this.$store.state.citizenApplication.password || '',
      loading: false,
      error: false,
      rarePassword: false,
      badPassword: false,
      terriblePassword: false,
      worstPassword: false,
      doNotCareAboutThePassword: false,
      noRegrets: false,
      dirty: true,
    };
  },
  computed: {
    name() {
      return this.$store.state.citizenApplication.name;
    },
    identifier() {
      return this.$store.state.citizenApplication.identifier;
    },
  },
  methods: {
    async nextStep() {
      if (this.loading || this.worstPassword) return;

      this.loading = true;
      const timeoutId = setTimeout(() => {
        this.fail();
      }, 30_000);

      let goodPassword = false;

      if (this.dirty) {
        try {
          this.$store.commit('citizenApplication/updatePassword', this.password);
          const quality = await this.$store.dispatch(
            'citizenApplication/checkPasswordQuality',
          );
          this.badPassword = false;
          this.terriblePassword = false;
          this.worstPassword = false;
          switch (quality) {
            case 'F':
            case 'E':
              this.worstPassword = true;
              break;
            case 'D':
            case 'C':
            case 'B':
            case 'A':
              this.terriblePassword = true;
              break;
            case '9':
            case '8':
            case '7':
            case '6':
            case '5':
            case '4':
            case '3':
            case '2':
              this.badPassword = true;
              break;
            case '1':
            case '0':
              this.badPassword = true;
              this.rarePassword = true;
              break;
            case 'good':
            default:
              goodPassword = true;
              break;
          }
        } catch (error) {
          this.error = true;
          console.error(error);
        } finally {
          clearTimeout(timeoutId);
          this.loading = false;
        }
        this.dirty = false;
      }
      if (goodPassword || (this.doNotCareAboutThePassword && this.noRegrets)) {
        this.$router.push({ name: 'CitizenApplicationSummary' });
      }
    },
    fail() {
      this.loading = false;
      this.error = true;
      this.resetPasswordQuality();
    },
    resetPasswordQuality() {
      this.rarePassword = false;
      this.badPassword = false;
      this.terriblePassword = false;
      this.worstPassword = false;
      this.doNotCareAboutThePassword = false;
      this.noRegrets = false;
    },
    validationNoName(context) {
      return context.value !== this.name;
    },
    validationNoIdentifier(context) {
      return context.value !== this.identifier || context.value === this.name;
    },
    validationCopy(context) {
      return context.value === this.password;
    },
  },
  mounted() {
    if (!this.name) {
      this.$router.push({ name: 'CitizenApplicationNames' });
    } else if (!this.identifier) {
      this.$router.push({ name: 'CitizenApplicationIdentifier' });
    }
  },
  watch: {
    password() {
      this.dirty = true;
      this.resetPasswordQuality();
    },
  },
};
</script>

<style lang="scss" scoped>
.worst-password,
.terrible-password,
.bad-password,
.error {
  color: #f44336;
}
.rare-password {
  color: #9C27B0;
}
.error {
  font-size: 0.9em;
}
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
  passwordValidationName: Votre mot de passe
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
  validationNoName: Votre mot de passe ne doit pas être votre nom.
  validationNoIdentifier: Votre mot de passe ne doit pas être votre identifiant.
  validationCopy: |
    Vous avez saisie deux mots de passe différents.
    Veuillez vérifier la saisie de vos mots de passe.
  worstPassword: |
    Le mot de passe que vous avez choisi est beaucoup trop facile à deviner.
    Vous devez en choisir un autre.
  terriblePassword: |
    Le mot de passe que vous avez choisi est très courant.
    Il est très fortement conseillé d'en choisir un autre.
  badPassword: |
    Le mot de passe que vous avez choisi n'est pas sécurisé
    car il fait partie des listes de mots de passes courants.
    Il est très fortement conseillé d'en choisir un autre.
  rarePassword: |
    Si vous pensez être la seule personne qui utilise ce mot de passe,
    vous devriez changer de mot de passe partout où vous l'avez utilisé.
  doNotCareAboutThePassword: Utiliser ce mot de passe quand même.
  noRegrets: J'assume les risques.
</i18n>
