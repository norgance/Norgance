<template>
  <div id="app">
    <Header />
    <router-view class="content" />
  </div>
</template>

<script>
import Header from './components/Header.vue';

export default {
  name: 'App',
  components: {
    Header,
  },
};
</script>

<style lang="scss">
//@import "~98.css/dist/98.css";
@import "./styles/formulate";
//@import "./styles/98";
@import "./styles/app";

body, html {
  margin: 0;
  padding: 0;
}

#app {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  box-sizing: border-box;

  & > .content {
    padding:1em;
    flex-grow: 1;
    display: flex;
    flex-direction: column;
  }
}
</style>
