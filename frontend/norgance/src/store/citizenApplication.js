export default {
  namespaced: true,
  state: {
    firstName: '',
    familyName: '',
    birthday: undefined,
    birthplace: '',
    identifier: '',
  },
  mutations: {
    updateFirstName(state, name) {
      state.firstName = name;
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
  },
};
