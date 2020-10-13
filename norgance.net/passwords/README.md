
## Why ?

We want our users to use good passwords. In our opinion, the best way to check whether a password is good is to see how many times it has leaked according to haveibeenpwned.com.

## Why not simply using the haveibeenpwned.com API ?

 * To learn and have fun with the rust programming language.
 * To not be dependant on a third-party api for a critical service like this.

## Keeping the password secrets

 * We use a similar strategy than haveibeenpwned.com
   * We search only the 5 first letters of the hexadecimal hash.
   * We return a list of at least 1024 corresponding suffixes.
   * If a suffix match, the password has been leaked and we can see how many times it did according to haveibeenpwned.com.
   * See https://en.wikipedia.org/wiki/K-anonymity
   * And https://haveibeenpwned.com/API/v3#SearchingPwnedPasswordsByRange
 * We overengineered it a bit for fun
   * Simply using sha1 hashes wasn't fun enough.
   * We hash every sha1 hash again with blake2b, and some specific static salt.
   * To continue using text files in a simple and fast way, the number of times a hash has leaked is a single character following a logarithmic scale. A 0 means a password has leaked very few times, F means way too many times (`123456` or `qwerty` for example).

## How to run it

 * Download the passwords sha1 datasets
   * https://haveibeenpwned.com/Passwords
 * Uncompress it
   * `7zr x pwned-passwords-sha1-ordered-by-hash-v5.7z`
 * Convert it to the norgance password hashing format
   * `./convertor pwned-passwords-sha1-ordered-by-hash-v5.txt norgance-passwords.csv`
 * Sort it
   * `sort -o norgance-passwords.csv norgance-passwords.csv`
 * Create the index
   * `./index_builder norgance-passwords.csv index.csv`
 * Remove the prefixes
   * `cat norgance-passwords.csv | cut -c 6- > hashes.csv`
   * I like useless uses of cats. 🐈💕
 * Start the server using docker
   * `docker run --name password_quality_server -d -p 3030:3030 -v $(pwd)/index.csv:/index.csv:ro -v $(pwd)/hashes.csv:/hashes
.csv:ro norgance/password_quality_server`