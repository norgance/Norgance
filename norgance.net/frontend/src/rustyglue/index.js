import mem from 'mem';
import PromiseWorker from './rustPromiseWorker';
import * as classes from './classes';

const worker = new Worker('./rustWorker.js', { name: 'rustWorker', type: 'module' });
const promiseWorker = new PromiseWorker(worker, classes);

export const debugClasses = classes;

export const norganceIdentifier = mem((identifier) => promiseWorker.call(
  'norgance_identifier', {
    args:
  [identifier],
  },
));

// TODO memory leaks with mem and rust ?
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

export function genX448PrivateKey(rng) {
  return promiseWorker.call('gen_x448_private_key', { args: [rng] });
}
