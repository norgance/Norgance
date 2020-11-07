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
  &.large {
    $size: 4em;
    width: $size;
    height: $size;
    border-radius: $size/2;
  }
  &.rainbow {
    animation: spinner 1250ms infinite linear,
      rainbow-spinner 12500ms infinite linear;
    border-color: hsl(340deg, 82%, 52%);
    border-right-color: transparent !important;
  }
}

@keyframes spinner {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

@keyframes rainbow-spinner {
  $steps: 36;
  @for $i from 0 through $steps {
    $kf: $i*100/$steps + "%";
    #{$kf} {
      border-color: hsl($i/$steps * 360, 50%, 50%);
    }
  }
}
</style>
