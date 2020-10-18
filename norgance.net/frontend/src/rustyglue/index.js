import mem from 'mem';
import PromiseWorker from './rustPromiseWorker';
import entropy from '../entropy';
import * as classes from './classes';

const worker = new Worker('./rustWorker.js', { name: 'rustWorker', type: 'module' });
const promiseWorker = new PromiseWorker(worker, classes);

export const norganceIdentifier = mem((identifier) => promiseWorker.call(
  'norgance_identifier', {
    args:
  [identifier],
  },
));

export const norganceCitizenSymmetricKey = mem(
  (identifier, password) => promiseWorker.call('norgance_citizen_symmetric_key', {
    args:
    [identifier, password],
  }), {
    cacheKey: JSON.stringify,
  },
);

export const norganceCitizenAccessKey = mem(
  (identifier, password) => promiseWorker.call('norgance_citizen_access_key', {
    args:
    [identifier, password],
  }), {
    cacheKey: JSON.stringify,
  },
);

export const norganceHibpPasswordHash = mem((password, size = 20) => promiseWorker.call('norgance_hibp_password_hash', {
  args:
  [password, size],
}), {
  cacheKey: JSON.stringify,
});

export function chatrouillePackUnsignedQuery(payload, publicKey) {
  return promiseWorker.call('chatrouille_pack_unsigned_query', {
    args: [payload, publicKey],
    preload: {
      query: { functionName: 'get_query' },
      sharedSecret: { functionName: 'get_shared_secret' },
    },
    returnClassName: 'ChatrouilleUnsignedQuery',
    freeResponseImmediately: true,
  });
}

export function chatrouilleUnpackResponse(packedData, sharedSecret) {
  return promiseWorker.call('chatrouille_unpack_response', {
    args: [packedData, sharedSecret],
    transfer: [packedData.buffer, sharedSecret.buffer],
  });
}

export function makeNorganceRng() {
  const entropyData = entropy().data;
  return promiseWorker.call('make_norgance_rng', {
    args: [entropyData],
    // We don't transfer the data, we copy it
    transfer: [],
    returnClassName: 'NorganceRng',
  });
}

export function genX448PrivateKey(rng) {
  return promiseWorker.call('gen_x448_private_key', { args: [rng] });
}
