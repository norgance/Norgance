/* eslint-disable no-underscore-dangle */
export default class RustClass {
  // The classnames are removed by the minifier
  // So we explicitly specify it.
  // We could also have a whitelist in the minifier
  // Or disabling removing classnames.
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
    if (!this.ptr) {
      throw new Error(`Invalid ptr for ${this.className}`);
    }
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
    const promise = this._call('free');
    // Immediately set ptr to 0 to prevent accidental multiple free
    this.ptr = 0;
    await promise;
  }
}
