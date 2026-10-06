import { MongoClient, type Db } from "mongodb";

const uri = process.env.MONGODB_URI?.trim();
const dbName = process.env.MONGODB_DB?.trim() || "ghostnote";

type GlobalMongo = {
  client?: MongoClient;
  promise?: Promise<MongoClient>;
  indexes?: Promise<void>;
};

const globalForMongo = globalThis as typeof globalThis & { __ghostnoteMongo?: GlobalMongo };

async function getClient(): Promise<MongoClient | null> {
  if (!uri) return null;

  const cache = globalForMongo.__ghostnoteMongo ?? {};
  if (cache.client) return cache.client;

  if (!cache.promise) {
    const client = new MongoClient(uri, {
      maxPoolSize: 8,
      serverSelectionTimeoutMS: 8000,
      retryWrites: true,
    });
    cache.promise = client.connect().then((connected) => {
      cache.client = connected;
      return connected;
    });
    globalForMongo.__ghostnoteMongo = cache;
  }

  try {
    return await cache.promise;
  } catch (error) {
    cache.promise = undefined;
    cache.client = undefined;
    throw error;
  }
}

async function ensureIndexes(db: Db) {
  const cache = globalForMongo.__ghostnoteMongo ?? {};
  if (!cache.indexes) {
    cache.indexes = Promise.all([
      db.collection("reservations").createIndex({ email: 1 }, { unique: true }),
      db.collection("subscriptions").createIndex({ email: 1 }, { unique: true }),
      db.collection("offerState").createIndex({ key: 1 }, { unique: true }),
    ])
      .then(() => undefined)
      .catch((error) => {
        cache.indexes = undefined;
        throw error;
      });
    globalForMongo.__ghostnoteMongo = cache;
  }
  await cache.indexes;
}

export async function getDb(): Promise<Db | null> {
  const client = await getClient();
  if (!client) return null;
  const db = client.db(dbName);
  try {
    await ensureIndexes(db);
  } catch {
    // Unique indexes are best-effort; do not block checkout if they already exist.
  }
  return db;
}

export async function pingDb() {
  if (!uri) return { ok: false as const, reason: "missing" as const };
  try {
    const db = await getDb();
    if (!db) return { ok: false as const, reason: "unavailable" as const };
    await db.command({ ping: 1 });
    return { ok: true as const, db: dbName };
  } catch {
    return { ok: false as const, reason: "error" as const };
  }
}

export function isMongoConfigured() {
  return Boolean(uri);
}
