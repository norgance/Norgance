import Vue from 'vue';
import {
  NorganceAccessKey,
  NorganceRng,
  NorganceX25519DalekPrivateKey,
  NorganceEd25519DalekPrivateKey,
} from '../rustyglue/classes';
import entropy from '../entropy';

const defaultState = {
  started: false,
  accessKey: false,
  symmetricKey: false,
  asymmetricKeys: false,
  done: false,
};

export default {
  namespaced: true,
  state: {
    ...defaultState,
  },
  mutations: {
    progress(state, step) {
      Vue.set(state, step, true);
    },
    reset(state) {
      Object.assign(state, defaultState);
    },
  },

  actions: {
    async register({ commit, rootState }) {
      commit('reset');
      commit('progress', 'started');

      const application = rootState.citizenApplication;
      const {
        identifier,
        identifierHash,
        password,
      } = application;

      const identity = {
        identifier,
        name: application.name,
        familyName: application.familyName || undefined,
        birthday: application.birthday || undefined,
        birthplace: application.birthplace || undefined,
      };

      const entropyInstance = entropy();
      commit('progress', 'accessKey');
      entropyInstance.ping();
      const accessKey = await NorganceAccessKey.derive(identifier, password);
      commit('progress', 'symmetricKey');
      entropyInstance.ping();
      // const symmetricKey = await norganceCitizenSymmetricKey(identifier, password);

      commit('progress', 'asymmetricKeys');
      entropyInstance.ping();
      const rng = await NorganceRng.fromEntropy(entropyInstance);
      const x25519PrivateKey = await NorganceX25519DalekPrivateKey.fromRng(rng);
      const x25519PublicKey = await x25519PrivateKey.getPublicKey();
      const ed25519PrivateKey = await NorganceEd25519DalekPrivateKey.fromRng(rng);
      const ed25519PublicKey = await ed25519PrivateKey.getPublicKey();

      const registration = {
        identifier: identifierHash,
        access_key: await accessKey.getPublicKeyBase64(),
        public_x25519_dalek: await x25519PublicKey.toBase64(),
        public_ed25519_dalek: await ed25519PublicKey.toBase64(),
        aead_data: 'abc',
      };

      await Promise.all([
        rng.free(),
        // symmetricKey.free(),
        accessKey.free(),
        x25519PrivateKey.free(),
        x25519PublicKey.free(),
        ed25519PrivateKey.free(),
        ed25519PublicKey.free(),
      ]);

      commit('progress', 'done');
      entropyInstance.ping();
      console.log(identity, identifierHash, accessKey, registration);
    },
  },
};
