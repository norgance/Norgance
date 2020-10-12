---
title: "PGP"
date: 2020-06-16T09:26:46+02:00
draft: false
---

Web pages on [norgance.com](https://norgance.com) are digitally signed with [PGP](https://en.wikipedia.org/wiki/Pretty_Good_Privacy).

Signing a webpage with PGP allows you to trust that the web page  you are reading has been written by its author, and not modified by anyone else.

The remaining section of this document will be technical. However, you don't need to read it. The standard security of your device is good enough and PGP is used in addition as a gimmick for IT people who likes cryptography. If you still want to verify the digital signatures, you can ask someone you trust and competent in this domain to do it with you.

## Why PGP

### How to trust a website ?

The secure web, `HTTPS`, is based on a [chain of trust](https://en.wikipedia.org/wiki/Chain_of_trust), so everyone can have a relatively good security and privacy while browsing the web, without a too cumbersome experience.

In a perfect world, the user trust her device, the device trust [certificate authorities](https://en.wikipedia.org/wiki/Certificate_authority). Software and hardware provider select the certificate authorities according to strict guidelines and this works well in practice.

However it happens relatively often that the something or someone breaks the chain of trust by adding new dodgy certificate authorities inside a device. Many corporations do that on their employees' devices without telling them. Some countries also do it, and some antivirus as well.

In this case, the consequences are no security and privacy. The content of a website can also be modified.

### Failed attempt for the problem

[HTTP Public Key Pinning](https://en.wikipedia.org/wiki/HTTP_Public_Key_Pinning) was a solution to this problem with some flaws, that was never used widely. It was interesting but a bit dangerous to use for everyone. It's not supported by modern devices and sofware anymore.

[Certificate Transparency](https://en.wikipedia.org/wiki/Certificate_Transparency) is a technical solution to address the issue of a certificate authority issuing malicious certificates without knowing it. Thanks to [Merkle trees](https://en.wikipedia.org/wiki/Merkle_tree) (in short Merktle trees are the good part of a blockchain without most all the issues a blockchain have), it's impossible for a certificate authority to issue certificates without knowing it. While this solution is a good thing to have, spies can't hack Google's certificate authority to spy on Google's websites using valid Google certificates without being unnoticed, it doesn't solve much. Most certificate authorities do not implement it and people are simply creating dodgy certificate authorities. A normal user never checks the certification paths of the websites she browses.

### PGP until something better

As you understand, there is no perfect solution today. The current solution works well for most users, and we use PGP to fill the gap with the users who want a bit more and who are willing to spend time and energy into their web browsing.

## How to validate the signature ?

The Norgance public key ID is `EF5DE21DE501CDE1` and it is hosted on `hkps://keyserver.ubuntu.com`.

You can use the following commands to verify the digital signature of a Norgance webpage.

```sh
gpg --keyserver hkps://keyserver.ubuntu.com --receive-keys EF5DE21DE501CDE1
curl https://norgance.com/ | gpg --verify
curl https://norgance.com/pgp/ | gpg --verify
```

In addition to this, a webpage should look like the following example.
It must not be anything before the text `<!-- -----BEGIN PGP SIGNED MESSAGE` and after the text `-----END PGP SIGNATURE----- -->` as these parts are not signed.

```html
<!--
-----BEGIN PGP SIGNED MESSAGE-----
Hash: SHA256

https://norgance.com/pgp/ -->
<!doctype html><html lang=fr>...</html><!--
-----BEGIN PGP SIGNATURE-----

iQIzBAEBCAAdFiEE2I/DOTrpAyYCNKjAHs+zwAgqgMkFAl7102oACgkQHs+zwAgq
...
Jwcoaak1lGotsCAOYpfCgfizo6CN7B+w/7S+MfshWV5SaTpAT+Q==MBym
-----END PGP SIGNATURE-----
-->
```
