<template>
  <router-link :to="{name: 'CitizenApplicationNames'}">{{$t('start')}}</router-link>
</template>
<script>
export default {
  name: 'CitizenApplicationStart',
};
</script>

<i18n lang="yaml">
en:
  start: Start !
fr:
  start: Commencer !
</i18n>
