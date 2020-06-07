# Architecture and Design

## Security and privacy

We want privacy by design thanks to cryptography.


### End 2 end encryption

Personal information will be encrypted by the clients. The backend, and therefore the nation, will not have access to the personal information. However some procedures involve to share information with others, which then may be encrypted or signed. We will make sure to have understable descriptions of what happens when you share data.

The encryption key for the user data will be derivated from its password and some salt. Most likely using argon2id with parameters taking about 1s on a normal computer in 2020.

### The password is important

The password will be only security for the users. We must make sure they are good enough for our usage.

Forgetting a password mean losing access to the account. By design. Therefore new citizens would be invited to make sure to never forget the password by writting it down on paper for example.

We may add two factors authentication too. Changing password would be possible at the condition to remember the previous password.

### Simple security is still good security

To improve security, the encrypted data will not be available to anyone that do not have a correct password.

The way is done is that we client will compute another hash of its password with specific salt, still using argon2id. This time it's simply a key to access to the encrypted data for the user.

If the database is accessed by attackers, this security will do nothing. But we will do everything to protect the data.

### Impossibility to guess documents

Document IDs will large enough, most likely UUIDv4 or something more user-friendly to write down, so it willn't be possible for people that do not have a contact with someone else to fetch his information.

### Trust and signatures

We will not use any kind of blockchain non-sense. However we way use Merkle trees where it may be neat to do so.

We will simply sign documents using asymetric cryptography. Probably using classic eliptic curves.

### JWT / JWE party

The final design may consist of many many many JWT and JWE documents embedded in each others. Be warned.


## Technologies

It's a web application.

### Database: PostGreSQL

The data is the most critical and important element of our infrastructure. We need a solid and reliable database with transactions. The amount of data will be relatively low and we don't need high availability or extreme parallelism. Therefore PostGreSQL is a solid choice in our opinion. Relationnal databases are also quite powerful to query things.

CouchDB as also been considered because it's neat, but its low performances and lack of transactions make us prefer PostGreSQL.

Graph databases are cool but we wanted something we knew already well enough.

### Dynamic server: Golang

We want to generate dynamic web pages server-side in the old fashioned way. The current choice is Golang because it offers a good standard library with for example template/html or net/http. The good performances regarding concurrency were also a decision point. For sure it's not the most exciting programming language but we need to use solid and boring technologies.

We could have gone full modern with React and Next.js or similar, and we may do so if we realize that our old fashioned golang approach is not enough.

### Frontend: Vanilla JS

We will try to do as much as possible Vanilla JS in the frontend. We may change our mind and go with Elm, Vue, or React.

### Datalayer : Hasura

We are not sure yet whether it will be exposed to the world-wide-web or not, but we may use Hasura as a datalayer, or an ORM, between our web pages and the PostGreSQL database.

### Session storage: Redis

We need to remember who is connected between web pages, a downside of generating pages server-side. Redis is a great in-memory database that do exactly what we need there and much more.

### Secrets and signing: Hashicorp Vault

We will need to save private keys of Norgance. Unfortunately it appears that it's not possible to export only the public key. We may have to write some tool to export public keys from the private keys export.

### End-2-end crypto

We can use WebCrypto or WebAssembly and some C, or WebAssembly and Rust. Or a mix of that.