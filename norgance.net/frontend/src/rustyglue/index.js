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
// TODO remove that
export const norganceCitizenSymmetricKey = mem(
  (identifier, password) => promiseWorker.call('norgance_citizen_symmetric_key', {
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
