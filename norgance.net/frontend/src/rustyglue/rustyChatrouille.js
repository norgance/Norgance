/* eslint-disable no-underscore-dangle */
/* eslint-disable max-classes-per-file */
import RustClass from './rustClass';

export class ChatrouilleQuery extends RustClass {
  static className = 'ChatrouilleQuery';
}

export class Chatrouille extends RustClass {
  static className = 'Chatrouille';

  static async withPublicKeyBase64(serverPublicKey) {
    return this._callStatic('with_public_key_base64', {
      args: [serverPublicKey],
    });
  }

  static async withPublicKey(serverPublicKey) {
    return this._callStatic('with_public_key', {
      args: [serverPublicKey],
      transfer: [serverPublicKey.buffer],
    });
  }

  async setClientAccessKey(accessKey) {
    return this._call('set_client_access_key', {
      args: [accessKey],
    });
  }

  async packUnsignedQuery(payload) {
    return this._call('pack_unsigned_query', {
      args: [payload],
      preload: {
        query: { functionName: 'get_query' },
      },
      returnClassName: 'ChatrouilleQuery',
    });
  }

  async packSignedQuery(payload) {
    return this._call('pack_signed_query', {
      args: [payload],
      preload: {
        query: { functionName: 'get_query' },
      },
      returnClassName: 'ChatrouilleQuery',
    });
  }

  static async unpackResponse(packedData, query) {
    return this._callStatic('unpack_response', {
      args: [packedData, query],
      transfer: [packedData.buffer],
    });
  }
}
