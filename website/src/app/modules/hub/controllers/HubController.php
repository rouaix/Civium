<?php

/**
 * Civium Hub — nœud serveur PHP
 *
 * Le hub permet à un serveur PHP de jouer le rôle de point de synchronisation
 * permanent pour des réseaux Civium. Les clients desktop / mobile s'y connectent
 * via HTTP au lieu du P2P libp2p lorsque le P2P n'est pas disponible.
 *
 * Tous les payloads de messages restent chiffrés côté client — le hub ne voit
 * jamais le contenu en clair.
 *
 * Endpoints :
 *   GET  /hub/status                → statut + clé publique du hub
 *   POST /hub/network/register      → enregistrer un réseau (signé par l'admin)
 *   GET  /hub/network/list          → liste des réseaux publics
 *   POST /hub/member/join           → un membre rejoint un réseau (signé par le membre)
 *   POST /hub/sync/push             → le client pousse des messages (signé)
 *   GET  /hub/sync/pull             → le client tire les messages depuis un timestamp (signé)
 *
 * Authentification : signature Ed25519 vérifiée sur chaque opération sensible.
 * Le hub conserve la clé publique de chaque membre enregistré.
 */
class HubController
{
    protected Base $f3;

    public function __construct()
    {
        $this->f3 = Base::instance();
    }

    private function db(): ?\DB\SQL
    {
        return $this->f3->get('DB') ?: null;
    }

    // ── Identité du hub ───────────────────────────────────────────────────────

    /**
     * Retourne (ou crée) l'identité Ed25519 du hub.
     * Stockée dans hub_identity (une seule ligne, id=1).
     */
    private function hubIdentity(): array
    {
        $db = $this->db();
        if (!$db) return [];

        $rows = $db->exec("SELECT pubkey_b58, secret_b58 FROM hub_identity WHERE id = 1");
        if ($rows && !empty($rows[0])) {
            return $rows[0];
        }

        // Première utilisation : générer une paire de clés Ed25519
        if (!function_exists('sodium_crypto_sign_keypair')) {
            return [];
        }

        $kp     = sodium_crypto_sign_keypair();
        $secret = sodium_crypto_sign_secretkey($kp); // 64 bytes (secret || public)
        $pub    = sodium_crypto_sign_publickey($kp);  // 32 bytes

        // On stocke uniquement les 32 premiers octets du secret (seed)
        $secretSeed = substr($secret, 0, 32);

        $pubB58    = self::base58Encode($pub);
        $secretB58 = self::base58Encode($secretSeed);

        $db->exec(
            "INSERT INTO hub_identity (id, pubkey_b58, secret_b58) VALUES (1, ?, ?)",
            [$pubB58, $secretB58]
        );

        return ['pubkey_b58' => $pubB58, 'secret_b58' => $secretB58];
    }

    // ── POST /admin/hub/reset-identity ───────────────────────────────────────

    public function adminResetIdentity(): void
    {
        header('Content-Type: application/json');

        $token       = (string) ($this->f3->get('SERVER.HTTP_X_ADMIN_TOKEN') ?: $this->f3->get('GET.token') ?: '');
        $configToken = (string) $this->f3->get('ADMIN_TOKEN');
        if (!$configToken || !hash_equals($configToken, $token)) {
            http_response_code(401);
            echo json_encode(['error' => 'non_authentifie']);
            return;
        }

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $db->exec("DELETE FROM hub_identity WHERE id = 1");
        $id = $this->hubIdentity();

        echo json_encode(['status' => 'ok', 'hub_pubkey' => $id['pubkey_b58'] ?? null]);
    }

    // ── GET /hub/status ───────────────────────────────────────────────────────

    public function status(): void
    {
        header('Content-Type: application/json');
        $id   = $this->hubIdentity();
        $main = $this->ensureMainNetwork();
        echo json_encode([
            'status'           => 'ok',
            'version'          => '1',
            'hub_pubkey'       => $id['pubkey_b58'] ?? null,
            'main_network_cid' => $main['network_cid'] ?? null,
            'app_url'          => rtrim((string) $this->f3->get('APP_URL'), '/'),
        ]);
    }

    // ── GET /hub/main-network ─────────────────────────────────────────────────

    public function mainNetwork(): void
    {
        header('Content-Type: application/json');
        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $main = $this->ensureMainNetwork();
        if (!$main) { $this->serviceUnavailable(); return; }

        echo json_encode([
            'network_cid'  => $main['network_cid'],
            'network_name' => $main['network_name'],
            'pubkey_b58'   => $main['pubkey_b58'],
        ]);
    }

    // ── GET /hub/network/public ───────────────────────────────────────────────

    public function networkPublicList(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $rows = $db->exec(
            "SELECT network_cid, network_name, admin_cid, is_public, parent_network_cid, created_at
             FROM hub_networks
             WHERE is_public = 1
             ORDER BY created_at ASC"
        ) ?: [];

        // Toujours inclure le réseau principal en premier
        $main = $this->ensureMainNetwork();
        if ($main) {
            $mainCid = $main['network_cid'];
            $rows    = array_filter($rows, fn($r) => $r['network_cid'] !== $mainCid);
            array_unshift($rows, [
                'network_cid'  => $mainCid,
                'network_name' => $main['network_name'],
                'admin_cid'    => null,
                'is_public'    => 1,
                'is_main'      => true,
                'created_at'   => $main['created_at'] ?? null,
            ]);
        }

        echo json_encode(['networks' => array_values($rows)]);
    }

    // ── Réseau principal : création automatique ───────────────────────────────

