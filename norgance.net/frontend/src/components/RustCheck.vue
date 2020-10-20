<template>
  <div>
    <div v-if="waiting">⏳</div>
    <div v-else>{{ valid ? "✅" : "💩" }}</div>
  </div>
</template>

<script>
import {
  norganceIdentifier,
  norganceHibpPasswordHash,
} from '../rustyglue';
import { anonymousGraphql } from '../chatrouille';

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
        this.valid = hash === 'ac2abc41328eb62dbf5bb4f6ae2fa4ce';
      }
    } catch (error) {
      this.valid = false;
      console.error(error);
    } finally {
      clearTimeout(timeoutId);
      this.waiting = false;
    }

    const lapin = await anonymousGraphql({
      operationName: 'loadCitizenPublicKey',
      variables: {
        identifier: 'p32JCE3v2HUUAm1Dq9iJbn3nyDs5JNnnG6wIifwb7zl6tZcH2Cjy7JUKdZbCutlJ',
      },
      query:
            'query loadCitizenPublicKey($identifier: String!) { loadCitizenPublicKeys(identifier: $identifier) { publicX25519Dalek publicEd25519Dalek }}',
    });
    console.log(lapin);

    console.time('canard');
    const pa = await norganceIdentifier('canard');
    console.timeEnd('canard');
    console.log(pa);
    console.time('canard');
    const pa2 = await norganceIdentifier('canard');
    console.log(pa2);
    console.timeEnd('canard');
  },
};
</script>
