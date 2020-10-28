import ky from 'ky';

import entropy from './entropy';
import { Chatrouille } from './rustyglue/rustyChatrouille';

class GraphqlError extends Error {
  constructor(errors) {
    super('Error from the GraphQL server');
    this.name = this.constructor.name;
    this.errors = errors;
  }
}

let CHATROUILLE_DEBUG_MODE = process.env.VUE_APP_CHATROUILLE_DEBUG_MODE === 'true';
const CHATROUILLE_PATH = process.env.VUE_APP_CHATROUILLE_PATH || 'http://localhost:3000/chatrouille';
const CHATROUILLE_INFORMATION_PATH = process.env.VUE_APP_CHATROUILLE_INFORMATION_PATH || `${CHATROUILLE_PATH}_information`;
const CHATROUILLE_HARCODED_PUBLIC_KEY = process.env.VUE_APP_CHATROUILLE_HARCODED_PUBLIC_KEY;

window.enableChatrouilleDebug = () => {
  CHATROUILLE_DEBUG_MODE = true;
};
if (!CHATROUILLE_DEBUG_MODE) {
  console.info('Run enableChatrouilleDebug() to see the network exchanges.');
}

async function loadChatrouilleInformation() {
  const response = await ky.get(CHATROUILLE_INFORMATION_PATH);
  const json = await response.json();
  console.log(json);
  return json;
}

let instance;
let instanceBuildingPromise;
async function buildChatrouilleInstance() {
  const entropyInstance = entropy();
  entropyInstance.ping();
  const information = await loadChatrouilleInformation();
  instance = await Chatrouille.withPublicKeyAndSignatureBase64(
    information.public_key_x448,
    information.public_key_x448_signature,
    CHATROUILLE_HARCODED_PUBLIC_KEY,
  );
  instanceBuildingPromise = undefined;
  entropyInstance.ping();
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

  const exp = Math.ceil(+new Date() / 1000);
  const payload = JSON.stringify({
    graphql,
    exp,
  });
  const query = await instance.packUnsignedQuery(payload);
  let decoded;
  try {
    entropyInstance.ping(); // Ping after processing
    const response = await ky.post(CHATROUILLE_PATH, {
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
