<template>
  <div>
    <FormulateForm @submit="nextStep">
      <FormulateInput
        name="birthday"
        type="date"
        :label="$t('birthday')"
        :help="$t('birthdayHelp')"
        v-model="birthday"
      />
      <p v-if="isInFuture" class="time-traveler">{{ $t("future") }}</p>

      <FormulateInput
        name="birthplace"
        :label="$t('birthplace')"
        :help="$t('birthplaceHelp')"
        v-model="birthplace"
      />
      <router-link :to="{ name: 'CitizenApplicationNames' }"
      tag="button" type="button" class="back-button">{{
        $t("back")
      }}</router-link>
      <FormulateInput type="submit" :name="$t('continue')" />
    </FormulateForm>
  </div>
</template>
<script>
const CURRENT_DATE = new Date();
export default {
  name: 'CitizenApplicationBirthday',
  data() {
    let birthday = '';
    const date = this.$store.state.citizenApplication.birthday;
    if (date) {
      let month = `${date.getMonth() + 1}`;
      let day = `${date.getDate()}`;
      const year = date.getFullYear();

      if (month.length < 2) month = `0${month}`;
      if (day.length < 2) day = `0${day}`;

      birthday = `${year}-${month}-${day}`;
    }
    return {
      birthday,
      birthplace: this.$store.state.citizenApplication.birthplace,
    };
  },
  computed: {
    isVeryOld() {
      const year = this.$store.state.citizenApplication.birthday.getFullYear();
      return CURRENT_DATE.getFullYear() - year > 100;
    },
    isInFuture() {
      return this.$store.state.citizenApplication.birthday > CURRENT_DATE;
    },
    isJustBorn() {
      // 7 days
      return (
        CURRENT_DATE - this.$store.state.citizenApplication.birthday
        < 1000 * 60 * 60 * 24 * 7
      );
    },
  },
  watch: {
    birthday(newBirthday) {
      const birthday = new Date(newBirthday);
      if (!Number.isNaN(birthday.getTime())) {
        this.$store.commit('citizenApplication/updateBirthday', birthday);
      }
    },
    birthplace(newBirthplace) {
      this.$store.commit('citizenApplication/updateBirthplace', newBirthplace);
    },
  },
  methods: {
    nextStep() {
      this.$router.push({ name: 'CitizenApplicationIdentifier' });
    },
  },
};
</script>

<style lang="scss" scoped>
.time-traveler {
  font-size: 0.9em;
  background-image: linear-gradient(to left, hsl(320, 75, 50), hsl(200, 100, 30));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}
</style>

<i18n lang="yaml">
en:
  birthday: When are you born ?
  future: We welcome time travelers.
  back: Back to your names
  continue: Continue
  birthdayError: A birthdate is required.
fr:
  birthday: Quelle est votre date de naissance ?
  future: Les voyageurs dans le temps sont les bienvenus.
  back: Retour aux noms
  continue: Continuer
  birthdayHelp: |
    Votre date de naissance est facultative
    et vous pouvez ne pas remplir le champ.
    Une date de naissance peut être utile pour que
    d'autres personnes puissent vous identifier plus précisement.
    Vous pouvez fournir une date de naissance approximative
    si vous ne connaissez pas votre date de naissance exacte.
  birthplace: Quel est votre lieu de naissance ?
  birthplaceHelp: |
    Votre lieu de naissance est facultatif.
    Vous pouvez le préciser si vous voulez qu'il apparaisse
    sur vos documents ou laisser le champ vide.
</i18n>
