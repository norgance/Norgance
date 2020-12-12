<template>
  <div>
    <FormulateForm @submit="nextStep">
      <FormulateInput
        name="name"
        :label="$t('name')"
        :help="$t('nameHelp')"
        :validationName="$t('nameValidationName')"
        validation="required"
        v-model="name"
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
    name: {
      get() {
        return this.$store.state.citizenApplication.name;
      },
      set(name) {
        this.$store.commit('citizenApplication/updateName', name);
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
  name: What is your name ?
  familyName: What is your family name ?
  nameHelp: |
    Your name will designate you.
    It is mandatory and will appear on your documents.
  familyNameHelp: |
    The family name is optional.
    Leave this field empty if you do not wish to have a family name.
  nameValidationName: Name
  continue: Continue
fr:
  name: Quel est votre nom ?
  familyName: Quel est votre nom de famille ?
  nameHelp: |
    Votre nom sert à vous désigner.
    Il est obligatoire et il apparaîtra sur vos documents.
  familyNameHelp: |
    Un nom de famille est facultatif.
    Laissez ce champ libre si vous ne souhaitez pas avoir de nom de famille.
  nameValidationName: Nom
  continue: Continuer
</i18n>
