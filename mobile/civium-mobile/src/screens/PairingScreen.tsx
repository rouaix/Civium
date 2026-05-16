/**
 * Scan the QR code displayed by the desktop app (pair_init output).
 * The QR code encodes a `civium://pair/<b58payload>` deep link.
 * On scan, we complete the pairing → recover secret_b58 → save identity.
 */
import React, { useCallback, useState } from 'react';
import {
  ActivityIndicator,
  Alert,
  StyleSheet,
  Text,
  TouchableOpacity,
  View,
} from 'react-native';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import {
  Camera,
  useCameraDevice,
  useCodeScanner,
} from 'react-native-vision-camera';
import type { RootStackParamList } from '../types';
import Civium, { withDataDir } from '../native/CiviumModule';

type Props = NativeStackScreenProps<RootStackParamList, 'Pairing'>;

export default function PairingScreen({ navigation }: Props) {
  const device  = useCameraDevice('back');
  const [processing, setProcessing] = useState(false);
  const [error, setError]           = useState<string | null>(null);

  const onCodeScanned = useCallback(
    async (codes: { value?: string }[]) => {
      if (processing) return;
      const value = codes[0]?.value;
      if (!value?.startsWith('civium://pair/')) return;

      setProcessing(true);
      setError(null);
      try {
        const secretB58 = await Civium.pairingComplete(value);
        const identity  = await withDataDir((dir) =>
          Civium.identityFromSecret(dir, secretB58),
        );
        navigation.replace('Networks', { identity });
      } catch (e) {
        setError(String(e));
        setProcessing(false);
      }
    },
    [processing, navigation],
  );

  const codeScanner = useCodeScanner({
    codeTypes: ['qr'],
    onCodeScanned,
  });

  if (!device) {
    return (
      <View style={styles.center}>
        <Text style={styles.noCamera}>Caméra arrière introuvable.</Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <Camera
        style={StyleSheet.absoluteFill}
        device={device}
        isActive={!processing}
        codeScanner={codeScanner}
      />

      {/* Overlay */}
      <View style={styles.overlay}>
        <View style={styles.topBar}>
          <TouchableOpacity
            style={styles.backBtn}
            onPress={() => navigation.goBack()}
          >
            <Text style={styles.backBtnText}>← Retour</Text>
          </TouchableOpacity>
          <Text style={styles.topTitle}>Scanner le QR Civium</Text>
        </View>

        <View style={styles.scanFrame} />

        <View style={styles.bottomBar}>
          {processing ? (
            <View style={styles.statusRow}>
              <ActivityIndicator color="#fff" />
              <Text style={styles.statusText}>Jumelage en cours…</Text>
            </View>
          ) : error ? (
            <View style={styles.errorBox}>
              <Text style={styles.errorText}>{error}</Text>
              <TouchableOpacity onPress={() => setError(null)}>
                <Text style={styles.retryText}>Réessayer</Text>
              </TouchableOpacity>
            </View>
          ) : (
            <Text style={styles.hint}>
              Pointez la caméra sur le QR code affiché dans l'app desktop
              (Identité → Jumeler un appareil).
            </Text>
          )}
        </View>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  center:     { flex: 1, justifyContent: 'center', alignItems: 'center', backgroundColor: '#000' },
  noCamera:   { color: '#fff', fontSize: 16 },
  container:  { flex: 1, backgroundColor: '#000' },
  overlay:    { ...StyleSheet.absoluteFillObject, flexDirection: 'column', justifyContent: 'space-between' },
  topBar:     {
    backgroundColor: 'rgba(0,0,0,0.6)',
    paddingTop: 56,
    paddingBottom: 16,
    paddingHorizontal: 20,
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  backBtn:      { paddingVertical: 4, paddingHorizontal: 8 },
  backBtnText:  { color: '#fff', fontSize: 16 },
  topTitle:     { color: '#fff', fontSize: 18, fontWeight: '600' },
  scanFrame:    {
    width: 250,
    height: 250,
    alignSelf: 'center',
    borderWidth: 2,
    borderColor: '#6366f1',
    borderRadius: 16,
    backgroundColor: 'transparent',
  },
  bottomBar: {
    backgroundColor: 'rgba(0,0,0,0.6)',
    padding: 24,
    minHeight: 100,
    justifyContent: 'center',
  },
  statusRow:  { flexDirection: 'row', alignItems: 'center', gap: 12 },
  statusText: { color: '#fff', fontSize: 15 },
  errorBox:   { gap: 8 },
  errorText:  { color: '#fca5a5', fontSize: 14 },
  retryText:  { color: '#a5b4fc', fontSize: 14, fontWeight: '600' },
  hint:       { color: 'rgba(255,255,255,0.7)', fontSize: 14, textAlign: 'center', lineHeight: 20 },
});
