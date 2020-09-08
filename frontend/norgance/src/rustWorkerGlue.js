let wasm;
// eslint-disable-next-line no-restricted-globals
addEventListener('message', async (event) => {
  if (!wasm) {
    wasm = await import('../rust/pkg');
  }
  postMessage(wasm.derivate_citizen_primary_key(event.data, 'azerty'));
});
