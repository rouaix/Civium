#import <React/RCTBridgeModule.h>

@interface RCT_EXTERN_MODULE(CiviumModule, NSObject)

RCT_EXTERN_METHOD(getDataDir:(RCTPromiseResolveBlock)resolve reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(identityExists:(NSString *)dataDir
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(identityInit:(NSString *)dataDir
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(identityFromSecret:(NSString *)dataDir
                  secretB58:(NSString *)secretB58
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(identityInfo:(NSString *)dataDir
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(pairingComplete:(NSString *)link
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(networkList:(NSString *)dataDir
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(messageList:(NSString *)dataDir
                  networkCid:(NSString *)networkCid
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(messageSend:(NSString *)dataDir
                  networkCid:(NSString *)networkCid
                  body:(NSString *)body
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject)

@end
