import React, { useEffect, useState } from 'react';
import {
  ActivityIndicator,
  Alert,
  StyleSheet,
  Text,
  TextInput,
  TouchableOpacity,
  View,
} from 'react-native';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../types';
import Civium, { withDataDir } from '../native/CiviumModule';

type Props = NativeStackScreenProps<RootStackParamList, 'Onboarding'>;

export default function OnboardingScreen({ navigation }: Props) {
  const [checking, setChecking]  = useState(true);
  const [loading, setLoading]    = useState(false);
  const [error, setError]        = useState<string | null>(null);

  // On mount: check if an identity already exists
  useEffect(() => {
    withDataDir(Civium.identityExists)
      .then(async (exists) => {
        if (exists) {
          const identity = await withDataDir(Civium.identityInfo);
          navigation.replace('Networks', { identity });
        }
      })
      .catch((e) => setError(String(e)))
      .finally(() => setChecking(false));
  }, [navigation]);

  async function handleCreate() {
    setLoading(true);
    setError(null);
    try {
      const identity = await withDataDir(Civium.identityInit);
      navigation.replace('Networks', { identity });
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  if (checking) {
    return (
      <View style={styles.center}>
        <ActivityIndicator size="large" color="#4f46e5" />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Civium</Text>
        <Text style={styles.subtitle}>Réseau souverain</Text>
      </View>

      {error && (
        <View style={styles.errorBox}>
          <Text style={styles.errorText}>{error}</Text>
        </View>
      )}

      <View style={styles.actions}>
        <TouchableOpacity
          style={[styles.btn, styles.btnPrimary]}
          onPress={handleCreate}
          disabled={loading}
        >
          {loading ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <Text style={styles.btnPrimaryText}>Créer une nouvelle identité</Text>
          )}
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.btn, styles.btnSecondary]}
          onPress={() => navigation.navigate('Pairing')}
          disabled={loading}
        >
          <Text style={styles.btnSecondaryText}>
            Scanner le QR code depuis l'app desktop
          </Text>
        </TouchableOpacity>
      </View>

      <Text style={styles.hint}>
        Votre clé privée ne quitte jamais cet appareil.
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  center:    { flex: 1, justifyContent: 'center', alignItems: 'center' },
  container: { flex: 1, backgroundColor: '#f8fafc', padding: 24 },
  header:    { marginTop: 80, marginBottom: 48, alignItems: 'center' },
  title:     { fontSize: 40, fontWeight: '700', color: '#1e1b4b', letterSpacing: -1 },
  subtitle:  { fontSize: 16, color: '#6366f1', marginTop: 4 },
  errorBox:  {
    backgroundColor: '#fef2f2',
    borderColor: '#fca5a5',
    borderWidth: 1,
    borderRadius: 8,
    padding: 12,
    marginBottom: 16,
  },
  errorText: { color: '#dc2626', fontSize: 14 },
  actions:   { gap: 12 },
  btn:       {
    borderRadius: 12,
    paddingVertical: 14,
    paddingHorizontal: 20,
    alignItems: 'center',
  },
  btnPrimary:      { backgroundColor: '#4f46e5' },
  btnPrimaryText:  { color: '#fff', fontWeight: '600', fontSize: 16 },
  btnSecondary:    {
    backgroundColor: '#fff',
    borderColor: '#c7d2fe',
    borderWidth: 1.5,
  },
  btnSecondaryText: { color: '#4f46e5', fontWeight: '600', fontSize: 16 },
  hint: {
    marginTop: 32,
    textAlign: 'center',
    color: '#94a3b8',
    fontSize: 13,
  },
});
