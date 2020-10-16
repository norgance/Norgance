let rust;
self.addEventListener('message', async (event) => {
  const { messageId, functionName, args } = event.data;

  if (typeof messageId === 'undefined') {
    return;
  }

  try {
    // Lazy loading
    if (!rust) {
      rust = await import('../rust/pkg');
    }

    if (typeof rust[functionName] !== 'function') {
      throw new TypeError(`Unknown function ${functionName}`);
    }

    let returnObject = await rust[functionName](...args);
    let transferables;

    if (returnObject instanceof rust.ChatrouilleUnsignedQuery) {
      // We need to build the final object in the worker
      const finalObject = {
        query: returnObject.get_query(),
        sharedSecret: returnObject.get_shared_secret(),
      };

      transferables = [
        finalObject.query.buffer,
        finalObject.sharedSecret.buffer,
      ];

      // It's important to not leak memory
      returnObject.free();
      returnObject = finalObject;
    } else if (returnObject) {
      if (returnObject instanceof ArrayBuffer) {
        transferables = [returnObject];
      } else if (returnObject.buffer instanceof ArrayBuffer) {
        transferables = [returnObject.buffer];
      }
    }

    self.postMessage({
      messageId,
      response: returnObject,
    }, transferables);

  } catch (error) {
    console.error(error);
    self.postMessage({
      messageId,
      error,
    });
  }
});
