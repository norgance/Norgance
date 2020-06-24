/** This code is not minified, so you can read it easily. */
(function main() {

  var COOKIES_OK = 'cookies-ok';
  var COOKIES_LANG = 'cookies-lang';

  var userAcceptsCookies = window.localStorage &&
    !!window.localStorage.getItem(COOKIES_OK);
  var sessionStore = userAcceptsCookies ?
    window.localStorage : window.sessionStorage;

  /** This function adds some interactivity to the cookie disclaimer such as showing the accept button and hiding the disclaimer once cookies are accepted. */
  function cookies() {

    var section = document.querySelector('section.cookies');
    if (!window.localStorage || !window.localStorage.getItem(COOKIES_OK)) {
      section.style.display = 'flex';
    }
    var buttons = section.querySelector('.buttons');
    buttons.style.display = 'block';

    var acceptButton = buttons.querySelector('.button-accept');
    acceptButton.addEventListener('click', function acceptCookies(event) {
      section.style.display = 'none';
      event.preventDefault();
      if (window.localStorage) {
        window.localStorage.setItem(COOKIES_OK, 'true');

        // Copy the lang cookie from the temporary store when the cookies usage
        // is accepted in case the user changes language before she accepts
        // the use of cookies.
        if (window.sessionStorage && window.sessionStorage.getItem(COOKIES_LANG)) {
          window.localStorage.setItem(COOKIES_LANG, window.sessionStorage.getItem(COOKIES_LANG));
          window.sessionStorage.removeItem(COOKIES_LANG);
        }

        sessionStore = window.localStorage;
      }
    });
  }

  /** This function redirects the user from the default english frontpage to his favourite language frontpage */
  function languageRedirect() {


    var languageSelector = document.querySelector('.language-selector');
    var languageLinks = languageSelector.querySelectorAll('a');
    var languageLinksMap = {};

    for (var i = 0, l = languageLinks.length; i < l; ++i) {
      var link = languageLinks[i];
      link.addEventListener('click', function clickLanguage() {
        sessionStore.setItem(COOKIES_LANG, this.getAttribute('lang'));
      });
      const lang = link.getAttribute('lang').match(/^[a-z]{2}/)[0];
      languageLinksMap[lang] = link;
    }

    // Only redirect from the frontpage
    if (window.location.pathname !== '/') {
      return;
    }

    // Do not redirect if visiting between pages
    // It also make it work when the user doesn't have cookies
    if (document.referrer.indexOf(location.origin) === 0) {
      return;
    }


    var defaultLanguage = document.body.parentNode.lang || 'en';
    var userLanguage = (navigator.language.match(/^[a-z]{2}/) || ['en'])[0].toLocaleLowerCase();

    // If the user has already selected a language, redirect to it 
    if (sessionStore && sessionStore.getItem(COOKIES_LANG)) {
      userLanguage = sessionStore.getItem(COOKIES_LANG).toLocaleLowerCase();
    }

    // Do not redirect the frontpage to itself when the language is the same
    // indexOf === 0 is an alternative to startsWith for old browsers
    if (userLanguage.indexOf(defaultLanguage) === 0) {
      return;
    }

    if (Object.prototype.hasOwnProperty.call(languageLinksMap, userLanguage)) {
      languageLinksMap[userLanguage].click();
      return;
    }

    // Iterate through the user languages to find the best match
    var userLanguages = navigator.languages;
    for (var i = 0, l = userLanguages.length; i < l; ++i) {
      var lang = (userLanguages[i].match(/^[a-z]{2}/) || ['en'])[0];
      if (lang !== 'en') {
        if (Object.prototype.hasOwnProperty.call(languageLinksMap, lang)) {
          // If it's a match, click the language link
          languageLinksMap[lang].click();
          return;
        }
      }
    }

  }

  try {
    cookies();
    languageRedirect();
  } catch (error) {
    if (console && console.error) {
      console.error(error);
    }
  }
})();