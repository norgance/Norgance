import PromiseWorker from './rustPromiseWorker';
import * as classes from './classes';

const worker = new Worker('./rustWorker.js', { name: 'rustWorker', type: 'module' });
const promiseWorker = new PromiseWorker(worker, classes);

export const debugClasses = classes;

export function norganceIdentifier(identifier) {
  return promiseWorker.call(
    'norgance_identifier', {
      args: [identifier],
    },
  );
}

export function norganceHibpPasswordHash(password, size = 20) {
  return promiseWorker.call('norgance_hibp_password_hash', {
    args: [password, size],
  });
}
