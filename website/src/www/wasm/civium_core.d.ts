/* tslint:disable */
/* eslint-disable */

/**
 * Build a new agenda event. Returns an `AgendaEvent` JSON.
 *
 * `start_at` / `end_at`: Unix timestamps in seconds (f64 for JS interop).
 * Pass `0` for `end_at` to leave it unset.
 */
export function agenda_event_build(title: string, description: string, location: string, start_at: number, end_at: number, network_cid_short: string, created_by: string): any;

export function civium_version(): string;

/**
 * Build and encrypt a new document. Returns a `Document` JSON.
 */
export function document_build(title: string, body: string, network_cid_short: string, created_by: string, group_key_b58: string): any;

/**
 * Decrypt a document body. Returns the plaintext string.
 */
export function document_decrypt_body(nonce_b58: string, body_ciphertext: string, group_key_b58: string): string;

/**
 * Generate a new random Ed25519 identity.
 *
 * Returns `{ cid_full, cid_short, secret_b58, pub_key_b58 }`.
 * Store `secret_b58` securely — it is the only way to recover the identity.
 */
export function generate_identity(): any;

/**
 * Decrypt a message encrypted by `group_key_encrypt`. Returns the plaintext string.
 */
export function group_key_decrypt(group_key_b58: string, nonce_b58: string, ciphertext_b58: string): string;

/**
 * Encrypt `plaintext` with the group key.
 *
 * Returns `{ nonce_b58, ciphertext_b58 }`.
 */
export function group_key_encrypt(group_key_b58: string, plaintext: string): any;

/**
 * Generate a new random group key. Returns its base58 representation.
 */
export function group_key_generate(): string;

/**
 * Restore an identity from its base58-encoded secret key.
 *
 * Returns `{ cid_full, cid_short, pub_key_b58 }`.
 */
export function load_identity(secret_b58: string): any;

/**
 * Build and encrypt a message. The message is NOT stored — the caller persists it.
 *
 * `kind`: `"thread"` | `"direct:<cid_short>"` | `"e2e:<cid_full>"`
 *
 * Returns a `Message` JSON ready to be stored in IndexedDB and broadcast via P2P.
 */
export function message_build(author_cid_short: string, kind: string, content: string, group_key_b58: string): any;

/**
 * Decrypt a message body. Returns the plaintext string.
 */
export function message_decrypt(nonce_b58: string, ciphertext_b58: string, group_key_b58: string): string;

/**
 * Create a new Civium network.
 *
 * Returns the full `NetworkData` JSON — persist it to IndexedDB.
 */
export function network_create(name: string, admin_secret_b58: string, admin_display_name: string): any;

/**
 * Create a new governance proposal.
 *
 * `options_json`: JSON array of option strings, e.g. `["Oui","Non","Abstention"]`.
 * `closes_at`: Unix timestamp (seconds) when voting closes (0 = 7 days from now).
 *
 * Returns a `Proposal` JSON.
 */
export function proposal_create(title: string, description: string, options_json: string, author_cid_full: string, network_cid_short: string, quorum_percent: number, closes_at: number): any;

/**
 * Compute the vote result for a proposal.
 *
 * `proposals_json`: JSON of a single `Proposal`.
 * `votes_json`: JSON array of `Vote`.
 * `delegations_json`: JSON array of `VoteDelegation` (pass `[]` if none).
 * `total_members`: number of members entitled to vote.
 *
 * Returns a `VoteResult` JSON.
 */
export function vote_compute(proposal_json: string, votes_json: string, delegations_json: string, total_members: number): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly agenda_event_build: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => [number, number, number];
    readonly civium_version: () => [number, number];
    readonly document_build: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number];
    readonly document_decrypt_body: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly generate_identity: () => [number, number, number];
    readonly group_key_decrypt: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly group_key_encrypt: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly group_key_generate: () => [number, number];
    readonly load_identity: (a: number, b: number) => [number, number, number];
    readonly message_build: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number];
    readonly network_create: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly proposal_create: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => [number, number, number];
    readonly vote_compute: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly message_decrypt: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
