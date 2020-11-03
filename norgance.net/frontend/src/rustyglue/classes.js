/* eslint-disable no-underscore-dangle */
/* eslint-disable max-classes-per-file */
import RustClass from './rustClass';
import { Chatrouille, ChatrouilleQuery } from './rustyChatrouille';

export { Chatrouille, ChatrouilleQuery };

export class NorganceAccessKey extends RustClass {
  static className = 'NorganceAccessKey';

  static async derive(identifier, password) {
    return this._callStatic('derive', {
      args: [identifier, password],
    });
  }

  async getPublicKeyBase64() {
    return this._call('get_public_key_base64');
  }
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

export class NorganceX25519DalekPrivateKey extends RustClass {
  static className = 'NorganceX25519DalekPrivateKey';

  static async fromRng(rng) {
    return this._callStatic('from_rng', {
      args: [rng],
    });
  }

  static async fromBase64(privateKeyBase64) {
    return this._callStatic('from_base64', {
      args: [privateKeyBase64],
    });
  }

  async toBase64() {
    return this._call('to_base64');
  }

  async getPublicKey() {
    return this._call('get_public_key', {
      returnClassName: 'NorganceX25519DalekPublicKey',
    });
  }
}

export class NorganceX25519DalekPublicKey extends RustClass {
  static className = 'NorganceX25519DalekPublicKey';

  static async fromBase64(publicKeyBase64) {
    return this._callStatic('from_base64', {
      args: [publicKeyBase64],
    });
  }

  async toBase64() {
    return this._call('to_base64');
  }
}

export class NorganceEd25519DalekPrivateKey extends RustClass {
  static className = 'NorganceEd25519DalekPrivateKey';

  static async fromRng(rng) {
    return this._callStatic('from_rng', {
      args: [rng],
    });
  }

  static async fromBase64(privateKeyBase64) {
    return this._callStatic('from_base64', {
      args: [privateKeyBase64],
    });
  }

  async toBase64() {
    return this._call('to_base64');
  }

  async getPublicKey() {
    return this._call('get_public_key', {
      returnClassName: 'NorganceEd25519DalekPublicKey',
    });
  }
}

export class NorganceEd25519DalekPublicKey extends RustClass {
  static className = 'NorganceEd25519DalekPublicKey';

  static async fromBase64(publicKeyBase64) {
    return this._callStatic('from_base64', {
      args: [publicKeyBase64],
    });
  }

  async toBase64() {
    return this._call('to_base64');
  }
}
