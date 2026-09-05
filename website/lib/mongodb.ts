import { MongoClient, type Db } from "mongodb";

const uri = process.env.MONGODB_URI;
const dbName = process.env.MONGODB_DB || "ghostnote";

type GlobalMongo = {
  client?: MongoClient;
  promise?: Promise<MongoClient>;
};

const globalForMongo = globalThis as typeof globalThis & { __ghostnoteMongo?: GlobalMongo };

async function getClient(): Promise<MongoClient | null> {
  if (!uri) return null;

  const cache = globalForMongo.__ghostnoteMongo ?? {};
  if (cache.client) return cache.client;

  if (!cache.promise) {
    const client = new MongoClient(uri);
    cache.promise = client.connect().then((connected) => {
      cache.client = connected;
      return connected;
    });
    globalForMongo.__ghostnoteMongo = cache;
  }

  return cache.promise;
}

export async function getDb(): Promise<Db | null> {
  const client = await getClient();
  return client ? client.db(dbName) : null;
}

export function isMongoConfigured() {
  return Boolean(uri);
}
