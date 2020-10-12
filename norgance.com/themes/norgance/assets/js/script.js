/** This code is not minified, so you can read it easily. */
(function main() {

  /** This function redirects the user from the default english frontpage to his favourite language frontpage */
  function languageRedirect() {

    var languageSelector = document.querySelector('.language-selector');
    var languageLinks = languageSelector.querySelectorAll('a');
    var languageLinksMap = {};

    for (var i = 0, l = languageLinks.length; i < l; ++i) {
      var link = languageLinks[i];
      const lang = link.getAttribute('lang').match(/^[a-z]{2}/)[0];
      languageLinksMap[lang] = link;
    }

    // Only redirect from the frontpage
    if (window.location.pathname !== '/') {
      return;
    }

    // Do not redirect if visiting between pages
    // It also make it work when the user doesn't have cookies
    if (!document.referrer || document.referrer.indexOf(location.origin) === 0) {
      return;
    }


    var defaultLanguage = document.body.parentNode.lang || 'en';
    var userLanguage = (navigator.language.match(/^[a-z]{2}/) || ['en'])[0].toLocaleLowerCase();

    // Do not redirect the frontpage to itself when the language is the same
    // indexOf === 0 is an alternative to startsWith for old browsers
    if (userLanguage.indexOf(defaultLanguage) === 0) {
      return;
    }

    // Iterate through the user languages to find the best match
    var userLanguages = navigator.languages || [userLanguage];
    for (var i = 0, l = userLanguages.length; i < l; ++i) {
      var lang = (userLanguages[i].match(/^[a-z]{2}/) || ['en'])[0].toLocaleLowerCase();
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
    languageRedirect();
  } catch (error) {
    if (console && console.error) {
      console.error(error);
    }
  }
})();