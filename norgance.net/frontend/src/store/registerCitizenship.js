import Vue from 'vue';
import { norganceCitizenAccessKey, norganceCitizenSymmetricKey } from '../rustyglue';
import {
  NorganceRng,
  NorganceX25519DalekPrivateKey,
  NorganceX448PrivateKey,
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
      const accessKey = await norganceCitizenAccessKey(identifier, password);
      commit('progress', 'symmetricKey');
      entropyInstance.ping();
      const symmetricKey = await norganceCitizenSymmetricKey(identifier, password);

      commit('progress', 'asymmetricKeys');
      entropyInstance.ping();
      const rng = await NorganceRng.fromEntropy(entropyInstance);
      const x448PrivateKey = await NorganceX448PrivateKey.fromRng(rng);
      const x448PublicKey = await x448PrivateKey.getPublicKey();
      const x25519PrivateKey = await NorganceX25519DalekPrivateKey.fromRng(rng);
      const x25519PublicKey = await x25519PrivateKey.getPublicKey();
      const ed25519PrivateKey = await NorganceEd25519DalekPrivateKey.fromRng(rng);
      const ed25519PublicKey = await ed25519PrivateKey.getPublicKey();

      const registration = {
        publicX448: await x448PublicKey.toBase64(),
        publicX25519Dalek: await x25519PublicKey.toBase64(),
        publicEd25519Dalek: await ed25519PublicKey.toBase64(),
      };

      await Promise.all([
        rng.free(),
        x448PrivateKey.free(),
        x448PublicKey.free(),
        x25519PrivateKey.free(),
        x25519PublicKey.free(),
        ed25519PrivateKey.free(),
        ed25519PublicKey.free(),
      ]);

      commit('progress', 'done');
      entropyInstance.ping();
      console.log(identity, identifierHash, symmetricKey, accessKey, x448PrivateKey, registration);
    },
  },
};
