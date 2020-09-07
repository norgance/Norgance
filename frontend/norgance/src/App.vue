<template>
  <div id="app">
    <div id="nav" class="w98">
      <router-link tag="button" class="w98" to="/" exact>Home</router-link>
      <router-link tag="button" to="/about" class="w98">About</router-link>
      <router-link tag="button" class="w98" :to="{
        name: 'CitizenApplicationStart',
        }"> Become a citizen</router-link>
      <router-link tag="button" to="/about1" class="w98">About</router-link>
      <router-link tag="button" to="/about2" class="w98">About</router-link>
      <router-link tag="button" to="/about2" class="w98">About</router-link>
      <router-link tag="button" to="/about3" class="w98">About</router-link>
      <router-link tag="button" to="/about4" class="w98">About</router-link>
      <router-link tag="button" to="/about5" class="w98">About</router-link>
      <router-link tag="button" to="/about6" class="w98">About</router-link>
      <router-link tag="button" to="/about8" class="w98">About</router-link>
      <router-link tag="button" to="/about7" class="w98">About</router-link>
      <router-link tag="button" to="/about9" class="w98">About</router-link>
      <router-link tag="button" to="/about10" class="w98">About</router-link>
      <router-link tag="button" to="/about11" class="w98">About</router-link>
    </div>
    <router-view class="content"/>
  </div>
</template>

<style lang="scss">
//@import "~98.css/dist/98.css";
@import "./styles/formulate";
//@import "./styles/98";
@import "./styles/app";
</style>
