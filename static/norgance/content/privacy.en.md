---
title: "Privacy"
date: 2020-06-16T09:26:46+02:00
draft: false
---

### Privacy by design

Protecting your privacy is extremely important and Norgance is designed to protect it by design. The data is kept secret and encrypted with keys and passwords only you know, and the data is organized in a way that only you know where to access it.

Some information must be shared to others, such as your spouse when you want to marry, but only you decide what to share, when, and to whom.

[The can read more technical details about the privacy by design.]({{<ref "how-does-it-work">}}). You can contact an expert in privacy and cryptography you trust to verify the specifications.

### Cookies

[Norgance.com](https://norgance.com) and [Norgance.net](https://norgance.net) use a technical cookie named `_cfduid`, from our technical provider [Cloudflare](https://www.cloudflare.com/).

[As explained by Cloudflare](https://support.cloudflare.com/hc/en-us/articles/200170156-What-does-the-Cloudflare-cfduid-cookie-do-#12345682), this technical cookie is used to identify individual visitors, to block malicious visitors while not bothering friendly visitors. It expires after 30 days and it doesn't allow neither Norgance or Cloudflare to track you accross websites or to link your web activity on Norgance with your identity.

### Privacy policy

#### Data collection

We collect anonymous traffic metrics through our Cloudflare technical provider. [You may want to read their full privacy policy.](https://www.cloudflare.com/privacypolicy/).

### Data integrity

[All webpages are signed using PGP](http://localhost:1313/pgp/). This technic has limitations, it's not automatic nor user friendly, but it allows you to verify that no one has modified the webpage you are reading or changed the source code of Norgance.

### Encrypted data exchanges

Data exchanges on [Norgance.net](https://norgance.net) are encrypted on top of the already encrypted connection. HTTPS connection are not encrypted end-2-end, especially when you use a CDN like Cloudflare or in corporate environments with HTTPS proxies, so we added a new layer of encryption. Which isn't perfect because you can't verify data integrity automatically, but it's much better than most of the websites.