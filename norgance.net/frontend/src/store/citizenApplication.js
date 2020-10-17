import { norganceIdentifier, norganceHibpPasswordHash } from '../rustyglue';
import { anonymousGraphql } from '../chatrouille';

const defaultState = {
  name: '',
  familyName: '',
  birthday: undefined,
  birthplace: '',
  identifier: '',
  identifierHash: '',
  identifierIsAvailable: true,
  password: '',
};

export default {
  namespaced: true,
  state: {
    ...defaultState,
  },
  mutations: {
    updateName(state, name) {
      state.name = name;
    },
    updateFamilyName(state, name) {
      state.familyName = name;
    },
    updateBirthday(state, birthday) {
      if (!(birthday instanceof Date)) {
        throw new Error('birthday must be a date');
      }
      state.birthday = birthday;
    },
    updateBirthPlace(state, birthplace) {
      state.birthplace = birthplace;
    },
    updateIdentifier(state, identifier) {
      state.identifier = identifier;
    },
    updateIdentifierHash(state, identifierHash) {
      state.identifierHash = identifierHash;
    },
    updateIdentifierAvailability(state, availability) {
      state.identifierIsAvailable = !!availability;
    },
    updatePassword(state, password) {
      state.password = password;
    },
    reset(state) {
      Object.assign(state, defaultState);
    },
  },
  actions: {
    async setIdentifier({ commit }, identifier) {
      commit('updateIdentifier', identifier);
      const hash = await norganceIdentifier(identifier);
      console.log(hash);
      commit('updateIdentifierHash', hash);
    },

    async registerCitizenship({ state }) {
      const identity = {
        identifier: state.identifier,
        name: state.name,
      };
      if (state.familyName) {
        identity.familyName = state.familyName;
      }
      if (state.birthday) {
        identity.birthday = state.birthday;
      }
      if (state.birthplace) {
        identity.birthplace = state.birthplace;
      }

      const privateKeys = {
        x448: 'canard',
        x25519Dalek: 'canard',
        ed25519Dalek: 'canard',
      };

      const privateData = {
        identity,
        privateKeys,
      };

      // TODO hein
      const accessKey = state.identifierHash.toLocaleLowerCase();
      const aeadData = btoa(JSON.stringify(privateData)).replace('=', '');

      console.log(aeadData);
      const result = await anonymousGraphql({
        query: `mutation registerCitizenship($registration: CitizenRegistration!) {
          registerCitizenship(registration: $registration) {
            success
          }
        }`,
        variables: {
          registration: {
            identifier: state.identifierHash,
            accessKey,
            publicX448: 'a',
            publicX25519Dalek: 'b',
            publicEd25519Dalek: 'c',
            aeadData,
          },
        },
      });
      console.log(result);
      // commit('reset');
    },

    async checkIdentifierAvailability({ commit, state }) {
      if (!state.identifierHash) {
        throw new Error('Identifier hash must be computed first');
      }
      const isIdentifierAvailable = await anonymousGraphql({
        query: `query isIdentifierAvailable($identifier: String!) {
          isIdentifierAvailable(identifier: $identifier)
        }`,
        variables: {
          identifier: state.identifierHash,
        },
      });
      commit('updateIdentifierAvailability', isIdentifierAvailable);
    },

    async checkPasswordQuality({ state }) {
      const hash = await norganceHibpPasswordHash(state.password);
      const prefix = hash.slice(0, 5);
      const suffix = hash.slice(5);
      console.log(hash);

      const checkPasswordQuality = await anonymousGraphql({
        query: `query checkPasswordQuality($prefix: String!) {
          checkPasswordQuality(prefix: $prefix) {
            suffix quality
          }
        }`,
        variables: {
          prefix,
        },
      });

      const quality = checkPasswordQuality.find((password) => password.suffix === suffix);

      return quality ? quality.quality : 'good';
    },
  },
};
