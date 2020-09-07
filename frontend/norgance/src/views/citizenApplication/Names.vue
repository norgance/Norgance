<template>
  <div>
    <FormulateForm @submit="nextStep">
      <FormulateInput
        name="firstName"
        :label="$t('firstName')"
        :help="$t('firstNameHelp')"
        :validationName="$t('firstNameValidationName')"
        validation="required"
        v-model="firstName"
        required
      />
      <FormulateInput
        name="familyName"
        :label="$t('familyName')"
        :help="$t('familyNameHelp')"
        v-model="familyName"
      />
      <FormulateInput type="submit" :name="$t('continue')" />
    </FormulateForm>
  </div>
</template>
<script>
export default {
  name: 'CitizenApplicationNames',
  computed: {
    firstName: {
      get() {
        return this.$store.state.citizenApplication.firstName;
      },
      set(name) {
        this.$store.commit('citizenApplication/updateFirstName', name);
      },
    },
    familyName: {
      get() {
        return this.$store.state.citizenApplication.familyName;
      },
      set(name) {
        this.$store.commit('citizenApplication/updateFamilyName', name);
      },
    },
  },
  methods: {
    nextStep() {
      this.$router.push({ name: 'CitizenApplicationBirthday' });
    },
  },
};
</script>

<i18n lang="yaml">
en:
  firstName: What is your name ?
  familyName: What is your family name ?
  firstNameHelp: Your name is required.
  familyNameHelp: |
    The family name is optional.
    Leave this field empty if you do not wish to have a family name.
  firstNameValidationName: Name
  continue: Continue
fr:
  firstName: Quel est votre nom ?
  familyName: Quel est votre nom de famille ?
  firstNameHelp: |
    Votre nom permettant de vous désigner.
    Il est obligatoire et il apparaîtra sur vos documents.
  familyNameHelp: |
    Un nom de famille est facultatif.
    Laissez ce champ libre si vous ne souhaitez pas avoir de nom de famille.
  firstNameValidationName: Nom
  continue: Continuer
</i18n>
