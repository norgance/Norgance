import PromiseWorker from 'promise-worker';
import mem from 'mem';

const worker = new Worker('./rustWorker.js', { name: 'rustWorker', type: 'module' });
const promiseWorker = new PromiseWorker(worker);

export const norganceIdentifier = mem((identifier) => promiseWorker.postMessage({
  function: 'norgance_identifier',
  args: [identifier],
}));

export const norganceCitizenSymmetricKey = mem(
  (identifier, password) => promiseWorker.postMessage({
    function: 'norgance_citizen_symmetric_key',
    args: [identifier, password],
  }), {
    cacheKey: JSON.stringify,
  },
);

export const norganceCitizenAccessKey = mem(
  (identifier, password) => promiseWorker.postMessage({
    function: 'norgance_citizen_access_key',
    args: [identifier, password],
  }), {
    cacheKey: JSON.stringify,
  },
);

export const norganceHibpPasswordHash = mem((password, size = 16) => promiseWorker.postMessage({
  function: 'norgance_hibp_password_hash',
  args: [password, size],
}), {
  cacheKey: JSON.stringify,
});

export default promiseWorker;
