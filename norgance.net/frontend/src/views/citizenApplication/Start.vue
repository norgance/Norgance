<template>
  <div>
    <p>{{ $t("disclaimer") }}</p>
    <p class="start">
    <router-link :to="{ name: 'CitizenApplicationNames' }" class="rainbow-button" tag="button">{{
      $t("start")
    }}</router-link>
    </p>
  </div>
</template>
<script>
export default {
  name: 'CitizenApplicationStart',
};
</script>

<style lang="scss" scoped>
p:not(.start) {
  font-size: 0.9em;
  color: #6d6d6d;
  margin-top: 0.2em;
}
p.start {
  text-align: center;
}
</style>

<i18n lang="yaml">
en:
  start: Start !
  disclaimer: |
    You have to fill a citizenship form to become a Norgance citizen.
    Your application will be accepted immediately once you finalize and submit it.
    The information are private and not shared without your agreement.
    However, if you prefer to not fill specific parts of the form, do not fill them.
fr:
  start: Commencer !
  disclaimer: |
    Pour devenir citoyen de Norgance, vous devez remplir un formulaire de citoyenneté.
    Votre citoyenneté sera immédiatement acceptée une fois votre demande finalisée et envoyée.
    Les informations sont privées et non partagées sans votre accord.
    Si toutefois vous ne souhaitez pas remplir certains champs du formulaire, ne le faites pas.

</i18n>
