import Vue from 'vue';
import { norganceCitizenAccessKey, norganceCitizenSymmetricKey } from '../rustyglue';
import { NorganceRng, NorganceX448PrivateKey } from '../rustyglue/classes';
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
  },

  actions: {
    async register({ commit, rootState }) {
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

      commit('progress', 'accessKey');
      const accessKey = await norganceCitizenAccessKey(identifier, password);
      commit('progress', 'symmetricKey');
      const symmetricKey = await norganceCitizenSymmetricKey(identifier, password);

      commit('progress', 'asymmetricKeys');
      const rng = await NorganceRng.fromEntropy(entropy());
      const x448PrivateKey = await NorganceX448PrivateKey.fromRng(rng);
      const x448PublicKey = await x448PrivateKey.getPublicKey();

      const registration = {
        publicX448: await x448PublicKey.toBase64(),
      };

      rng.free();
      x448PrivateKey.free();
      x448PublicKey.free();

      commit('progress', 'done');
      console.log(identity, identifierHash, symmetricKey, accessKey, x448PrivateKey, registration);
    },
  },
};
