/* eslint-disable no-underscore-dangle */
export default class RustClass {
  static className = 'RustClass';

  constructor(flatSource, promiseWorker) {
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
