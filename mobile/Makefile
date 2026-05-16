# Civium mobile — Makefile
# Run from the civium/mobile/ directory.
# Requires: cargo-ndk, cargo-lipo, uniffi-bindgen (cargo install uniffi-bindgen-cli)

CARGO  = cargo
UNIFFI = uniffi-bindgen
DESKTOP_DIR = ../desktop

ANDROID_TARGETS = aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
IOS_TARGETS     = aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

ANDROID_OUT = civium-mobile/android/app/src/main/jniLibs
SWIFT_OUT   = civium-mobile/ios/generated/swift
KOTLIN_OUT  = civium-mobile/android/app/src/main/java/uniffi/civium_ffi

.PHONY: all android ios android-bindings ios-bindings clean

all: android ios

# ── Android ───────────────────────────────────────────────────────────────────

android:
	cd $(DESKTOP_DIR) && \
	cargo ndk $(foreach t,$(ANDROID_TARGETS),-t $(t)) \
	  -o ../mobile/$(ANDROID_OUT) \
	  build -p civium-ffi --release

android-bindings: android
	cd $(DESKTOP_DIR) && \
	$(UNIFFI) generate \
	  --library target/aarch64-linux-android/release/libcivium_ffi.so \
	  --language kotlin \
	  --out-dir ../mobile/$(KOTLIN_OUT)

# ── iOS ───────────────────────────────────────────────────────────────────────

ios:
	cd $(DESKTOP_DIR) && \
	$(foreach t,$(IOS_TARGETS),\
	  cargo build -p civium-ffi --release --target $(t);)
	xcodebuild -create-xcframework \
	  -library $(DESKTOP_DIR)/target/aarch64-apple-ios/release/libcivium_ffi.a \
	    -headers civium-mobile/ios/generated/swift/ \
	  -library $(DESKTOP_DIR)/target/aarch64-apple-ios-sim/release/libcivium_ffi.a \
	    -headers civium-mobile/ios/generated/swift/ \
	  -output civium-mobile/ios/CiviumFFI.xcframework

ios-bindings:
	cd $(DESKTOP_DIR) && \
	$(UNIFFI) generate \
	  --library target/aarch64-apple-ios/release/libcivium_ffi.a \
	  --language swift \
	  --out-dir ../mobile/$(SWIFT_OUT)

# ── Misc ─────────────────────────────────────────────────────────────────────

clean:
	cd $(DESKTOP_DIR) && cargo clean -p civium-ffi
	rm -rf civium-mobile/ios/CiviumFFI.xcframework
	rm -rf civium-mobile/ios/generated
	rm -rf civium-mobile/android/app/src/main/jniLibs
	rm -rf civium-mobile/android/app/src/main/java/uniffi
