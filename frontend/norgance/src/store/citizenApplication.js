export default {
  namespaced: true,
  state: {
    name: '',
    familyName: '',
    birthday: undefined,
    birthplace: '',
    identifier: '',
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
  },
};
