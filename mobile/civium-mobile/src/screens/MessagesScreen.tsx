import React, { useCallback, useEffect, useRef, useState } from 'react';
import {
  ActivityIndicator,
  FlatList,
  KeyboardAvoidingView,
  Platform,
  StyleSheet,
  Text,
  TextInput,
  TouchableOpacity,
  View,
} from 'react-native';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import type { MessageInfo, RootStackParamList } from '../types';
import Civium, { withDataDir } from '../native/CiviumModule';

type Props = NativeStackScreenProps<RootStackParamList, 'Messages'>;

function formatTime(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString('fr-FR', {
    hour: '2-digit',
    minute: '2-digit',
  });
}

export default function MessagesScreen({ route }: Props) {
  const { identity, network } = route.params;

  const [messages, setMessages]   = useState<MessageInfo[]>([]);
  const [loading, setLoading]     = useState(true);
  const [body, setBody]           = useState('');
  const [sending, setSending]     = useState(false);
  const [error, setError]         = useState<string | null>(null);
  const listRef = useRef<FlatList>(null);

  const loadMessages = useCallback(async () => {
    setError(null);
    try {
      const msgs = await withDataDir((dir) =>
        Civium.messageList(dir, network.cid_short),
      );
      setMessages(msgs);
      setTimeout(() => listRef.current?.scrollToEnd({ animated: false }), 50);
    } catch (e) {
      setError(String(e));
    }
  }, [network.cid_short]);

  useEffect(() => {
    loadMessages().finally(() => setLoading(false));
    // Refresh every 5 s for offline sync visibility
    const interval = setInterval(loadMessages, 5000);
    return () => clearInterval(interval);
  }, [loadMessages]);

  async function handleSend() {
    const text = body.trim();
    if (!text || sending) return;
    setSending(true);
    setError(null);
    try {
      const msg = await withDataDir((dir) =>
        Civium.messageSend(dir, network.cid_short, text),
      );
      setMessages((prev) => [...prev, msg]);
      setBody('');
      setTimeout(() => listRef.current?.scrollToEnd({ animated: true }), 50);
    } catch (e) {
      setError(String(e));
    } finally {
      setSending(false);
    }
  }

  function renderMessage({ item }: { item: MessageInfo }) {
    const isMine = item.author_cid_short === identity.cid_short;
    return (
      <View style={[styles.bubble, isMine ? styles.bubbleMine : styles.bubbleOther]}>
        {!isMine && (
          <Text style={styles.bubbleAuthor}>{item.author_cid_short}</Text>
        )}
        <Text style={[styles.bubbleText, isMine && styles.bubbleTextMine]}>
          {item.body}
        </Text>
        <Text style={[styles.bubbleTime, isMine && styles.bubbleTimeMine]}>
          {formatTime(item.sent_at)}
        </Text>
      </View>
    );
  }

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
      keyboardVerticalOffset={Platform.OS === 'ios' ? 88 : 0}
    >
      {loading ? (
        <View style={styles.center}>
          <ActivityIndicator size="large" color="#4f46e5" />
        </View>
      ) : (
        <FlatList
          ref={listRef}
          data={messages}
          keyExtractor={(m) => m.id}
          renderItem={renderMessage}
          contentContainerStyle={styles.listContent}
          onContentSizeChange={() => listRef.current?.scrollToEnd({ animated: false })}
          ListEmptyComponent={
            <View style={styles.emptyInner}>
              <Text style={styles.emptyHint}>
                Aucun message. Envoyez le premier !
              </Text>
            </View>
          }
        />
      )}

      {error && (
        <View style={styles.errorBar}>
          <Text style={styles.errorText}>{error}</Text>
        </View>
      )}

      <View style={styles.composer}>
        <TextInput
          style={styles.input}
          value={body}
          onChangeText={setBody}
          placeholder="Message…"
          placeholderTextColor="#94a3b8"
          multiline
          maxLength={4000}
          returnKeyType="send"
          onSubmitEditing={handleSend}
          blurOnSubmit={false}
        />
        <TouchableOpacity
          style={[styles.sendBtn, (!body.trim() || sending) && styles.sendBtnDisabled]}
          onPress={handleSend}
          disabled={!body.trim() || sending}
        >
          {sending ? (
            <ActivityIndicator size="small" color="#fff" />
          ) : (
            <Text style={styles.sendBtnText}>↑</Text>
          )}
        </TouchableOpacity>
      </View>
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container:   { flex: 1, backgroundColor: '#f1f5f9' },
  center:      { flex: 1, justifyContent: 'center', alignItems: 'center' },
  listContent: { padding: 16, gap: 6, paddingBottom: 8 },
  bubble: {
    maxWidth: '78%',
    borderRadius: 16,
    paddingVertical: 8,
    paddingHorizontal: 12,
    marginVertical: 2,
  },
  bubbleMine: {
    backgroundColor: '#4f46e5',
    alignSelf: 'flex-end',
    borderBottomRightRadius: 4,
  },
  bubbleOther: {
    backgroundColor: '#fff',
    alignSelf: 'flex-start',
    borderBottomLeftRadius: 4,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.05,
    shadowRadius: 2,
    elevation: 1,
  },
  bubbleAuthor: { fontSize: 11, color: '#6366f1', fontWeight: '600', marginBottom: 2 },
  bubbleText:       { fontSize: 15, color: '#1e293b', lineHeight: 20 },
  bubbleTextMine:   { color: '#fff' },
  bubbleTime:       { fontSize: 11, color: '#94a3b8', marginTop: 4, alignSelf: 'flex-end' },
  bubbleTimeMine:   { color: 'rgba(255,255,255,0.6)' },
  emptyInner: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    marginTop: 120,
  },
  emptyHint:  { color: '#94a3b8', fontSize: 15 },
  errorBar:   {
    backgroundColor: '#fef2f2',
    paddingVertical: 8,
    paddingHorizontal: 16,
  },
  errorText:  { color: '#dc2626', fontSize: 13 },
  composer: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    backgroundColor: '#fff',
    borderTopWidth: 1,
    borderTopColor: '#e2e8f0',
    paddingHorizontal: 12,
    paddingVertical: 8,
    gap: 8,
  },
  input: {
    flex: 1,
    borderWidth: 1,
    borderColor: '#e2e8f0',
    borderRadius: 20,
    paddingHorizontal: 14,
    paddingVertical: Platform.OS === 'ios' ? 10 : 8,
    fontSize: 15,
    color: '#1e293b',
    maxHeight: 120,
    backgroundColor: '#f8fafc',
  },
  sendBtn: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#4f46e5',
    justifyContent: 'center',
    alignItems: 'center',
  },
  sendBtnDisabled: { backgroundColor: '#c7d2fe' },
  sendBtnText:     { color: '#fff', fontSize: 20, lineHeight: 22 },
});
