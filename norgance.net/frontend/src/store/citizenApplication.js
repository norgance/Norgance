import { norganceIdentifier, norganceHibpPasswordHash } from '../rust';
import { anonymousQuery } from '../chatrouille';

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

    async registerCitizenship({ commit }) {
      commit('reset');
    },

    async checkIdentifierAvailability({ commit, state }) {
      if (!state.identifierHash) {
        throw new Error('Identifier hash must be computed first');
      }
      const isIdentifierAvailable = await anonymousQuery({
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

      const checkPasswordQuality = await anonymousQuery({
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
