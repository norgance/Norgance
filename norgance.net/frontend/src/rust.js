import mem from 'mem';

class PromiseWorker {
  constructor(worker) {
    this.worker = worker;
    this.callbacks = new Map();
    this.currentMessageId = 0;

    worker.addEventListener('message', this.onMessage.bind(this));
  }

  onMessage(event) {
    const data = event.data;
    if (!data) return;

    const { messageId } = data;
    const callback = this.callbacks.get(messageId);

    // Ignore messages that are not from us
    if (!callback) {
      return;
    }

    callback(data);
    this.callbacks.delete(messageId);
  }

  call(functionName, args = []) {
    const messageId = this.currentMessageId;
    this.currentMessageId += 1;

    if (this.currentMessageId > Number.MAX_SAFE_INTEGER) {
      // We assume we got all the previous callbacks
      // If not, something very wrong did happen
      this.currentMessageId = 0;
    }

    const data = {
      messageId,
      functionName,
      args,
    };

    return new Promise((resolve, reject) => {
      this.callbacks.set(messageId, ({ error, response }) => {
        if (error) {
          reject(error);
        } else {
          resolve(response);
        }
      });

      this.worker.postMessage(data, args.filter((arg) => arg instanceof ArrayBuffer));
    });
  }
}

const worker = new Worker('./rustWorker.js', { name: 'rustWorker', type: 'module' });
const promiseWorker = new PromiseWorker(worker);

export const norganceIdentifier = mem((identifier) => promiseWorker.call(
  'norgance_identifier',
  [identifier],
));

export const norganceCitizenSymmetricKey = mem(
  (identifier, password) => promiseWorker.call('norgance_citizen_symmetric_key',
    [identifier, password]), {
    cacheKey: JSON.stringify,
  },
);

export const norganceCitizenAccessKey = mem(
  (identifier, password) => promiseWorker.call('norgance_citizen_access_key',
    [identifier, password]), {
    cacheKey: JSON.stringify,
  },
);

export const norganceHibpPasswordHash = mem((password, size = 20) => promiseWorker.call('norgance_hibp_password_hash',
  [password, size]), {
  cacheKey: JSON.stringify,
});

require('./rustWorker');

export function chatrouillePackUnsignedQuery(payload, publicKey) {
  return promiseWorker.call('chatrouille_pack_unsigned_query',
    [payload, publicKey]);
}

export function chatrouilleUnpackResponse(packedData, sharedSecret) {
  return promiseWorker.call('chatrouille_unpack_response',
    [packedData, sharedSecret]);
}

export default promiseWorker;
