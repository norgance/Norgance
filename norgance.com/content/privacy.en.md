---
title: "Privacy"
date: 2020-06-16T09:26:46+02:00
draft: false
---

### Privacy by design

Protecting your privacy is extremely important and Norgance is designed to protect it by design. The data is kept secret and encrypted with keys and passwords only you know, and the data is organized in a way that only you know where to access it.

Some information must be shared to others, such as your spouse when you want to marry, but only you decide what to share, when, and to whom.

Norgance respects your privacy and has no interest into 

The source code is available for anyone to audit. This may require some technical expertise, but you can ask an expert in privacy and cryptography to do it.

### Cookies

[Norgance.com](https://norgance.com) and [Norgance.net](https://norgance.net) use a technical cookie named `_cfduid`, from our technical provider [Cloudflare](https://www.cloudflare.com/).

[As explained by Cloudflare](https://support.cloudflare.com/hc/en-us/articles/200170156-What-does-the-Cloudflare-cfduid-cookie-do-#12345682), this technical cookie is used to identify individual visitors, to block malicious visitors while not bothering friendly visitors. It expires after 30 days and it doesn't allow neither Norgance or Cloudflare to track you accross websites or to link your web activity on Norgance with your identity.

### Data collection

We collect anonymous traffic metrics through our Cloudflare technical provider. [You may want to read their full privacy policy.](https://www.cloudflare.com/privacypolicy/).

### Data storage

Your personal data is encrypted on your device and stored by Norgance without any solution to decrypt it. The encrypted data and its backups are physically stored in Europe.

------------

### Technical informations about our privacy by design

#### End to end encryption

The data is encrypted on your device and decrypted on your device. Nothing is stored on your device. The keys are derivated from your citizenname and your password. A high quality password is strongly encouraged.

#### The algorithms Norgance uses.

The data is encrypted using `xchacha20poly1305`. Random nounce are always used.
The key derivation algorithm is `argon2id` and the hashing algorithm is `blake2b`. Salt is always used.
The assymetric cryptography algorithms are elliptic curve algorithms, namely `ed25519`, `x25519`, and `x448`.
The cryptographically secure pseudo-random number generator (CPRNG) running on your device uses entropy from your web-browser and your  interactions on norgance.net, with the `chacha20` algorithm.

#### Content delivery network

We use a content delivery network, currently Cloudflare, to protect the technical infrastructure running Norgance. Having a content delivery network between the Norgance infrastructure and your browser is not perfect because it adds one intermediate, but we think your privacy is better protected if the Norgance infrastructure is not directly exposed to Internet. We also implemented two innovatives techniques to sign and encrypt data transfers on top of the classic web technologies.

#### Data integrity

[All webpages are signed using PGP](/pgp/). Thanks to the signatures, you can be sure that no one has modified the containt and that the main maintainers of Norgance published the last updates. However, this approach has limitations because the signatures are not automatically verified and PGP is not very user friendly. But it allows some technical users to verify that no one has modified the webpages they are reading or changed the source code of Norgance. It's better than nothing.

#### Double encrypted data exchanges

Data exchanges on [Norgance.net](https://norgance.net) are encrypted on top of the already encrypted HTTPS connection. HTTPS connections are supposedly very safe and encrypted end to end, but in practice it's not always the case. Especially when you use a content delivery network or in corporate environments with HTTPS proxies and corporate root certificates. So we added a new layer of encryption on top of the encryption. Please note that the new layer of encryption cannot verify data integrity automatically with this approach, you rely on HTTPS and [PGP](/pgp/).

