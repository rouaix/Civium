package com.civium

import com.facebook.react.bridge.*
import uniffi.civium_ffi.*

/**
 * React Native native module for Android.
 *
 * Calls into the civium-ffi Rust library via UniFFI-generated Kotlin bindings.
 * The .so file (libcivium_ffi.so) must be compiled with cargo-ndk and placed in
 * android/app/src/main/jniLibs/<abi>/ before building.
 *
 * Build command (run from desktop/):
 *   cargo ndk -t armeabi-v7a -t arm64-v8a -t x86_64 \
 *     -o ../mobile/civium-mobile/android/app/src/main/jniLibs \
 *     build -p civium-ffi --release
 *
 * Then generate Kotlin bindings:
 *   cargo run --bin uniffi-bindgen generate \
 *     --library target/release/libcivium_ffi.so \
 *     --language kotlin \
 *     --out-dir ../mobile/civium-mobile/android/app/src/main/java/uniffi/civium_ffi/
 */
class CiviumModule(private val reactContext: ReactApplicationContext) :
    ReactContextBaseJavaModule(reactContext) {

    override fun getName() = "CiviumModule"

    /** Absolute path to the app's internal data directory. */
    @ReactMethod
    fun getDataDir(promise: Promise) {
        promise.resolve(reactContext.filesDir.absolutePath)
    }

    @ReactMethod
    fun identityExists(dataDir: String, promise: Promise) {
        promise.resolve(identityExists(dataDir))
    }

    @ReactMethod
    fun identityInit(dataDir: String, promise: Promise) {
        runCatching { identityInit(dataDir) }
            .onSuccess { promise.resolve(it.toWritableMap()) }
            .onFailure { promise.reject("CIVIUM_FFI", it.message, it) }
    }

    @ReactMethod
    fun identityFromSecret(dataDir: String, secretB58: String, promise: Promise) {
        runCatching { identityFromSecret(dataDir, secretB58) }
            .onSuccess { promise.resolve(it.toWritableMap()) }
            .onFailure { promise.reject("CIVIUM_FFI", it.message, it) }
    }

    @ReactMethod
    fun identityInfo(dataDir: String, promise: Promise) {
        runCatching { identityInfo(dataDir) }
            .onSuccess { promise.resolve(it.toWritableMap()) }
            .onFailure { promise.reject("CIVIUM_FFI", it.message, it) }
    }

    @ReactMethod
    fun pairingComplete(link: String, promise: Promise) {
        runCatching { pairingComplete(link) }
            .onSuccess { promise.resolve(it) }
            .onFailure { promise.reject("CIVIUM_FFI", it.message, it) }
    }

    @ReactMethod
    fun networkList(dataDir: String, promise: Promise) {
        runCatching { networkList(dataDir) }
            .onSuccess { list ->
                val arr = Arguments.createArray()
                list.forEach { arr.pushMap(it.toWritableMap()) }
                promise.resolve(arr)
            }
            .onFailure { promise.reject("CIVIUM_FFI", it.message, it) }
    }

    @ReactMethod
    fun messageList(dataDir: String, networkCid: String, promise: Promise) {
        runCatching { messageList(dataDir, networkCid) }
            .onSuccess { list ->
                val arr = Arguments.createArray()
                list.forEach { arr.pushMap(it.toWritableMap()) }
                promise.resolve(arr)
            }
            .onFailure { promise.reject("CIVIUM_FFI", it.message, it) }
    }

    @ReactMethod
    fun messageSend(dataDir: String, networkCid: String, body: String, promise: Promise) {
        runCatching { messageSend(dataDir, networkCid, body) }
            .onSuccess { promise.resolve(it.toWritableMap()) }
            .onFailure { promise.reject("CIVIUM_FFI", it.message, it) }
    }
}

// ── Extension helpers ─────────────────────────────────────────────────────────

private fun IdentityInfo.toWritableMap(): WritableMap =
    Arguments.createMap().apply {
        putString("cid_short",  cidShort)
        putString("cid_full",   cidFull)
        putString("secret_b58", secretB58)
    }

private fun NetworkInfo.toWritableMap(): WritableMap =
    Arguments.createMap().apply {
        putString("cid_short",    cidShort)
        putString("name",         name)
        putInt("member_count",    memberCount.toInt())
    }

private fun MessageInfo.toWritableMap(): WritableMap =
    Arguments.createMap().apply {
        putString("id",               id)
        putString("author_cid_short", authorCidShort)
        putString("body",             body)
        putDouble("sent_at",          sentAt.toDouble())
        putBoolean("is_direct",       isDirect)
    }
