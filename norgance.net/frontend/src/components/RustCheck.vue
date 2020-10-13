<template>
  <div>
    <div v-if="waiting">⏳</div>
    <div v-else>{{ valid ? "✅" : "💩" }}</div>
  </div>
</template>

<script>
import ky from 'ky';
import {
  norganceIdentifier,
  norganceHibpPasswordHash,
  chatrouillePackUnsignedQuery,
  chatrouilleUnpackResponse,
} from '../rust';

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

    const publicKey = Uint8Array.from([
      62,
      183,
      168,
      41,
      176,
      205,
      32,
      245,
      188,
      252,
      11,
      89,
      155,
      111,
      236,
      207,
      109,
      164,
      98,
      113,
      7,
      189,
      176,
      212,
      243,
      69,
      180,
      48,
      39,
      216,
      185,
      114,
      252,
      62,
      52,
      251,
      66,
      50,
      161,
      60,
      167,
      6,
      220,
      181,
      122,
      236,
      61,
      174,
      7,
      189,
      193,
      198,
      123,
      243,
      54,
      9,
    ]);
    const lapin = await chatrouillePackUnsignedQuery(
      JSON.stringify({
        graphql: {
          operationName: 'loadCitizenPublicKey',
          variables: {
            identifier: 'p32JCE3v2HUUAm1Dq9iJbn3nyDs5JNnnG6wIifwb7zl6tZcH2Cjy7JUKdZbCutlJ',
          },
          query:
            'query loadCitizenPublicKey($identifier: String!) { loadCitizenPublicKeys(identifier: $identifier) { publicX448 publicX25519Dalek publicEd25519Dalek }}',
        },
      }),
      publicKey,
    );
    console.log(lapin, lapin.lapin);
    const renard = await ky.post('http://localhost:3000/chatrouille', {
      body: lapin.query,
    });
    console.log(renard);
    const data = await renard.arrayBuffer();
    const packedData = new Uint8Array(data);
    const sharedSecret = lapin.sharedSecret;
    const decoded = await chatrouilleUnpackResponse(packedData, sharedSecret);
    const json = JSON.parse(decoded);
    console.log(json);

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
