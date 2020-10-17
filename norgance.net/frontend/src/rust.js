/* eslint-disable max-classes-per-file */
import mem from 'mem';
import entropy from './entropy';

class PromiseWorker {
  constructor(worker) {
    this.worker = worker;
    this.callbacks = new Map();
    this.classes = new Map();
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

  registerClass(classConstructor) {
    this.classes.set(classConstructor.name, classConstructor);
  }

  call(functionName, {
    args = [],
    transfer = undefined,
    preload = undefined,
    className = undefined,
  }) {
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
      preload,
      className,
    };

    return new Promise((resolve, reject) => {
      this.callbacks.set(messageId, ({ error, response }) => {
        if (error) {
          reject(error);
          return;
        }

        if (response.ptr && response.className) {
          const ClassConstructor = this.classes.get(response.className);
          if (!ClassConstructor) {
            reject(new Error(`Unknown class ${response.className}`));
            return;
          }
          resolve(new ClassConstructor(response));
          return;
        }

        resolve(response);
      });

      this.worker.postMessage(data, transfer);
    });
  }
}

const worker = new Worker('./rustWorker.js', { name: 'rustWorker', type: 'module' });
const promiseWorker = new PromiseWorker(worker);

class RustClass {
  constructor(flatSource) {
    this.className = this.constructor.name;
    Object.assign(this, flatSource);
    if (!this.ptr) {
      throw new Error(`Missing ptr for ${this.className}`);
    }
  }

  async free() {
    return this.promiseWorker('free', {
      className: this.className,
    });
  }
}

export class ChatrouilleUnsignedQuery extends RustClass {}
promiseWorker.registerClass(ChatrouilleUnsignedQuery);

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
  return promiseWorker.call('make_norgance_rng', { args: [entropyData] });
}

export function genX448PrivateKey(rng) {
  return promiseWorker.call('gen_x448_private_key', { args: [rng] });
}
export default promiseWorker;
