<template>
  <div>
    <div v-if="waiting">⏳</div>
    <div v-else>{{ valid ? '✅' : '💩' }}</div>
  </div>
</template>

<script>
import { norgancePasswordHash } from '../rust';

export default {
  name: 'RustCheck',
  data() {
    return {
      waiting: true,
      valid: false,
    };
  },
  async mounted() {
    // We allow 30 seconds to load the rust environment
    // and return a result.
    const timeoutId = setTimeout(() => {
      this.waiting = false;
      this.valid = false;
      console.error('Rust timeout');
    }, 30000);

    try {
      // We compute a hash that we already know
      // to check whether the computations are valid and correct.
      const hash = await norgancePasswordHash('0000❤');
      if (this.waiting) {
        this.valid = hash === 'ac2abc41328eb62dbf5bb4f6ae2fa4ce';
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
