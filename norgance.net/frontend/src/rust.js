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

export const norganceHibpPasswordHash = mem((password, size = 20) => promiseWorker.postMessage({
  function: 'norgance_hibp_password_hash',
  args: [password, size],
}), {
  cacheKey: JSON.stringify,
});

export function chatrouillePackUnsignedQuery(payload, publicKey) {
  return promiseWorker.postMessage({
    function: 'chatrouille_pack_unsigned_query',
    args: [payload, publicKey],
  });
}

export function chatrouilleUnpackResponse(packedData, sharedSecret) {
  return promiseWorker.postMessage({
    function: 'chatrouille_unpack_response',
    args: [packedData, sharedSecret],
  });
}

export default promiseWorker;
