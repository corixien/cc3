// ElectronSafeStorage replacement: AES-256-GCM with a random key kept in the OS keyring.
import { Entry } from "@napi-rs/keyring";
import * as Effect from "effect/Effect";
import * as Layer from "effect/Layer";
import * as Option from "effect/Option";
import * as NodeCrypto from "node:crypto";

import * as ElectronSafeStorage from "../../desktop/src/electron/ElectronSafeStorage.ts";

const entry = new Entry("cc3", "safe-storage-key");

const loadKey = (): Buffer => {
  const stored = entry.getPassword();
  if (stored) return Buffer.from(stored, "base64");
  const key = NodeCrypto.randomBytes(32);
  entry.setPassword(key.toString("base64"));
  return key;
};

export const layer = Layer.effect(
  ElectronSafeStorage.ElectronSafeStorage,
  Effect.sync(() => {
    let key: Buffer | undefined;
    const getKey = () => (key ??= loadKey());
    return ElectronSafeStorage.ElectronSafeStorage.of({
      isEncryptionAvailable: Effect.try({
        try: () => {
          getKey();
          return true;
        },
        catch: (cause) => new ElectronSafeStorage.ElectronSafeStorageAvailabilityError({ cause }),
      }).pipe(Effect.orElseSucceed(() => false)),
      encryptString: (value) =>
        Effect.try({
          try: () => {
            const iv = NodeCrypto.randomBytes(12);
            const cipher = NodeCrypto.createCipheriv("aes-256-gcm", getKey(), iv);
            const data = Buffer.concat([cipher.update(value, "utf8"), cipher.final()]);
            return new Uint8Array(Buffer.concat([iv, cipher.getAuthTag(), data]));
          },
          catch: (cause) => new ElectronSafeStorage.ElectronSafeStorageEncryptError({ cause }),
        }),
      decryptString: (value) =>
        Effect.try({
          try: () => {
            const buffer = Buffer.from(value);
            const decipher = NodeCrypto.createDecipheriv(
              "aes-256-gcm",
              getKey(),
              buffer.subarray(0, 12),
            );
            decipher.setAuthTag(buffer.subarray(12, 28));
            return Buffer.concat([decipher.update(buffer.subarray(28)), decipher.final()]).toString(
              "utf8",
            );
          },
          catch: (cause) => new ElectronSafeStorage.ElectronSafeStorageDecryptError({ cause }),
        }),
      selectedStorageBackend: Effect.succeed(Option.some("keyring")),
    });
  }),
);
