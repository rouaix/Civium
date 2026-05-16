/**
 * React Native bridge to the native CiviumModule (Kotlin / Swift).
 * The native side calls into the civium-ffi Rust library via UniFFI-generated bindings.
 */
import { NativeModules, Platform } from 'react-native';
import type { IdentityInfo, MessageInfo, NetworkInfo } from '../types';

const LINKING_ERROR =
  "Le module natif Civium est introuvable. Assurez-vous d'avoir compilé " +
  'civium-ffi pour la plateforme cible (Android: cargo-ndk, iOS: cargo-lipo).';

const { CiviumModule } = NativeModules;

if (!CiviumModule) {
  // In dev/storybook without native build, provide stubs so the app doesn't crash on import.
  console.warn(LINKING_ERROR);
}

function stub(name: string): never {
  throw new Error(`CiviumModule.${name}: ${LINKING_ERROR}`);
}

export interface ICiviumModule {
  /** Path to the app's data directory (provided by the native side). */
  getDataDir(): Promise<string>;

  identityExists(dataDir: string): Promise<boolean>;
  identityInit(dataDir: string): Promise<IdentityInfo>;
  identityFromSecret(dataDir: string, secretB58: string): Promise<IdentityInfo>;
  identityInfo(dataDir: string): Promise<IdentityInfo>;

  pairingComplete(link: string): Promise<string>;

  networkList(dataDir: string): Promise<NetworkInfo[]>;

  messageList(dataDir: string, networkCid: string): Promise<MessageInfo[]>;
  messageSend(dataDir: string, networkCid: string, body: string): Promise<MessageInfo>;
}

const Civium: ICiviumModule = CiviumModule ?? {
  getDataDir:         () => stub('getDataDir'),
  identityExists:     () => stub('identityExists'),
  identityInit:       () => stub('identityInit'),
  identityFromSecret: () => stub('identityFromSecret'),
  identityInfo:       () => stub('identityInfo'),
  pairingComplete:    () => stub('pairingComplete'),
  networkList:        () => stub('networkList'),
  messageList:        () => stub('messageList'),
  messageSend:        () => stub('messageSend'),
};

export default Civium;

/** Convenience: resolves the data dir once then calls the given FFI function. */
export async function withDataDir<T>(fn: (dataDir: string) => Promise<T>): Promise<T> {
  const dataDir = await Civium.getDataDir();
  return fn(dataDir);
}
