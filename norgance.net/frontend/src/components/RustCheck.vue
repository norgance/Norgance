<template>
  <div>
    <div v-if="waiting">⏳</div>
    <div v-else-if="!valid">💩</div>
  </div>
</template>

<script>
import {
  norganceHibpPasswordHash,
} from '../rustyglue';

export default {
  name: 'RustCheck',
  data() {
    return {
      waiting: true,
      valid: false,
    };
  },
  async mounted() {
    // We allow one minute to load the rust environment
    // and return a result.
    const timeoutId = setTimeout(() => {
      this.waiting = false;
      this.valid = false;
      console.error('Rust timeout');
    }, 60000);

    try {
      // We compute a hash that we already know
      // to check whether the computations are valid and correct.
      const hash = await norganceHibpPasswordHash('0000❤');
      if (this.waiting) {
        this.valid = hash === '68017291cec61fd22528f9bdcbb70a16740dee05';
      }
    } catch (error) {
      this.valid = false;
      console.error(error);
    } finally {
      clearTimeout(timeoutId);
      this.waiting = false;
    }
  },
};
</script>
