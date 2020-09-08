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
  
  return rust[functionName].apply(rust, message.args);
});