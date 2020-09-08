<template>
  <div>{{ hash }}</div>
</template>

<script>
export default {
  name: 'RustCheck',
  data() {
    return {
      hash: '',
    };
  },
  async mounted() {
    const piWorker = new Worker('../rustWorkerGlue.js', { name: 'rustWorkerGlue', type: 'module' });
    piWorker.onmessage = (event) => {
      console.log(event.data);
      this.hash = event.data;
    };
    piWorker.postMessage('42');
  },
};
</script>