    /**
     * Crée le réseau principal "Civium" si inexistant (appelé à chaque démarrage).
     * Utilise la clé du hub pour signer l'enregistrement.
     * Retourne les infos du réseau principal (cid, name, pubkey_b58).
     */
    private function ensureMainNetwork(): ?array
    {
        $db = $this->db();
        if (!$db) return null;

        // Déjà initialisé ?
        $rows = $db->exec("SELECT network_cid, network_name, pubkey_b58, created_at FROM hub_main_network WHERE id = 1");
        if ($rows && !empty($rows[0])) {
            return $rows[0];
        }

        // Générer un keypair pour le réseau principal
        if (!function_exists('sodium_crypto_sign_seed_keypair')) return null;

        $seed      = random_bytes(32);
        $kp        = sodium_crypto_sign_seed_keypair($seed);
        $pub       = sodium_crypto_sign_publickey($kp);
        $secretB58 = self::base58Encode($seed);
        $pubB58    = self::base58Encode($pub);

        // CID = "civ1" + base58(sha256(pubkey)) — approx. de la dérivation blake3 desktop
        $networkCid = 'civ1' . self::base58Encode(hash('sha256', $pub, true));
        $networkName = 'Civium';

        // Identité hub = admin du réseau principal
        $hubId    = $this->hubIdentity();
        $hubPub   = $hubId['pubkey_b58'] ?? '';
        $hubCid   = 'civ1' . self::base58Encode(hash('sha256', self::base58Decode($hubPub), true));
        $ts       = time();
        $canonical = "register|{$networkCid}|{$networkName}|{$hubCid}|{$ts}";

        // Signer avec la clé du hub
        $hubSeed   = self::base58Decode($hubId['secret_b58'] ?? '');
        $hubKp     = sodium_crypto_sign_seed_keypair($hubSeed);
        $sigBytes  = sodium_crypto_sign_detached($canonical, sodium_crypto_sign_secretkey($hubKp));
        $sig       = self::base58Encode($sigBytes);

        try {
            $db->begin();

            $db->exec(
                "INSERT IGNORE INTO hub_networks (network_cid, network_name, admin_cid, admin_pubkey, is_public)
                 VALUES (?, ?, ?, ?, 1)",
                [$networkCid, $networkName, $hubCid, $hubPub]
            );

            // L'admin (hub) se rejoint automatiquement
            $db->exec(
                "INSERT IGNORE INTO hub_members (network_cid, member_cid, display_name, pubkey_b58)
                 VALUES (?, ?, ?, ?)",
                [$networkCid, $hubCid, 'Civium Hub', $hubPub]
            );

            $db->exec(
                "INSERT INTO hub_main_network (id, network_cid, network_name, secret_b58, pubkey_b58)
                 VALUES (1, ?, ?, ?, ?)",
                [$networkCid, $networkName, $secretB58, $pubB58]
            );

            $db->commit();
            error_log("[Hub] Réseau principal Civium créé : {$networkCid}");

            return ['network_cid' => $networkCid, 'network_name' => $networkName, 'pubkey_b58' => $pubB58, 'created_at' => date('Y-m-d H:i:s')];

        } catch (\Exception $e) {
            $db->rollback();
            error_log('[Hub ensureMainNetwork] ' . $e->getMessage());
            return null;
        }
    }

    // ── POST /hub/network/register ────────────────────────────────────────────

    public function networkRegister(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body        = $this->jsonBody();
        $networkCid  = trim((string) ($body['network_cid']  ?? ''));
        $networkName = trim((string) ($body['network_name'] ?? ''));
        $adminCid    = trim((string) ($body['admin_cid']    ?? ''));
        $adminPubkey = trim((string) ($body['admin_pubkey'] ?? ''));
        $timestamp   = (int) ($body['timestamp'] ?? 0);
        $signature   = trim((string) ($body['signature']    ?? ''));
        $isPublic        = (int) (bool) ($body['is_public'] ?? false);
        $parentNetworkCid = trim((string) ($body['parent_network_cid'] ?? '')) ?: null;

        if (!$networkCid || !$networkName || !$adminCid || !$adminPubkey || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        // Vérifier la signature Ed25519 de l'admin
        $canonical = "register|{$networkCid}|{$networkName}|{$adminCid}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $adminPubkey)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        // Vérifier que le timestamp n'est pas trop vieux (5 minutes)
        if (abs(time() - $timestamp) > 300) {
            http_response_code(403);
            echo json_encode(['error' => 'timestamp_expire']);
            return;
        }

        try {
            $db->exec(
                "INSERT INTO hub_networks (network_cid, network_name, admin_cid, admin_pubkey, is_public, parent_network_cid)
                 VALUES (?, ?, ?, ?, ?, ?)
                 ON DUPLICATE KEY UPDATE
                     network_name       = VALUES(network_name),
                     admin_pubkey       = VALUES(admin_pubkey),
                     is_public          = VALUES(is_public),
                     parent_network_cid = VALUES(parent_network_cid)",
                [$networkCid, $networkName, $adminCid, $adminPubkey, $isPublic, $parentNetworkCid]
            );

            // Enregistrer automatiquement l'admin comme membre
            $db->exec(
                "INSERT IGNORE INTO hub_members (network_cid, member_cid, display_name, pubkey_b58)
                 VALUES (?, ?, ?, ?)",
                [$networkCid, $adminCid, 'Admin', $adminPubkey]
            );
        } catch (\Exception $e) {
            error_log('[Hub networkRegister] ' . $e->getMessage());
            http_response_code(500);
            echo json_encode(['error' => 'database_error']);
            return;
        }

        http_response_code(201);
        echo json_encode(['status' => 'registered', 'network_cid' => $networkCid]);
    }

    // ── GET /hub/network/list ─────────────────────────────────────────────────

    public function networkList(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $rows = $db->exec(
            "SELECT network_cid, network_name, admin_cid, is_public, parent_network_cid, created_at FROM hub_networks ORDER BY created_at DESC"
        ) ?: [];

        echo json_encode(['networks' => $rows]);
    }

    // ── GET /hub/member/list ──────────────────────────────────────────────────

    public function memberList(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $networkCid = trim((string) ($this->f3->get('GET.network_cid') ?? ''));
        if (!$networkCid) {
            http_response_code(422);
            echo json_encode(['error' => 'network_cid_requis']);
            return;
        }

        $rows = $db->exec(
            "SELECT member_cid, display_name, joined_at FROM hub_members WHERE network_cid = ? ORDER BY joined_at ASC",
            [$networkCid]
        ) ?: [];

        echo json_encode(['members' => $rows]);
    }

    // ── POST /hub/member/join ─────────────────────────────────────────────────

    public function memberJoin(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body        = $this->jsonBody();
        $networkCid  = trim((string) ($body['network_cid']   ?? ''));
        $memberCid   = trim((string) ($body['member_cid']    ?? ''));
        $displayName = trim((string) ($body['display_name']  ?? ''));
        $pubkey      = trim((string) ($body['pubkey_b58']    ?? ''));
        $timestamp   = (int) ($body['timestamp'] ?? 0);
        $signature   = trim((string) ($body['signature']     ?? ''));

        if (!$networkCid || !$memberCid || !$pubkey || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        // Vérifier que le réseau existe
        $net = $db->exec("SELECT 1 FROM hub_networks WHERE network_cid = ?", [$networkCid]);
        if (!$net || empty($net[0])) {
            http_response_code(404);
            echo json_encode(['error' => 'reseau_inconnu']);
            return;
        }

        // Vérifier la signature
        $canonical = "join|{$networkCid}|{$memberCid}|{$displayName}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        if (abs(time() - $timestamp) > 300) {
            http_response_code(403);
            echo json_encode(['error' => 'timestamp_expire']);
            return;
        }

        try {
            $db->exec(
                "INSERT INTO hub_members (network_cid, member_cid, display_name, pubkey_b58)
                 VALUES (?, ?, ?, ?)
                 ON DUPLICATE KEY UPDATE display_name = VALUES(display_name), pubkey_b58 = VALUES(pubkey_b58)",
                [$networkCid, $memberCid, $displayName, $pubkey]
            );
        } catch (\Exception $e) {
            error_log('[Hub memberJoin] ' . $e->getMessage());
            http_response_code(500);
            echo json_encode(['error' => 'database_error']);
            return;
        }

        // Notifier les membres existants de l'arrivée du nouveau membre
        $this->notify($db, $networkCid, 'member_joined', "{$displayName} a rejoint le réseau", $memberCid);

        http_response_code(201);
        echo json_encode(['status' => 'joined', 'network_cid' => $networkCid, 'member_cid' => $memberCid]);
    }

    // ── POST /hub/sync/push ───────────────────────────────────────────────────

    public function push(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body       = $this->jsonBody();
        $networkCid = trim((string) ($body['network_cid'] ?? ''));
        $memberCid  = trim((string) ($body['member_cid']  ?? ''));
        $timestamp  = (int) ($body['timestamp'] ?? 0);
        $signature  = trim((string) ($body['signature']   ?? ''));
        $messages   = (array) ($body['messages'] ?? []);

        if (!$networkCid || !$memberCid || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        // Récupérer la clé publique du membre
        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) {
            http_response_code(403);
            echo json_encode(['error' => 'membre_non_enregistre']);
            return;
        }

        $canonical = "push|{$networkCid}|{$memberCid}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        if (abs(time() - $timestamp) > 300) {
            http_response_code(403);
            echo json_encode(['error' => 'timestamp_expire']);
            return;
        }

        $stored = 0;
        foreach ($messages as $msg) {
            $messageId = (string) ($msg['id'] ?? '');
            if (!$messageId) continue;

            try {
                $db->exec(
                    "INSERT IGNORE INTO hub_messages (network_cid, message_id, sender_cid, payload_json)
                     VALUES (?, ?, ?, ?)",
                    [$networkCid, $messageId, $memberCid, json_encode($msg)]
                );
                $stored++;
            } catch (\Exception $e) {
                error_log('[Hub push] ' . $e->getMessage());
            }
        }

        if ($stored > 0) {
            $senderName = $this->getMemberDisplayName($db, $networkCid, $memberCid) ?? $memberCid;
            $this->notify($db, $networkCid, 'message', "Nouveau message de {$senderName}", $memberCid);
        }

        echo json_encode(['status' => 'ok', 'stored' => $stored]);
    }

