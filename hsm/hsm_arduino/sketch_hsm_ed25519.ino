#include <base64.hpp>
#include <ChaCha.h>
#include <Ed25519.h>

static const int analogInPin = A1;

/** SECRETS */
// Should be a random number between 0 and 1023
static const uint8_t dynamicdelaySecret = 123;
static const uint8_t startingEntropy[32] = {
  1, 2, 3, 4, 5, 6, 7, 8,
  9, 10, 11, 12, 13, 14, 15, 16,
  17, 18, 19, 20, 21, 22, 23, 24,
  25, 26, 27, 28, 29, 30, 31, 32
};

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
      timeToWait = map(~(a ^ dynamicdelaySecret), 0, 1023, 20, 50);
      delayMicroseconds(timeToWait);
      b = analogRead(analogInPin);
      timeToWait = map(~(b ^ dynamicdelaySecret), 0, 1023, 500, 4000);
      delayMicroseconds(timeToWait);
    } while (a == b);

    if (a & 1) {
      buffer[i / 8] |= 1 << (i % 8);
    } else {
      buffer[i / 8] &= ~(1 << (i % 8));
    }
  }
}

static void setupRND(const uint8_t entropy[32]) {

  uint8_t entropyStarter[32] = {0};
  if (entropy != NULL) {
    memcpy(entropyStarter, entropy, 32);
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


static void collectRND(uint8_t output[], const uint16_t size) {

  uint16_t remainingBits = size;
  uint8_t *outputBatchPointer = output;
  uint8_t *buffer = documentBuffer;

  while (remainingBits > 0) {
    uint16_t sizeCurrentBatch = min(256, remainingBits);
    collectEntropyBits(buffer, sizeCurrentBatch);

    chacha.encrypt(outputBatchPointer, buffer, sizeCurrentBatch);

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


static void derivePublicKey() {
  Ed25519::derivePublicKey((uint8_t*)publicKey, (const uint8_t*)privateKey);
}

static void printPublicKey() {
  memset(documentBuffer, 0, 256);
  encode_base64(publicKey, 32, documentBuffer);
  Serial.println((char*) documentBuffer);
  memset(documentBuffer, 0, 256);
}

static void randomPrivateKey() {
  collectRND(privateKey, 32);
  derivePublicKey();
  Serial.print("PUBLIC_KEY ");
  printPublicKey();
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
  pinMode(LED_BUILTIN, OUTPUT);

  Serial.begin(9600);

  digitalWrite(LED_BUILTIN, HIGH);
  Serial.println("BOOTING");
  setupRND(startingEntropy);
  randomPrivateKey();
  Serial.println("READY");
  digitalWrite(LED_BUILTIN, LOW);
}

void loop() {
  if (!Serial) return;

  digitalWrite(LED_BUILTIN, LOW);
  memset(serialInputBuffer, 0, 255);
  if (Serial.readBytesUntil('\n', serialInputBuffer, 255) > 0) {
    digitalWrite(LED_BUILTIN, HIGH);
    char* command = strtok(serialInputBuffer, " ");

    if (command == NULL) return;

    if (strcmp("CANARD", command) == 0) {
      Serial.println("KOINKOIN");
    } else if (strcmp("SIGN", command) == 0) {
      char* documentBase64 = strtok(NULL, " ");
      unsigned int documentSize = safer_decode_base64((unsigned char*) documentBase64, documentBuffer, 128);
      sign(documentSize);
    } else if (strcmp("GET_PUBLIC_KEY", command) == 0) {
      printPublicKey();
    } else if (strcmp("RENEW_KEYPAIR", command) == 0) {
      randomPrivateKey();
    } else {
      Serial.println("UNKNOWN COMMAND");
    }
  }

}