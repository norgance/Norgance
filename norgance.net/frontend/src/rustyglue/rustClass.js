export default class RustClass {
  constructor(flatSource, promiseWorker) {
    this.className = this.constructor.name;
    this.promiseWorker = promiseWorker;
    Object.assign(this, flatSource);
    if (!this.ptr) {
      throw new Error(`Missing ptr for ${this.className}`);
    }
  }

  async _call(functionName, options) {
    return this.promiseWorker(functionName, {
      className: this.className,
      ...options,
    });
  }

  async free() {
    return this._call('free');
  }
}