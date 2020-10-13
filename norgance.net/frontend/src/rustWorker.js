import registerPromiseWorker from 'promise-worker/register';

let rust;

registerPromiseWorker(async (message) => {
  if (!rust) {
    rust = await import('../rust/pkg');
  }
  const functionName = message.function;
  if (typeof rust[functionName] !== 'function') {
    throw new TypeError(`Unknown function ${functionName}`);
  }

  const returnObject = await rust[functionName](...message.args);

  if (returnObject instanceof rust.ChatrouilleUnsignedQuery) {
    // We need to build the final object in the worker
    const finalObject = {
      query: returnObject.get_query(),
      sharedSecret: returnObject.get_shared_secret(),
    };

    // It's important to not leak memory
    returnObject.free();
    return finalObject;
  }

  return returnObject;
});
