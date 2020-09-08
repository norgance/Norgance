<template>
  <div>{{ hash }}</div>
</template>

<script>
import { derivateCitizenPrimaryKey } from '../rust';

export default {
  name: 'RustCheck',
  data() {
    return {
      hash: '',
    };
  },
  async mounted() {
    this.hash = await derivateCitizenPrimaryKey(
      '42', 'azerty',
    );
  },
};
</script>
