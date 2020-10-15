import { norganceIdentifier } from '../rust';
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
      const response = await anonymousQuery({
        query: `query isIdentifierAvailable($identifier: String!) {
          isIdentifierAvailable(identifier: $identifier)
        }`,
        variables: {
          identifier: state.identifierHash,
        },
      });
      commit('updateIdentifierAvailability', response.isIdentifierAvailable);
    },
  },
};
