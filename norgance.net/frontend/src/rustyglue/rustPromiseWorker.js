export default class RustPromiseWorker {
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
    freeResponseImmediately = false,
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
      freeResponseImmediately,
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
          resolve(new ClassConstructor(response, this));
          return;
        }

        resolve(response);
      });

      this.worker.postMessage(data, transfer);
    });
  }
}