/* eslint-disable no-underscore-dangle */
export default class RustClass {
  static className = 'RustClass';

  // Must be defined
  static promiseWorker = undefined;

  constructor(flatSource, promiseWorker) {
    Object.assign(this, flatSource);
    if (!this.ptr) {
      throw new Error(`Missing ptr for ${this.className}`);
    }
    this.promiseWorker = promiseWorker;
  }

  async _call(functionName, options) {
    return this.promiseWorker.call(functionName, {
      className: this.className,
      ptr: this.ptr,
      ...options,
    });
  }

  static async _callStatic(functionName, options) {
    return this.promiseWorker.call(functionName, {
      className: this.className,
      // Return a class instance by default
      returnClassName: this.className,
      staticFunction: true,
      ...options,
    });
  }

  async free() {
    return this._call('free');
  }
}
