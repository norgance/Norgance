<!--
From https://github.com/jlong/css-spinners/
MIT License
-->

<template>
  <div class="spinner"></div>
</template>

<style lang="scss" scoped>
.spinner {
  margin-left: 0.5em;
  animation: spinner 1250ms infinite linear;
  border: 0.25em solid white;
  border-right-color: transparent;
  border-radius: 0.5em;
  box-sizing: border-box;
  display: inline-block;
  position: relative;
  overflow: hidden;
  text-indent: -9999px;
  width: 1em;
  height: 1em;
}

@keyframes spinner {
 0% {
   transform: rotate(0deg);
 }
 100% {
   transform: rotate(360deg);
 }
}
</style>