    // ── GET /hub/sync/pull ────────────────────────────────────────────────────

    public function pull(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $networkCid = trim((string) ($this->f3->get('GET.network_cid') ?? ''));
        $memberCid  = trim((string) ($this->f3->get('GET.member_cid')  ?? ''));
        $since      = (int) ($this->f3->get('GET.since')     ?? 0);
        $timestamp  = (int) ($this->f3->get('GET.timestamp') ?? 0);
        $signature  = trim((string) ($this->f3->get('GET.signature')   ?? ''));

        if (!$networkCid || !$memberCid || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) {
            http_response_code(403);
            echo json_encode(['error' => 'membre_non_enregistre']);
            return;
        }

        $canonical = "pull|{$networkCid}|{$memberCid}|{$since}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        if (abs(time() - $timestamp) > 300) {
            http_response_code(403);
            echo json_encode(['error' => 'timestamp_expire']);
            return;
        }

        $sinceDate = date('Y-m-d H:i:s', $since);
        $rows = $db->exec(
            "SELECT message_id, sender_cid, payload_json, received_at
             FROM hub_messages
             WHERE network_cid = ? AND received_at > ?
             ORDER BY received_at ASC
             LIMIT 500",
            [$networkCid, $sinceDate]
        ) ?: [];

        $messages = array_map(function ($row) {
            $payload = json_decode($row['payload_json'], true) ?? [];
            $payload['_hub_received_at'] = $row['received_at'];
            return $payload;
        }, $rows);

