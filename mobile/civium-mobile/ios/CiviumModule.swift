/**
 * React Native native module for iOS.
 *
 * Calls into the civium-ffi Rust library via UniFFI-generated Swift bindings.
 * The XCFramework must be compiled with cargo-lipo / xcodebuild and added to
 * the Xcode project (Build Phases → Link Binary With Libraries).
 *
 * Build command (run from desktop/):
 *   cargo build -p civium-ffi --release \
 *     --target aarch64-apple-ios \
 *     --target aarch64-apple-ios-sim \
 *     --target x86_64-apple-ios
 *
 *   xcodebuild -create-xcframework \
 *     -library target/aarch64-apple-ios/release/libcivium_ffi.a \
 *       -headers generated/swift/ \
 *     -library target/aarch64-apple-ios-sim/release/libcivium_ffi.a \
 *       -headers generated/swift/ \
 *     -output CiviumFFI.xcframework
 *
 * Then generate Swift bindings:
 *   cargo run --bin uniffi-bindgen generate \
 *     --library target/aarch64-apple-ios/release/libcivium_ffi.a \
 *     --language swift \
 *     --out-dir generated/swift/
 */

import Foundation
import CiviumFfi   // UniFFI-generated module

@objc(CiviumModule)
class CiviumModule: NSObject {

    /// Path to the app's Documents directory (persists across updates).
    @objc func getDataDir(_ resolve: RCTPromiseResolveBlock, reject _: RCTPromiseRejectBlock) {
        let dir = NSSearchPathForDirectoriesInDomains(.documentDirectory, .userDomainMask, true).first ?? ""
        resolve(dir)
    }

    @objc func identityExists(_ dataDir: String, resolve: RCTPromiseResolveBlock, reject _: RCTPromiseRejectBlock) {
        resolve(CiviumFfi.identityExists(dataDir: dataDir))
    }

    @objc func identityInit(_ dataDir: String, resolve: RCTPromiseResolveBlock, reject: RCTPromiseRejectBlock) {
        do {
            let info = try CiviumFfi.identityInit(dataDir: dataDir)
            resolve(info.toDict())
        } catch { reject("CIVIUM_FFI", error.localizedDescription, error) }
    }

    @objc func identityFromSecret(_ dataDir: String, secretB58: String,
                                   resolve: RCTPromiseResolveBlock, reject: RCTPromiseRejectBlock) {
        do {
            let info = try CiviumFfi.identityFromSecret(dataDir: dataDir, secretB58: secretB58)
            resolve(info.toDict())
        } catch { reject("CIVIUM_FFI", error.localizedDescription, error) }
    }

    @objc func identityInfo(_ dataDir: String, resolve: RCTPromiseResolveBlock, reject: RCTPromiseRejectBlock) {
        do {
            let info = try CiviumFfi.identityInfo(dataDir: dataDir)
            resolve(info.toDict())
        } catch { reject("CIVIUM_FFI", error.localizedDescription, error) }
    }

    @objc func pairingComplete(_ link: String, resolve: RCTPromiseResolveBlock, reject: RCTPromiseRejectBlock) {
        do {
            let secret = try CiviumFfi.pairingComplete(link: link)
            resolve(secret)
        } catch { reject("CIVIUM_FFI", error.localizedDescription, error) }
    }

    @objc func networkList(_ dataDir: String, resolve: RCTPromiseResolveBlock, reject: RCTPromiseRejectBlock) {
        do {
            let nets = try CiviumFfi.networkList(dataDir: dataDir)
            resolve(nets.map { $0.toDict() })
        } catch { reject("CIVIUM_FFI", error.localizedDescription, error) }
    }

    @objc func messageList(_ dataDir: String, networkCid: String,
                            resolve: RCTPromiseResolveBlock, reject: RCTPromiseRejectBlock) {
        do {
            let msgs = try CiviumFfi.messageList(dataDir: dataDir, networkCid: networkCid)
            resolve(msgs.map { $0.toDict() })
        } catch { reject("CIVIUM_FFI", error.localizedDescription, error) }
    }

    @objc func messageSend(_ dataDir: String, networkCid: String, body: String,
                            resolve: RCTPromiseResolveBlock, reject: RCTPromiseRejectBlock) {
        do {
            let msg = try CiviumFfi.messageSend(dataDir: dataDir, networkCid: networkCid, body: body)
            resolve(msg.toDict())
        } catch { reject("CIVIUM_FFI", error.localizedDescription, error) }
    }

    @objc static func requiresMainQueueSetup() -> Bool { false }
}

// ── Extension helpers ─────────────────────────────────────────────────────────

private extension IdentityInfo {
    func toDict() -> [String: Any] {
        ["cid_short": cidShort, "cid_full": cidFull, "secret_b58": secretB58]
    }
}

private extension NetworkInfo {
    func toDict() -> [String: Any] {
        ["cid_short": cidShort, "name": name, "member_count": memberCount]
    }
}

private extension MessageInfo {
    func toDict() -> [String: Any] {
        ["id": id, "author_cid_short": authorCidShort, "body": body,
         "sent_at": sentAt, "is_direct": isDirect]
    }
}
