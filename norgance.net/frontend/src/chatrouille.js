import ky from 'ky';

import entropy from './entropy';
import { Chatrouille } from './rustyglue/rustyChatrouille';

const publicKey = Uint8Array.from([
  62,
  183,
  168,
  41,
  176,
  205,
  32,
  245,
  188,
  252,
  11,
  89,
  155,
  111,
  236,
  207,
  109,
  164,
  98,
  113,
  7,
  189,
  176,
  212,
  243,
  69,
  180,
  48,
  39,
  216,
  185,
  114,
  252,
  62,
  52,
  251,
  66,
  50,
  161,
  60,
  167,
  6,
  220,
  181,
  122,
  236,
  61,
  174,
  7,
  189,
  193,
  198,
  123,
  243,
  54,
  9,
]);

class GraphqlError extends Error {
  constructor(errors) {
    super('Error from the GraphQL server');
    this.name = this.constructor.name;
    this.errors = errors;
  }
}

let CHATROUILLE_DEBUG_MODE = window.location.hostname === 'localhost';

window.enableChatrouilleDebug = () => {
  CHATROUILLE_DEBUG_MODE = true;
};
if (!CHATROUILLE_DEBUG_MODE) {
  console.info('Run enableChatrouilleDebug() to see the network exchanges.');
}

let instance;
let instanceBuildingPromise;
async function buildChatrouilleInstance() {
  instance = await Chatrouille.withPublicKey(publicKey);
  instanceBuildingPromise = undefined;
}
instanceBuildingPromise = buildChatrouilleInstance();

export async function anonymousGraphql(graphql) {
  if (CHATROUILLE_DEBUG_MODE) {
    console.info('Chatrouille query', graphql);
  }

  if (instanceBuildingPromise) {
    await instanceBuildingPromise;
  }

  const entropyInstance = entropy();
  entropyInstance.ping(); // Ping before processing
  const payload = JSON.stringify({ graphql });
  const query = await instance.packUnsignedQuery(payload);
  let decoded;
  try {
    entropyInstance.ping(); // Ping after processing
    const response = await ky.post('http://localhost:3000/chatrouille', {
      body: query.query,
    });
    entropyInstance.ping(); // Ping after response
    const responseBody = await response.arrayBuffer();
    decoded = await Chatrouille.unpackResponse(new Uint8Array(responseBody), query);
  } finally {
    query.free();
  }
  const jsonResponse = JSON.parse(decoded);
  entropyInstance.ping(); // Ping after response processing
  if (CHATROUILLE_DEBUG_MODE) {
    console.info('Chatrouille response', jsonResponse);
  }
  if (jsonResponse.errors && Array.isArray(jsonResponse.errors) && jsonResponse.errors.length > 0) {
    throw new GraphqlError(jsonResponse.errors);
  }

  const data = jsonResponse.data;
  const dataEntries = Object.entries(data);
  if (dataEntries.length === 1) {
    return dataEntries[0][1];
  }
  return data;
}

export async function authenticatedQuery(/* graphql, citizenIdentifier, citizenPrivateKey */) {
  throw new Error('TODO');
}