        echo json_encode([
            'network_cid' => $networkCid,
            'messages'    => $messages,
            'count'       => count($messages),
        ]);
    }

    // ── POST /hub/agenda/event ────────────────────────────────────────────────

    public function agendaCreate(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body        = $this->jsonBody();
        $networkCid  = trim((string) ($body['network_cid']  ?? ''));
        $memberCid   = trim((string) ($body['member_cid']   ?? ''));
        $title       = trim((string) ($body['title']        ?? ''));
        $description = trim((string) ($body['description']  ?? ''));
        $startAt     = trim((string) ($body['start_at']     ?? ''));
        $endAt       = trim((string) ($body['end_at']       ?? ''));
        $timestamp   = (int) ($body['timestamp'] ?? 0);
        $signature   = trim((string) ($body['signature']    ?? ''));

        if (!$networkCid || !$memberCid || !$title || !$startAt || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) {
            http_response_code(403);
            echo json_encode(['error' => 'membre_non_enregistre']);
            return;
        }

        $canonical = "agenda|{$networkCid}|{$memberCid}|{$title}|{$startAt}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        if (abs(time() - $timestamp) > 300) {
            http_response_code(403);
            echo json_encode(['error' => 'timestamp_expire']);
            return;
        }

        $id = sprintf('%04x%04x-%04x-%04x-%04x-%04x%04x%04x',
            mt_rand(0, 0xffff), mt_rand(0, 0xffff),
            mt_rand(0, 0xffff),
            mt_rand(0, 0x0fff) | 0x4000,
            mt_rand(0, 0x3fff) | 0x8000,
            mt_rand(0, 0xffff), mt_rand(0, 0xffff), mt_rand(0, 0xffff)
        );

        // Valider le format des dates
        $startDt = date('Y-m-d H:i:s', strtotime($startAt));
        $endDt   = $endAt ? date('Y-m-d H:i:s', strtotime($endAt)) : null;

        try {
            $db->exec(
                "INSERT INTO hub_agenda_events (id, network_cid, author_cid, title, description, start_at, end_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
                [$id, $networkCid, $memberCid, $title, $description ?: null, $startDt, $endDt]
            );
        } catch (\Exception $e) {
            error_log('[Hub agendaCreate] ' . $e->getMessage());
            http_response_code(500);
            echo json_encode(['error' => 'database_error']);
            return;
        }

        http_response_code(201);
        echo json_encode(['status' => 'created', 'event_id' => $id]);
    }

    // ── GET /hub/agenda/events ─────────────────────────────────────────────────

    public function agendaList(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $networkCid = trim((string) ($this->f3->get('GET.network_cid') ?? ''));
        $memberCid  = trim((string) ($this->f3->get('GET.member_cid')  ?? ''));
        $timestamp  = (int) ($this->f3->get('GET.timestamp') ?? 0);
        $signature  = trim((string) ($this->f3->get('GET.signature')   ?? ''));

        if (!$networkCid || !$memberCid || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) {
            http_response_code(403);
            echo json_encode(['error' => 'membre_non_enregistre']);
            return;
        }

        $canonical = "agenda_list|{$networkCid}|{$memberCid}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        $events = $db->exec(
            "SELECT id, author_cid, title, description, start_at, end_at, created_at
             FROM hub_agenda_events
             WHERE network_cid = ?
             ORDER BY start_at ASC
             LIMIT 200",
            [$networkCid]
        ) ?: [];

        echo json_encode(['events' => $events]);
    }

    // ── POST /hub/governance/proposal ────────────────────────────────────────

    public function proposalCreate(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body        = $this->jsonBody();
        $networkCid  = trim((string) ($body['network_cid']  ?? ''));
        $memberCid   = trim((string) ($body['member_cid']   ?? ''));
        $title       = trim((string) ($body['title']        ?? ''));
        $description = trim((string) ($body['description']  ?? ''));
        $timestamp   = (int) ($body['timestamp'] ?? 0);
        $signature   = trim((string) ($body['signature']    ?? ''));

        if (!$networkCid || !$memberCid || !$title || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) {
            http_response_code(403);
            echo json_encode(['error' => 'membre_non_enregistre']);
            return;
        }

        $canonical = "proposal|{$networkCid}|{$memberCid}|{$title}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        if (abs(time() - $timestamp) > 300) {
            http_response_code(403);
            echo json_encode(['error' => 'timestamp_expire']);
            return;
        }

        $id = sprintf('%04x%04x-%04x-%04x-%04x-%04x%04x%04x',
            mt_rand(0, 0xffff), mt_rand(0, 0xffff),
            mt_rand(0, 0xffff),
            mt_rand(0, 0x0fff) | 0x4000,
            mt_rand(0, 0x3fff) | 0x8000,
            mt_rand(0, 0xffff), mt_rand(0, 0xffff), mt_rand(0, 0xffff)
        );

        try {
            $db->exec(
                "INSERT INTO hub_proposals (id, network_cid, author_cid, title, description)
                 VALUES (?, ?, ?, ?, ?)",
                [$id, $networkCid, $memberCid, $title, $description]
            );
        } catch (\Exception $e) {
            error_log('[Hub proposalCreate] ' . $e->getMessage());
            http_response_code(500);
            echo json_encode(['error' => 'database_error']);
            return;
        }

        http_response_code(201);
        echo json_encode(['status' => 'created', 'proposal_id' => $id]);
    }

    // ── GET /hub/governance/proposals ─────────────────────────────────────────

    public function proposalList(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $networkCid = trim((string) ($this->f3->get('GET.network_cid') ?? ''));
        $memberCid  = trim((string) ($this->f3->get('GET.member_cid')  ?? ''));
        $timestamp  = (int) ($this->f3->get('GET.timestamp') ?? 0);
        $signature  = trim((string) ($this->f3->get('GET.signature')   ?? ''));

        if (!$networkCid || !$memberCid || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) {
            http_response_code(403);
            echo json_encode(['error' => 'membre_non_enregistre']);
            return;
        }

        $canonical = "proposals|{$networkCid}|{$memberCid}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        $proposals = $db->exec(
            "SELECT p.id, p.author_cid, p.title, p.description, p.status, p.created_at,
                    COUNT(v.id) AS vote_count,
                    SUM(CASE WHEN v.choice = 'yes' THEN 1 ELSE 0 END) AS yes_count,
                    SUM(CASE WHEN v.choice = 'no'  THEN 1 ELSE 0 END) AS no_count,
                    SUM(CASE WHEN v.choice = 'abstain' THEN 1 ELSE 0 END) AS abstain_count,
                    MAX(CASE WHEN v.voter_cid = ? THEN v.choice ELSE NULL END) AS my_vote
             FROM hub_proposals p
             LEFT JOIN hub_votes v ON v.proposal_id = p.id
             WHERE p.network_cid = ?
             GROUP BY p.id
             ORDER BY p.created_at DESC
             LIMIT 100",
            [$memberCid, $networkCid]
        ) ?: [];

        echo json_encode(['proposals' => $proposals]);
    }

    // ── POST /hub/governance/vote ─────────────────────────────────────────────

    public function voteCast(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body       = $this->jsonBody();
        $proposalId = trim((string) ($body['proposal_id'] ?? ''));
        $networkCid = trim((string) ($body['network_cid'] ?? ''));
        $memberCid  = trim((string) ($body['member_cid']  ?? ''));
        $choice     = trim((string) ($body['choice']      ?? ''));
        $timestamp  = (int) ($body['timestamp'] ?? 0);
        $signature  = trim((string) ($body['signature']   ?? ''));

        if (!$proposalId || !$networkCid || !$memberCid || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        if (!in_array($choice, ['yes', 'no', 'abstain'], true)) {
            http_response_code(422);
            echo json_encode(['error' => 'choix_invalide']);
            return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) {
            http_response_code(403);
            echo json_encode(['error' => 'membre_non_enregistre']);
            return;
        }

        $canonical = "vote|{$proposalId}|{$networkCid}|{$memberCid}|{$choice}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        if (abs(time() - $timestamp) > 300) {
            http_response_code(403);
            echo json_encode(['error' => 'timestamp_expire']);
            return;
        }

        // Vérifier que la proposition existe et est ouverte
        $prop = $db->exec(
            "SELECT id, status FROM hub_proposals WHERE id = ? AND network_cid = ?",
            [$proposalId, $networkCid]
        );
        if (!$prop || empty($prop[0])) {
            http_response_code(404);
            echo json_encode(['error' => 'proposition_introuvable']);
            return;
        }
        if ($prop[0]['status'] !== 'open') {
            http_response_code(409);
            echo json_encode(['error' => 'proposition_fermee']);
            return;
        }

        $voteId = sprintf('%04x%04x-%04x-%04x-%04x-%04x%04x%04x',
            mt_rand(0, 0xffff), mt_rand(0, 0xffff),
            mt_rand(0, 0xffff),
            mt_rand(0, 0x0fff) | 0x4000,
            mt_rand(0, 0x3fff) | 0x8000,
            mt_rand(0, 0xffff), mt_rand(0, 0xffff), mt_rand(0, 0xffff)
        );

        try {
            $db->exec(
                "INSERT INTO hub_votes (id, proposal_id, network_cid, voter_cid, choice)
                 VALUES (?, ?, ?, ?, ?)
                 ON DUPLICATE KEY UPDATE choice = VALUES(choice), voted_at = NOW()",
                [$voteId, $proposalId, $networkCid, $memberCid, $choice]
            );
        } catch (\Exception $e) {
            error_log('[Hub voteCast] ' . $e->getMessage());
            http_response_code(500);
            echo json_encode(['error' => 'database_error']);
            return;
        }

        echo json_encode(['status' => 'voted', 'choice' => $choice]);
    }

    // ── POST /hub/document ────────────────────────────────────────────────────

    public function documentCreate(): void
    {
        header('Content-Type: application/json');
        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body       = $this->jsonBody();
        $networkCid = trim((string) ($body['network_cid'] ?? ''));
        $memberCid  = trim((string) ($body['member_cid']  ?? ''));
        $timestamp  = (int) ($body['timestamp'] ?? 0);
        $signature  = trim((string) ($body['signature']   ?? ''));
        $title      = trim((string) ($body['title']       ?? ''));
        $content    = (string) ($body['content'] ?? '');

        if (!$networkCid || !$memberCid || !$timestamp || !$signature || !$title) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) { http_response_code(403); echo json_encode(['error' => 'membre_non_enregistre']); return; }

        $canonical = "document_create|{$networkCid}|{$memberCid}|{$title}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403); echo json_encode(['error' => 'signature_invalide']); return;
        }
        if (abs(time() - $timestamp) > 300) {
            http_response_code(403); echo json_encode(['error' => 'timestamp_expire']); return;
        }

        $id = $this->uuidv4();
        try {
            $db->exec(
                "INSERT INTO hub_documents (id, network_cid, author_cid, title, content, last_edited_by)
                 VALUES (?, ?, ?, ?, ?, ?)",
                [$id, $networkCid, $memberCid, $title, $content, $memberCid]
            );
        } catch (\Exception $e) {
            error_log('[Hub documentCreate] ' . $e->getMessage());
            http_response_code(500); echo json_encode(['error' => 'database_error']); return;
        }

        $this->notify($db, $networkCid, 'document', "Nouveau document : {$title}", $memberCid);
        http_response_code(201);
        echo json_encode(['status' => 'created', 'id' => $id]);
    }

    // ── PUT /hub/document ─────────────────────────────────────────────────────

    public function documentUpdate(): void
    {
        header('Content-Type: application/json');
        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body       = $this->jsonBody();
        $networkCid = trim((string) ($body['network_cid'] ?? ''));
        $memberCid  = trim((string) ($body['member_cid']  ?? ''));
        $docId      = trim((string) ($body['id']          ?? ''));
        $timestamp  = (int) ($body['timestamp'] ?? 0);
        $signature  = trim((string) ($body['signature']   ?? ''));
        $title      = trim((string) ($body['title']       ?? ''));
        $content    = (string) ($body['content'] ?? '');

        if (!$networkCid || !$memberCid || !$docId || !$timestamp || !$signature || !$title) {
            http_response_code(422); echo json_encode(['error' => 'champs_manquants']); return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) { http_response_code(403); echo json_encode(['error' => 'membre_non_enregistre']); return; }

        $canonical = "document_update|{$networkCid}|{$memberCid}|{$docId}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403); echo json_encode(['error' => 'signature_invalide']); return;
        }
        if (abs(time() - $timestamp) > 300) {
            http_response_code(403); echo json_encode(['error' => 'timestamp_expire']); return;
        }

        try {
            $db->exec(
                "UPDATE hub_documents SET title = ?, content = ?, last_edited_by = ?, updated_at = NOW()
                 WHERE id = ? AND network_cid = ?",
                [$title, $content, $memberCid, $docId, $networkCid]
            );
        } catch (\Exception $e) {
            error_log('[Hub documentUpdate] ' . $e->getMessage());
            http_response_code(500); echo json_encode(['error' => 'database_error']); return;
        }

        echo json_encode(['status' => 'updated']);
    }

    // ── GET /hub/documents ────────────────────────────────────────────────────

    public function documentList(): void
    {
        header('Content-Type: application/json');
        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $networkCid = trim((string) ($this->f3->get('GET.network_cid') ?? ''));
        $memberCid  = trim((string) ($this->f3->get('GET.member_cid')  ?? ''));
        $timestamp  = (int) ($this->f3->get('GET.timestamp') ?? 0);
        $signature  = trim((string) ($this->f3->get('GET.signature')   ?? ''));

        if (!$networkCid || !$memberCid || !$timestamp || !$signature) {
            http_response_code(422); echo json_encode(['error' => 'champs_manquants']); return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) { http_response_code(403); echo json_encode(['error' => 'membre_non_enregistre']); return; }

        $canonical = "document_list|{$networkCid}|{$memberCid}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403); echo json_encode(['error' => 'signature_invalide']); return;
        }
        if (abs(time() - $timestamp) > 300) {
            http_response_code(403); echo json_encode(['error' => 'timestamp_expire']); return;
        }

        $rows = $db->exec(
            "SELECT id, author_cid, title, content, last_edited_by, created_at, updated_at
             FROM hub_documents WHERE network_cid = ? ORDER BY updated_at DESC LIMIT 200",
            [$networkCid]
        ) ?: [];

        echo json_encode(['documents' => $rows]);
    }

    // ── DELETE /hub/document ──────────────────────────────────────────────────

    public function documentDelete(): void
    {
        header('Content-Type: application/json');
        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body       = $this->jsonBody();
        $networkCid = trim((string) ($body['network_cid'] ?? ''));
        $memberCid  = trim((string) ($body['member_cid']  ?? ''));
        $docId      = trim((string) ($body['id']          ?? ''));
        $timestamp  = (int) ($body['timestamp'] ?? 0);
        $signature  = trim((string) ($body['signature']   ?? ''));

        if (!$networkCid || !$memberCid || !$docId || !$timestamp || !$signature) {
            http_response_code(422); echo json_encode(['error' => 'champs_manquants']); return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) { http_response_code(403); echo json_encode(['error' => 'membre_non_enregistre']); return; }

        $canonical = "document_delete|{$networkCid}|{$memberCid}|{$docId}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403); echo json_encode(['error' => 'signature_invalide']); return;
        }
        if (abs(time() - $timestamp) > 300) {
            http_response_code(403); echo json_encode(['error' => 'timestamp_expire']); return;
        }

        // Seul l'auteur ou un admin peut supprimer
        $rows = $db->exec(
            "SELECT author_cid FROM hub_documents WHERE id = ? AND network_cid = ?",
            [$docId, $networkCid]
        );
        if (empty($rows[0])) { http_response_code(404); echo json_encode(['error' => 'document_introuvable']); return; }

        $isAdmin = !empty($db->exec(
            "SELECT 1 FROM hub_networks WHERE network_cid = ? AND admin_cid = ?",
            [$networkCid, $memberCid]
        ));
        if ($rows[0]['author_cid'] !== $memberCid && !$isAdmin) {
            http_response_code(403); echo json_encode(['error' => 'permission_refusee']); return;
        }

        $db->exec("DELETE FROM hub_documents WHERE id = ? AND network_cid = ?", [$docId, $networkCid]);
        echo json_encode(['status' => 'deleted']);
    }

    // ── POST /hub/dm ──────────────────────────────────────────────────────────

    public function dmSend(): void
    {
        header('Content-Type: application/json');
        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body          = $this->jsonBody();
        $networkCid    = trim((string) ($body['network_cid']    ?? ''));
        $senderCid     = trim((string) ($body['sender_cid']     ?? ''));
        $recipientCid  = trim((string) ($body['recipient_cid']  ?? ''));
        $senderName    = trim((string) ($body['sender_name']    ?? ''));
        $msgBody       = trim((string) ($body['body']           ?? ''));
        $timestamp     = (int) ($body['timestamp'] ?? 0);
        $signature     = trim((string) ($body['signature']      ?? ''));

        if (!$networkCid || !$senderCid || !$recipientCid || !$msgBody || !$timestamp || !$signature) {
            http_response_code(422); echo json_encode(['error' => 'champs_manquants']); return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $senderCid);
        if (!$pubkey) { http_response_code(403); echo json_encode(['error' => 'membre_non_enregistre']); return; }

        $canonical = "dm|{$networkCid}|{$senderCid}|{$recipientCid}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403); echo json_encode(['error' => 'signature_invalide']); return;
        }
        if (abs(time() - $timestamp) > 300) {
            http_response_code(403); echo json_encode(['error' => 'timestamp_expire']); return;
        }

        $id = $this->uuidv4();
        try {
            $db->exec(
                "INSERT INTO hub_dms (id, network_cid, sender_cid, recipient_cid, sender_name, body)
                 VALUES (?, ?, ?, ?, ?, ?)",
                [$id, $networkCid, $senderCid, $recipientCid, $senderName, $msgBody]
            );
        } catch (\Exception $e) {
            error_log('[Hub dmSend] ' . $e->getMessage());
            http_response_code(500); echo json_encode(['error' => 'database_error']); return;
        }

        $this->createNotification($db, $networkCid, $recipientCid, 'dm',
            "Message de {$senderName}", $msgBody);
        http_response_code(201);
        echo json_encode(['status' => 'sent', 'id' => $id]);
    }

    // ── GET /hub/dms ──────────────────────────────────────────────────────────

    public function dmList(): void
    {
        header('Content-Type: application/json');
        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $networkCid   = trim((string) ($this->f3->get('GET.network_cid')   ?? ''));
        $memberCid    = trim((string) ($this->f3->get('GET.member_cid')    ?? ''));
        $otherCid     = trim((string) ($this->f3->get('GET.other_cid')     ?? ''));
        $timestamp    = (int) ($this->f3->get('GET.timestamp') ?? 0);
        $signature    = trim((string) ($this->f3->get('GET.signature')     ?? ''));

        if (!$networkCid || !$memberCid || !$otherCid || !$timestamp || !$signature) {
            http_response_code(422); echo json_encode(['error' => 'champs_manquants']); return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) { http_response_code(403); echo json_encode(['error' => 'membre_non_enregistre']); return; }

        $canonical = "dm_list|{$networkCid}|{$memberCid}|{$otherCid}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403); echo json_encode(['error' => 'signature_invalide']); return;
        }
        if (abs(time() - $timestamp) > 300) {
            http_response_code(403); echo json_encode(['error' => 'timestamp_expire']); return;
        }

        $rows = $db->exec(
            "SELECT id, sender_cid, recipient_cid, sender_name, body, sent_at
             FROM hub_dms
             WHERE network_cid = ?
               AND ((sender_cid = ? AND recipient_cid = ?) OR (sender_cid = ? AND recipient_cid = ?))
             ORDER BY sent_at ASC LIMIT 500",
            [$networkCid, $memberCid, $otherCid, $otherCid, $memberCid]
        ) ?: [];

        echo json_encode(['dms' => $rows]);
    }

    // ── GET /hub/dms/conversations ────────────────────────────────────────────
    // Retourne la liste des interlocuteurs avec le dernier message

    public function dmConversations(): void
    {
        header('Content-Type: application/json');
        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $networkCid = trim((string) ($this->f3->get('GET.network_cid') ?? ''));
        $memberCid  = trim((string) ($this->f3->get('GET.member_cid')  ?? ''));
        $timestamp  = (int) ($this->f3->get('GET.timestamp') ?? 0);
        $signature  = trim((string) ($this->f3->get('GET.signature')   ?? ''));

        if (!$networkCid || !$memberCid || !$timestamp || !$signature) {
            http_response_code(422); echo json_encode(['error' => 'champs_manquants']); return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) { http_response_code(403); echo json_encode(['error' => 'membre_non_enregistre']); return; }

        $canonical = "dm_conversations|{$networkCid}|{$memberCid}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403); echo json_encode(['error' => 'signature_invalide']); return;
        }
        if (abs(time() - $timestamp) > 300) {
            http_response_code(403); echo json_encode(['error' => 'timestamp_expire']); return;
        }

        // Récupère les derniers messages par conversation (interlocuteur unique)
        $rows = $db->exec(
            "SELECT
                IF(sender_cid = ?, recipient_cid, sender_cid) AS other_cid,
                MAX(sent_at) AS last_at,
                SUBSTRING_INDEX(GROUP_CONCAT(body ORDER BY sent_at DESC SEPARATOR '|||'), '|||', 1) AS last_body,
                SUBSTRING_INDEX(GROUP_CONCAT(sender_name ORDER BY sent_at DESC SEPARATOR '|||'), '|||', 1) AS last_sender_name
             FROM hub_dms
             WHERE network_cid = ? AND (sender_cid = ? OR recipient_cid = ?)
             GROUP BY IF(sender_cid = ?, recipient_cid, sender_cid)
             ORDER BY last_at DESC LIMIT 50",
            [$memberCid, $networkCid, $memberCid, $memberCid, $memberCid]
        ) ?: [];

        // Enrichir avec display_name de chaque interlocuteur
        $result = [];
        foreach ($rows as $row) {
            $otherName = $this->getMemberDisplayName($db, $networkCid, $row['other_cid']);
            $result[] = [
                'other_cid'  => $row['other_cid'],
                'other_name' => $otherName ?: $row['other_cid'],
                'last_at'    => $row['last_at'],
                'last_body'  => mb_substr($row['last_body'] ?? '', 0, 80),
            ];
        }

        echo json_encode(['conversations' => $result]);
    }

    // ── GET /hub/notifications ────────────────────────────────────────────────

    public function notificationList(): void
    {
        header('Content-Type: application/json');
        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $networkCid = trim((string) ($this->f3->get('GET.network_cid') ?? ''));
        $memberCid  = trim((string) ($this->f3->get('GET.member_cid')  ?? ''));
        $timestamp  = (int) ($this->f3->get('GET.timestamp') ?? 0);
        $signature  = trim((string) ($this->f3->get('GET.signature')   ?? ''));

        if (!$networkCid || !$memberCid || !$timestamp || !$signature) {
            http_response_code(422); echo json_encode(['error' => 'champs_manquants']); return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) { http_response_code(403); echo json_encode(['error' => 'membre_non_enregistre']); return; }

        $canonical = "notifications|{$networkCid}|{$memberCid}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403); echo json_encode(['error' => 'signature_invalide']); return;
        }
        if (abs(time() - $timestamp) > 300) {
            http_response_code(403); echo json_encode(['error' => 'timestamp_expire']); return;
        }

        $rows = $db->exec(
            "SELECT id, type, title, body, is_read, created_at
             FROM hub_notifications
             WHERE network_cid = ? AND member_cid = ?
             ORDER BY created_at DESC LIMIT 50",
            [$networkCid, $memberCid]
        ) ?: [];

        $unread = count(array_filter($rows, fn($r) => !(int)$r['is_read']));
        echo json_encode(['notifications' => $rows, 'unread' => $unread]);
    }

    // ── POST /hub/notifications/read ──────────────────────────────────────────

    public function notificationMarkRead(): void
    {
        header('Content-Type: application/json');
        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body       = $this->jsonBody();
        $networkCid = trim((string) ($body['network_cid'] ?? ''));
        $memberCid  = trim((string) ($body['member_cid']  ?? ''));
        $timestamp  = (int) ($body['timestamp'] ?? 0);
        $signature  = trim((string) ($body['signature']   ?? ''));
        $ids        = (array) ($body['ids'] ?? []);

        if (!$networkCid || !$memberCid || !$timestamp || !$signature) {
            http_response_code(422); echo json_encode(['error' => 'champs_manquants']); return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) { http_response_code(403); echo json_encode(['error' => 'membre_non_enregistre']); return; }

        $canonical = "notifications_read|{$networkCid}|{$memberCid}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403); echo json_encode(['error' => 'signature_invalide']); return;
        }
        if (abs(time() - $timestamp) > 300) {
            http_response_code(403); echo json_encode(['error' => 'timestamp_expire']); return;
        }

        if (empty($ids)) {
            // Marquer tout comme lu
            $db->exec(
                "UPDATE hub_notifications SET is_read = 1
                 WHERE network_cid = ? AND member_cid = ? AND is_read = 0",
                [$networkCid, $memberCid]
            );
        } else {
            foreach ($ids as $notifId) {
                $db->exec(
                    "UPDATE hub_notifications SET is_read = 1
                     WHERE id = ? AND network_cid = ? AND member_cid = ?",
                    [(string)$notifId, $networkCid, $memberCid]
                );
            }
        }

        echo json_encode(['status' => 'read']);
    }

    // ── GET /hub/directory ────────────────────────────────────────────────────
    // Annuaire des membres d'un réseau avec recherche optionnelle

    public function directorySearch(): void
    {
        header('Content-Type: application/json');
        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $networkCid = trim((string) ($this->f3->get('GET.network_cid') ?? ''));
        $memberCid  = trim((string) ($this->f3->get('GET.member_cid')  ?? ''));
        $query      = trim((string) ($this->f3->get('GET.q')           ?? ''));
        $timestamp  = (int) ($this->f3->get('GET.timestamp') ?? 0);
        $signature  = trim((string) ($this->f3->get('GET.signature')   ?? ''));

        if (!$networkCid || !$memberCid || !$timestamp || !$signature) {
            http_response_code(422); echo json_encode(['error' => 'champs_manquants']); return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) { http_response_code(403); echo json_encode(['error' => 'membre_non_enregistre']); return; }

        $canonical = "directory|{$networkCid}|{$memberCid}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403); echo json_encode(['error' => 'signature_invalide']); return;
        }
        if (abs(time() - $timestamp) > 300) {
            http_response_code(403); echo json_encode(['error' => 'timestamp_expire']); return;
        }

        if ($query) {
            $like = '%' . $query . '%';
            $rows = $db->exec(
                "SELECT member_cid, display_name, joined_at FROM hub_members
                 WHERE network_cid = ? AND (display_name LIKE ? OR member_cid LIKE ?)
                 ORDER BY display_name ASC LIMIT 100",
                [$networkCid, $like, $like]
            ) ?: [];
        } else {
            $rows = $db->exec(
                "SELECT member_cid, display_name, joined_at FROM hub_members
                 WHERE network_cid = ? ORDER BY display_name ASC LIMIT 200",
                [$networkCid]
            ) ?: [];
        }

        echo json_encode(['members' => $rows, 'query' => $query]);
    }

    // ── DELETE /hub/member/leave ──────────────────────────────────────────────

    public function memberLeave(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body       = $this->jsonBody();
        $networkCid = trim((string) ($body['network_cid'] ?? ''));
        $memberCid  = trim((string) ($body['member_cid']  ?? ''));
        $timestamp  = (int) ($body['timestamp'] ?? 0);
        $signature  = trim((string) ($body['signature']   ?? ''));

        if (!$networkCid || !$memberCid || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) {
            http_response_code(404);
            echo json_encode(['error' => 'membre_non_enregistre']);
            return;
        }

        $canonical = "leave|{$networkCid}|{$memberCid}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        if (abs(time() - $timestamp) > 300) {
            http_response_code(403);
            echo json_encode(['error' => 'timestamp_expire']);
            return;
        }

        $db->exec(
            "DELETE FROM hub_members WHERE network_cid = ? AND member_cid = ?",
            [$networkCid, $memberCid]
        );

        echo json_encode(['status' => 'left']);
    }

    // ── PUT /hub/agenda/event ─────────────────────────────────────────────────

    public function agendaUpdate(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body       = $this->jsonBody();
        $networkCid = trim((string) ($body['network_cid'] ?? ''));
        $memberCid  = trim((string) ($body['member_cid']  ?? ''));
        $eventId    = trim((string) ($body['event_id']    ?? ''));
        $title      = trim((string) ($body['title']       ?? ''));
        $description= trim((string) ($body['description'] ?? ''));
        $startAt    = trim((string) ($body['start_at']    ?? ''));
        $endAt      = trim((string) ($body['end_at']      ?? ''));
        $timestamp  = (int) ($body['timestamp'] ?? 0);
        $signature  = trim((string) ($body['signature']   ?? ''));

        if (!$networkCid || !$memberCid || !$eventId || !$title || !$startAt || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) {
            http_response_code(403);
            echo json_encode(['error' => 'membre_non_enregistre']);
            return;
        }

        $canonical = "agenda_update|{$networkCid}|{$memberCid}|{$eventId}|{$title}|{$startAt}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        if (abs(time() - $timestamp) > 300) {
            http_response_code(403);
            echo json_encode(['error' => 'timestamp_expire']);
            return;
        }

        // Seul l'auteur peut modifier
        $row = $db->exec(
            "SELECT author_cid FROM hub_agenda_events WHERE id = ? AND network_cid = ?",
            [$eventId, $networkCid]
        );
        if (!$row || empty($row[0])) {
            http_response_code(404);
            echo json_encode(['error' => 'evenement_inconnu']);
            return;
        }
        if ($row[0]['author_cid'] !== $memberCid) {
            http_response_code(403);
            echo json_encode(['error' => 'non_autorise']);
            return;
        }

        $startDt = date('Y-m-d H:i:s', strtotime($startAt));
        $endDt   = $endAt ? date('Y-m-d H:i:s', strtotime($endAt)) : null;

        $db->exec(
            "UPDATE hub_agenda_events SET title=?, description=?, start_at=?, end_at=? WHERE id=? AND network_cid=?",
            [$title, $description ?: null, $startDt, $endDt, $eventId, $networkCid]
        );

        echo json_encode(['status' => 'updated']);
    }

    // ── DELETE /hub/agenda/event ──────────────────────────────────────────────

    public function agendaDelete(): void
    {
        header('Content-Type: application/json');

        $db = $this->db();
        if (!$db) { $this->serviceUnavailable(); return; }

        $body       = $this->jsonBody();
        $networkCid = trim((string) ($body['network_cid'] ?? ''));
        $memberCid  = trim((string) ($body['member_cid']  ?? ''));
        $eventId    = trim((string) ($body['event_id']    ?? ''));
        $timestamp  = (int) ($body['timestamp'] ?? 0);
        $signature  = trim((string) ($body['signature']   ?? ''));

        if (!$networkCid || !$memberCid || !$eventId || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        $pubkey = $this->getMemberPubkey($db, $networkCid, $memberCid);
        if (!$pubkey) {
            http_response_code(403);
            echo json_encode(['error' => 'membre_non_enregistre']);
            return;
        }

        $canonical = "agenda_delete|{$networkCid}|{$memberCid}|{$eventId}|{$timestamp}";
        if (!$this->verifySignature($canonical, $signature, $pubkey)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        if (abs(time() - $timestamp) > 300) {
            http_response_code(403);
            echo json_encode(['error' => 'timestamp_expire']);
            return;
        }

        // Seul l'auteur peut supprimer
        $row = $db->exec(
            "SELECT author_cid FROM hub_agenda_events WHERE id = ? AND network_cid = ?",
            [$eventId, $networkCid]
        );
        if (!$row || empty($row[0])) {
            http_response_code(404);
            echo json_encode(['error' => 'evenement_inconnu']);
            return;
        }
        if ($row[0]['author_cid'] !== $memberCid) {
            http_response_code(403);
            echo json_encode(['error' => 'non_autorise']);
            return;
        }

        $db->exec(
            "DELETE FROM hub_agenda_events WHERE id = ? AND network_cid = ?",
            [$eventId, $networkCid]
        );

        echo json_encode(['status' => 'deleted']);
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    private function jsonBody(): array
    {
        $raw = file_get_contents('php://input');
        $data = json_decode($raw, true);
        return is_array($data) ? $data : [];
    }

    private function serviceUnavailable(): void
    {
        http_response_code(503);
        echo json_encode(['error' => 'service_indisponible']);
    }

    private function getMemberPubkey(\DB\SQL $db, string $networkCid, string $memberCid): ?string
    {
        $rows = $db->exec(
            "SELECT pubkey_b58 FROM hub_members WHERE network_cid = ? AND member_cid = ?",
            [$networkCid, $memberCid]
        );
        return (!empty($rows[0])) ? (string) $rows[0]['pubkey_b58'] : null;
    }

    private function getMemberDisplayName(\DB\SQL $db, string $networkCid, string $memberCid): ?string
    {
        $rows = $db->exec(
            "SELECT display_name FROM hub_members WHERE network_cid = ? AND member_cid = ?",
            [$networkCid, $memberCid]
        );
        return (!empty($rows[0])) ? (string) $rows[0]['display_name'] : null;
    }

    /**
     * Crée une notification pour un membre donné.
     */
    private function createNotification(\DB\SQL $db, string $networkCid, string $memberCid,
                                        string $type, string $title, ?string $body = null): void
    {
        try {
            $db->exec(
                "INSERT INTO hub_notifications (id, network_cid, member_cid, type, title, body)
                 VALUES (?, ?, ?, ?, ?, ?)",
                [$this->uuidv4(), $networkCid, $memberCid, $type,
                 mb_substr($title, 0, 255), $body ? mb_substr($body, 0, 1000) : null]
            );
        } catch (\Exception $e) {
            error_log('[Hub notify] ' . $e->getMessage());
        }
    }

    /**
     * Crée une notification pour tous les membres d'un réseau, sauf l'expéditeur.
     */
    private function notify(\DB\SQL $db, string $networkCid, string $type,
                            string $title, string $excludeCid = ''): void
    {
        $rows = $db->exec(
            "SELECT member_cid FROM hub_members WHERE network_cid = ?",
            [$networkCid]
        ) ?: [];
        foreach ($rows as $row) {
            if ($row['member_cid'] === $excludeCid) continue;
            $this->createNotification($db, $networkCid, $row['member_cid'], $type, $title);
        }
    }

    private function uuidv4(): string
    {
        $data = random_bytes(16);
        $data[6] = chr(ord($data[6]) & 0x0f | 0x40);
        $data[8] = chr(ord($data[8]) & 0x3f | 0x80);
        return vsprintf('%s%s-%s-%s-%s-%s%s%s', str_split(bin2hex($data), 4));
    }

    private function verifySignature(string $canonical, string $signatureB58, string $pubkeyB58): bool
    {
        if (!function_exists('sodium_crypto_sign_verify_detached')) return false;
        try {
            $pubBytes = self::base58Decode($pubkeyB58);
            $sigBytes = self::base58Decode($signatureB58);
            if (strlen($pubBytes) !== 32 || strlen($sigBytes) !== 64) return false;
            return sodium_crypto_sign_verify_detached($sigBytes, $canonical, $pubBytes);
        } catch (\Exception $e) {
            return false;
        }
    }

    // ── Base58 (alphabet Bitcoin) ─────────────────────────────────────────────

    private static function base58Encode(string $input): string
    {
        $alphabet = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';

        $num = '0';
        for ($i = 0; $i < strlen($input); $i++) {
            $num = bcadd(bcmul($num, '256'), (string) ord($input[$i]));
        }

        $result = '';
        while (bccomp($num, '0') > 0) {
            $rem    = (int) bcmod($num, '58');
            $result = $alphabet[$rem] . $result;
            $num    = bcdiv($num, '58', 0);
        }

        for ($i = 0; $i < strlen($input) && ord($input[$i]) === 0; $i++) {
            $result = '1' . $result;
        }

        return $result;
    }

    private static function base58Decode(string $input): string
    {
        $alphabet = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
        $base     = strlen($alphabet);

        $num = '0';
        for ($i = 0; $i < strlen($input); $i++) {
            $pos = strpos($alphabet, $input[$i]);
            if ($pos === false) throw new \Exception("Caractère Base58 invalide : '{$input[$i]}'");
            $num = bcadd(bcmul($num, (string) $base), (string) $pos);
        }

        $bytes = '';
        while (bccomp($num, '0') > 0) {
            $bytes = chr((int) bcmod($num, '256')) . $bytes;
            $num   = bcdiv($num, '256', 0);
        }

        for ($i = 0; $i < strlen($input) && $input[$i] === '1'; $i++) {
            $bytes = "\x00" . $bytes;
        }

        return $bytes;
    }
}
