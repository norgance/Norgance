import PromiseWorker from 'promise-worker';

const worker = new Worker('./rustWorker.js', { name: 'rustWorker', type: 'module' });
const promiseWorker = new PromiseWorker(worker);

export async function derivateCitizenPrimaryKey(username, password) {
  return promiseWorker.postMessage({
    function: 'derivate_citizen_primary_key',
    args: [username, password],
  });
}

export default promiseWorker;
