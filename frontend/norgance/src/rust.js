import PromiseWorker from 'promise-worker';
import mem from 'mem';

const worker = new Worker('./rustWorker.js', { name: 'rustWorker', type: 'module' });
const promiseWorker = new PromiseWorker(worker);

async function derivateCitizenPrimaryKeyNoCache(username) {
  return promiseWorker.postMessage({
    function: 'derivate_citizen_primary_key',
    args: [username],
  });
}

export const derivateCitizenPrimaryKey = mem(derivateCitizenPrimaryKeyNoCache);

export async function norgancePasswordHash(password, size = 16) {
  return promiseWorker.postMessage({
    function: 'norgance_password_hash',
    args: [password, size],
  });
}

export default promiseWorker;
