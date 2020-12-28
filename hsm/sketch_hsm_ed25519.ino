#include <base64.hpp>
#include <ChaCha.h>
#include <Ed25519.h>

// Should be a random number between 0 and 1023
#define DYNAMIC_DELAY_SECRET 123

static const int analogInPin = A1;
static int sensorValue = 0;
static ChaCha chacha;
static char serialInputBuffer[256] = {0};
static uint8_t documentBuffer[256] = {0};
uint8_t privateKey[32] = {0};
uint8_t publicKey[32] = {0};

static void collectEntropyBits(uint8_t buffer[], const uint16_t size) {
  unsigned int timeToWait = 0;
  uint8_t a = 0;
  uint8_t b = 0;

  for (uint16_t i = 0; i < size * 8; ++i) {
    do {
      a = analogRead(analogInPin);
      timeToWait = map(~(a ^ DYNAMIC_DELAY_SECRET), 0, 1023, 20, 50);
      delayMicroseconds(timeToWait);
      b = analogRead(analogInPin);
      timeToWait = map(~(b ^ DYNAMIC_DELAY_SECRET), 0, 1023, 500, 4000);
      delayMicroseconds(timeToWait);
    } while (a == b);

    if (a & 1) {
      buffer[i / 8] |= 1 << (i % 8);
    } else {
      buffer[i / 8] &= ~(1 << (i % 8));
    }
  }
}

static void setupRND(uint8_t entropy[32]) {

  uint8_t entropyStarter[32] = {0};
  if (entropy != NULL) {
    memcpy(entropyStarter, entropy, 32);
  } else {
    collectEntropyBits(entropyStarter, 32);
  }

  chacha.setNumRounds(20);

  uint8_t iv[8] = {0};
  collectEntropyBits(iv, 8);
  chacha.setIV(iv, 8);

  uint8_t key[32] = {0};
  collectEntropyBits(key, 32);
  chacha.setKey(key, 32);

  // We compute a new Chacha Key using the entropy
  uint8_t newKey[32] = {0};
  chacha.encrypt(newKey, entropyStarter, 32);
  chacha.setKey(newKey, 32);
}

static void shuffleRND() {
  uint8_t newEntropy[32] = {0};
  collectEntropyBits(newEntropy, 32);
  uint8_t newKey[32] = {0};
  chacha.encrypt(newKey, newEntropy, 32);
  chacha.setKey(newKey, 32);
}

static void collectRND(uint8_t output[], const uint16_t size, bool encrypt = true) {

  uint16_t remainingBits = size;
  uint8_t *outputBatchPointer = output;
  uint8_t buffer[256];

  while (remainingBits > 0) {
    uint16_t sizeCurrentBatch = min(256, remainingBits);
    collectEntropyBits(buffer, sizeCurrentBatch);

    if (encrypt) {
      chacha.encrypt(outputBatchPointer, buffer, sizeCurrentBatch);
    } else {
      memcpy(outputBatchPointer, buffer, sizeCurrentBatch);
    }

    remainingBits -= sizeCurrentBatch;
    outputBatchPointer += sizeCurrentBatch;
  }
}

static unsigned int safer_decode_base64(unsigned char* input, uint8_t* output, unsigned int maxLength) {
  unsigned int computedLength = decode_base64_length(input);
  if (computedLength > maxLength) {
    return 0;
  }
  return decode_base64(input, output);
}


static void setPrivateKey() {
  Ed25519::derivePublicKey((uint8_t*)publicKey, (const uint8_t*)privateKey);
  Serial.print("PUBLIC_KEY ");
  memset(documentBuffer, 0, 256);
  encode_base64(publicKey, 32, documentBuffer);
  Serial.println((char*) documentBuffer);
  memset(documentBuffer, 0, 256);
}

static void randomPrivateKey() {
  collectRND(privateKey, 32);
  setPrivateKey();
}


static void sign(unsigned int documentSize) {
  uint8_t signature[64] = {0};
  Ed25519::sign(signature, privateKey, publicKey, documentBuffer, documentSize);
  memset(documentBuffer, 0, 256);
  encode_base64(signature, 64, documentBuffer);
  Serial.println((char*) documentBuffer);
  memset(documentBuffer, 0, 256);
}

void setup() {
  Serial.begin(9600);
  while (!Serial) {}

  Serial.println("BOOTING");
  setupRND(NULL);
  randomPrivateKey();
  Serial.println("READY");
}

void loop() {
  memset(serialInputBuffer, 0, 255);
  if (Serial.readBytesUntil('\n', serialInputBuffer, 255) > 0) {
    char* command = strtok(serialInputBuffer, " \t");

    if (command == NULL) return;

    // Make command uppercase
    for (char* c = command; *c; ++c) {
      *c = (char) toupper(*c);
    }

    if (strcmp("CANARD", command) == 0) {
      Serial.println("KOINKOIN");
    } else if (strcmp("SIGN", command) == 0) {
      char* documentBase64 = strtok(NULL, " \t");
      unsigned int documentSize = safer_decode_base64((unsigned char*) documentBase64, documentBuffer, 256);
      sign(documentSize);
    } else if (strcmp("SHUFFLE_RND", command) == 0) {
      shuffleRND();
      Serial.println("OK");

    } else if (strcmp("RANDOM_PRIVATE_KEY", command) == 0) {
      randomPrivateKey();
    }
#ifdef DEVELOPMENT_MODE
    else if (strcmp("SETUP_RND", command) == 0) {
      char* entropyBase64 = strtok(NULL, " \t");
      size_t entropyLength = strlen(entropyBase64);
      if (entropyLength == 0) {
        setupRND(NULL);
        Serial.println("OK");
      } else if (strlen(entropyBase64) == 44) {
        uint8_t entropy[32] = {0};
        safer_decode_base64((unsigned char*) entropyBase64, entropy, 32);
        setupRND(entropy);
        Serial.println("OK");
      } else {
        Serial.println("Invalid entropy.");
      }
    } else if (strcmp("SET_PRIVATE_KEY", command) == 0) {
      char* keyBase64 = strtok(NULL, " \t");
      if (strlen(keyBase64) == 44) {
        safer_decode_base64((unsigned char*) keyBase64, privateKey, 32);
        setPrivateKey();
      } else {
        Serial.println("Invalid private key.");
      }
    }
#endif
    else {
      Serial.println("Command not found.");
    }


  }

}
