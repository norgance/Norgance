import Vue from 'vue';
import {
  NorganceAccessKey,
  NorganceVaultKey,
  NorganceRng,
  NorganceX25519DalekPrivateKey,
  NorganceEd25519DalekPrivateKey,
  NorganceVault,
} from '../rustyglue/classes';
import entropy from '../entropy';
import { anonymousGraphql } from '../chatrouille';

const defaultState = {
  started: false,
  accessKey: false,
  symmetricKey: false,
  asymmetricKeys: false,
  registering: false,
  done: false,
  error: false,
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
    done(state) {
      Object.assign(state, defaultState);
      Vue.set(state, 'done', true);
    },
  },

  actions: {
    async register({ commit, rootState }) {
      commit('reset');
      commit('progress', 'started');

      try {
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
        const vaultKey = await NorganceVaultKey.derive(identifier, password);

        commit('progress', 'asymmetricKeys');
        entropyInstance.ping();
        const rng = await NorganceRng.fromEntropy(entropyInstance);
        const x25519PrivateKey = await NorganceX25519DalekPrivateKey.fromRng(rng);
        const x25519PublicKey = await x25519PrivateKey.getPublicKey();
        const ed25519PrivateKey = await NorganceEd25519DalekPrivateKey.fromRng(rng);
        const ed25519PublicKey = await ed25519PrivateKey.getPublicKey();

        commit('progress', 'registering');

        const privateData = {
          identity,
          keys: {
            private_x25519_dalek: await x25519PrivateKey.toBase64(),
            private_ed25519_dalek: await ed25519PrivateKey.toBase64(),
          },
        };
        const aeadData = await NorganceVault.seal(vaultKey, JSON.stringify(privateData));

        const registration = {
          identifier: identifierHash,
          accessKey: await accessKey.getPublicKeyBase64(),
          publicX25519Dalek: await x25519PublicKey.toBase64(),
          publicEd25519Dalek: await ed25519PublicKey.toBase64(),
          aeadData,
        };

        entropyInstance.ping();
        console.log(identity, identifierHash, accessKey, registration);

        try {
          const toto = await anonymousGraphql({
            operationName: 'registerCitizenship',
            variables: {
              registration,
            },
            query: 'mutation registerCitizenship($registration: CitizenRegistration!) { registerCitizenship(registration: $registration) { success } }',
          });
          commit('done');
          console.log(toto);
        } finally {
          // Cleaning
          await Promise.all([
            rng.free(),
            // symmetricKey.free(),
            vaultKey.free(),
            accessKey.free(),
            x25519PrivateKey.free(),
            x25519PublicKey.free(),
            ed25519PrivateKey.free(),
            ed25519PublicKey.free(),
          ]);
        }
      } catch (error) {
        commit('progress', 'error');
        console.error(error);
      }
    },
  },
};
