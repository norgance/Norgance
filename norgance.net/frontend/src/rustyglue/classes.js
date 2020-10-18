/* eslint-disable no-underscore-dangle */
/* eslint-disable max-classes-per-file */
import RustClass from './rustClass';

export class ChatrouilleUnsignedQuery extends RustClass {
  // The classnames are removed by the minifier
  // So we explicitly specify it.
  // We could also have a whitelist in the minifier
  // Or disabling removing classnames.
  static className = 'ChatrouilleUnsignedQuery';
}

export class NorganceRng extends RustClass {
  static className = 'NorganceRng';

  static async fromEntropy(entropy) {
    return this._callStatic('from_entropy', {
      args: [entropy.data],
      // We don't transfer the data, we copy it
      transfer: [],
    });
  }
}

export class NorganceX448PrivateKey extends RustClass {
  static className = 'NorganceX448PrivateKey';

  static async fromBase64(privateKeyBase64) {
    return this._callStatic('from_base64', {
      args: [privateKeyBase64],
    });
  }

  static async fromRng(rng) {
    return this._callStatic('from_rng', {
      args: [rng],
    });
  }

  async toBase64() {
    return this._call('to_base64');
  }
}
