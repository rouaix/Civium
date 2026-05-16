import React, { useCallback, useEffect, useState } from 'react';
import {
  ActivityIndicator,
  FlatList,
  RefreshControl,
  StyleSheet,
  Text,
  TouchableOpacity,
  View,
} from 'react-native';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import type { NetworkInfo, RootStackParamList } from '../types';
import Civium, { withDataDir } from '../native/CiviumModule';

type Props = NativeStackScreenProps<RootStackParamList, 'Networks'>;

export default function NetworksScreen({ route, navigation }: Props) {
  const { identity } = route.params;
  const [networks, setNetworks]   = useState<NetworkInfo[]>([]);
  const [loading, setLoading]     = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError]         = useState<string | null>(null);

  const loadNetworks = useCallback(async () => {
    setError(null);
    try {
      const nets = await withDataDir(Civium.networkList);
      setNetworks(nets);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    loadNetworks().finally(() => setLoading(false));
  }, [loadNetworks]);

  async function onRefresh() {
    setRefreshing(true);
    await loadNetworks();
    setRefreshing(false);
  }

  function renderNetwork({ item }: { item: NetworkInfo }) {
    return (
      <TouchableOpacity
        style={styles.card}
        onPress={() => navigation.navigate('Messages', { identity, network: item })}
        activeOpacity={0.75}
      >
        <View style={styles.cardAvatar}>
          <Text style={styles.cardAvatarText}>
            {item.name.substring(0, 2).toUpperCase()}
          </Text>
        </View>
        <View style={styles.cardBody}>
          <Text style={styles.cardName}>{item.name}</Text>
          <Text style={styles.cardMeta}>
            {item.member_count} membre{item.member_count !== 1 ? 's' : ''}
            {'  ·  '}
            <Text style={styles.cardCid}>{item.cid_short}</Text>
          </Text>
        </View>
        <Text style={styles.chevron}>›</Text>
      </TouchableOpacity>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <View>
          <Text style={styles.headerTitle}>Mes réseaux</Text>
          <Text style={styles.headerSub}>{identity.cid_short}</Text>
        </View>
      </View>

      {loading ? (
        <View style={styles.center}>
          <ActivityIndicator size="large" color="#4f46e5" />
        </View>
      ) : error ? (
        <View style={styles.center}>
          <Text style={styles.errorText}>{error}</Text>
          <TouchableOpacity onPress={loadNetworks} style={styles.retryBtn}>
            <Text style={styles.retryBtnText}>Réessayer</Text>
          </TouchableOpacity>
        </View>
      ) : (
        <FlatList
          data={networks}
          keyExtractor={(n) => n.cid_short}
          renderItem={renderNetwork}
          contentContainerStyle={networks.length === 0 ? styles.emptyContainer : styles.listContent}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor="#4f46e5" />
          }
          ListEmptyComponent={
            <View style={styles.emptyInner}>
              <Text style={styles.emptyTitle}>Aucun réseau</Text>
              <Text style={styles.emptyHint}>
                Connectez-vous à votre app desktop ou rejoignez un réseau via un lien d'invitation
                pour voir vos réseaux apparaître ici.
              </Text>
            </View>
          }
        />
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#f8fafc' },
  header: {
    backgroundColor: '#fff',
    paddingTop: 56,
    paddingBottom: 16,
    paddingHorizontal: 20,
    borderBottomWidth: 1,
    borderBottomColor: '#e2e8f0',
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-end',
  },
  headerTitle: { fontSize: 24, fontWeight: '700', color: '#1e1b4b' },
  headerSub:   { fontSize: 12, color: '#94a3b8', marginTop: 2, fontFamily: 'monospace' },
  center:      { flex: 1, justifyContent: 'center', alignItems: 'center', padding: 24 },
  listContent: { padding: 16, gap: 10 },
  card: {
    backgroundColor: '#fff',
    borderRadius: 14,
    padding: 16,
    flexDirection: 'row',
    alignItems: 'center',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.06,
    shadowRadius: 4,
    elevation: 2,
  },
  cardAvatar: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: '#ede9fe',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  cardAvatarText: { fontSize: 16, fontWeight: '700', color: '#4f46e5' },
  cardBody:  { flex: 1 },
  cardName:  { fontSize: 16, fontWeight: '600', color: '#1e293b' },
  cardMeta:  { fontSize: 13, color: '#94a3b8', marginTop: 2 },
  cardCid:   { fontFamily: 'monospace', fontSize: 11 },
  chevron:   { fontSize: 22, color: '#c7d2fe' },
  emptyContainer: { flex: 1 },
  emptyInner: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 32,
    marginTop: 80,
  },
  emptyTitle:  { fontSize: 18, fontWeight: '600', color: '#334155', marginBottom: 12 },
  emptyHint:   { fontSize: 14, color: '#94a3b8', textAlign: 'center', lineHeight: 22 },
  errorText:   { color: '#dc2626', fontSize: 14, textAlign: 'center', marginBottom: 16 },
  retryBtn:    { backgroundColor: '#ede9fe', paddingVertical: 10, paddingHorizontal: 20, borderRadius: 8 },
  retryBtnText: { color: '#4f46e5', fontWeight: '600' },
});
