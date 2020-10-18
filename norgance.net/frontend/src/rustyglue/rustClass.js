/* eslint-disable no-underscore-dangle */
export default class RustClass {
  static className = 'RustClass';

  // Must be defined
  static promiseWorker = undefined;

  constructor(flatSource) {
    Object.assign(this, flatSource);
    if (!this.ptr) {
      throw new Error(`Missing ptr for ${this.className}`);
    }
  }

  static async _call(functionName, options) {
    return this.promiseWorker.call(functionName, {
      className: this.className,
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
