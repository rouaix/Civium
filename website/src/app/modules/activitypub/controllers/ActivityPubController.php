<?php

/**
 * Fédération ActivityPub — Civium RCC
 *
 * Chaque réseau Civium enregistré au RCC et ayant activé la fédération
 * dispose d'un acteur ActivityPub de type "Group" à l'URL :
 *   https://www.rouaix.com/civium/users/<cid_short>
 *
 * Endpoints :
 *   GET  /.well-known/webfinger          → webfinger()
 *   GET  /users/@cid                     → actor()
 *   GET  /users/@cid/followers           → followers()
 *   GET  /users/@cid/following           → following()
 *   GET  /users/@cid/outbox              → outbox()
 *   POST /users/@cid/inbox               → inbox()
 *   POST /api/ap/enable                  → enable()
 *   POST /api/ap/post                    → post()
 *
 * Note serveur : pour la compatibilité Mastodon complète, configurer une
 * redirection au niveau du serveur web :
 *   /.well-known/webfinger → /civium/.well-known/webfinger
 */
class ActivityPubController
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

    private function baseUrl(): string
    {
        $scheme = $this->f3->get('SCHEME') ?: 'https';
        $host   = $this->f3->get('HOST')   ?: parse_url((string) $this->f3->get('APP_URL'), PHP_URL_HOST) ?: 'localhost';
        $base   = rtrim((string) $this->f3->get('BASE'), '/');
        return $scheme . '://' . $host . $base;
    }

    private function actorUrl(string $cidShort): string
    {
        return $this->baseUrl() . '/users/' . rawurlencode($cidShort);
    }

    private function cidShort(): string
    {
        return (string) ($this->f3->get('PARAMS.cid') ?? '');
    }

    // ── GET /.well-known/webfinger ─────────────────────────────────────────────

    public function webfinger(): void
    {
        header('Content-Type: application/jrd+json');
        $resource = (string) ($this->f3->get('GET.resource') ?? '');

        // resource = acct:cid_short@domain  ou  https://domain/civium/users/cid_short
        $cidShort = '';
        if (str_starts_with($resource, 'acct:')) {
            $cidShort = explode('@', substr($resource, 5))[0];
        } elseif (str_contains($resource, '/users/')) {
            $parts    = explode('/users/', $resource, 2);
            $cidShort = rawurldecode($parts[1] ?? '');
        }

        if (!$cidShort || !$this->actorExists($cidShort)) {
            http_response_code(404);
            echo json_encode(['error' => 'not_found']);
            return;
        }

        $actorUrl = $this->actorUrl($cidShort);
        echo json_encode([
            'subject' => $resource ?: 'acct:' . $cidShort . '@' . (parse_url($this->baseUrl(), PHP_URL_HOST) ?? 'civium'),
            'links'   => [
                [
                    'rel'  => 'self',
                    'type' => 'application/activity+json',
                    'href' => $actorUrl,
                ],
            ],
        ], JSON_UNESCAPED_SLASHES);
    }

    // ── GET /users/@cid ────────────────────────────────────────────────────────

    public function actor(): void
    {
        header('Content-Type: application/activity+json');
        $cidShort = $this->cidShort();
        $actor    = $this->buildActor($cidShort);

        if (!$actor) {
            http_response_code(404);
            echo json_encode(['error' => 'not_found']);
            return;
        }

        echo json_encode($actor, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);
    }

    // ── GET /users/@cid/followers ──────────────────────────────────────────────

    public function followers(): void
    {
        header('Content-Type: application/activity+json');
        $cidShort = $this->cidShort();

        if (!$this->actorExists($cidShort)) {
            http_response_code(404);
            echo json_encode(['error' => 'not_found']);
            return;
        }

        $db        = $this->db();
        $actorUrl  = $this->actorUrl($cidShort);
        $networkCid = $this->networkCidFull($cidShort);
        $followers  = [];

        if ($db && $networkCid) {
            $rows = $db->exec(
                "SELECT actor_url FROM ap_followers WHERE network_cid = ?",
                [$networkCid]
            );
            $followers = array_column($rows ?: [], 'actor_url');
        }

        echo json_encode([
            '@context'   => 'https://www.w3.org/ns/activitystreams',
            'id'         => $actorUrl . '/followers',
            'type'       => 'OrderedCollection',
            'totalItems' => count($followers),
            'orderedItems' => $followers,
        ], JSON_UNESCAPED_SLASHES);
    }

    // ── GET /users/@cid/following ──────────────────────────────────────────────

    public function following(): void
    {
        header('Content-Type: application/activity+json');
        $cidShort = $this->cidShort();
        $actorUrl = $this->actorUrl($cidShort);

        echo json_encode([
            '@context'   => 'https://www.w3.org/ns/activitystreams',
            'id'         => $actorUrl . '/following',
            'type'       => 'OrderedCollection',
            'totalItems' => 0,
            'orderedItems' => [],
        ], JSON_UNESCAPED_SLASHES);
    }

    // ── GET /users/@cid/outbox ─────────────────────────────────────────────────

    public function outbox(): void
    {
        header('Content-Type: application/activity+json');
        $cidShort   = $this->cidShort();
        $actorUrl   = $this->actorUrl($cidShort);
        $networkCid = $this->networkCidFull($cidShort);
        $items      = [];

        $db = $this->db();
        if ($db && $networkCid) {
            $rows = $db->exec(
                "SELECT activity_json FROM ap_outbox
                 WHERE network_cid = ?
                 ORDER BY created_at DESC LIMIT 20",
                [$networkCid]
            );
            foreach ($rows ?: [] as $row) {
                $act = json_decode($row['activity_json'], true);
                if ($act) $items[] = $act;
            }
        }

        echo json_encode([
            '@context'     => 'https://www.w3.org/ns/activitystreams',
            'id'           => $actorUrl . '/outbox',
            'type'         => 'OrderedCollection',
            'totalItems'   => count($items),
            'orderedItems' => $items,
        ], JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);
    }

    // ── POST /users/@cid/inbox ─────────────────────────────────────────────────

    public function inbox(): void
    {
        header('Content-Type: application/activity+json');
        $cidShort = $this->cidShort();

        if (!$this->actorExists($cidShort)) {
            http_response_code(404);
            echo json_encode(['error' => 'not_found']);
            return;
        }

        $raw      = file_get_contents('php://input');
        $activity = json_decode($raw, true);

        if (!is_array($activity)) {
            http_response_code(400);
            echo json_encode(['error' => 'invalid_json']);
            return;
        }

        // Vérifier la signature HTTP
        $headers   = HttpSignature::requestHeaders();
        $path      = '/' . ltrim(parse_url($_SERVER['REQUEST_URI'] ?? '', PHP_URL_PATH), '/');
        $base      = rtrim((string) $this->f3->get('BASE'), '/');
        $localPath = $base ? substr($path, strlen($base)) : $path;

        $actorUrl = HttpSignature::verify($headers, 'post', $localPath);

        // En mode DEBUG, on accepte sans vérification de signature
        $debug = (int) $this->f3->get('DEBUG') > 0;
        if (!$actorUrl && !$debug) {
            error_log("[AP inbox] Signature invalide pour $cidShort — path=$localPath");
            http_response_code(401);
            echo json_encode(['error' => 'invalid_signature']);
            return;
        }
        if (!$actorUrl) {
            $actorUrl = $activity['actor'] ?? 'unknown';
        }

        $type = $activity['type'] ?? '';

        switch ($type) {
            case 'Follow':
                $this->handleFollow($cidShort, $activity, $actorUrl);
                break;

            case 'Undo':
                $object = $activity['object'] ?? [];
                if (is_array($object) && ($object['type'] ?? '') === 'Follow') {
                    $this->handleUnfollow($cidShort, $actorUrl);
                }
                break;

            case 'Create':
                // Réception d'une note — loggée pour l'instant
                error_log("[AP inbox] Create reçu de $actorUrl pour $cidShort");
                break;

            default:
                error_log("[AP inbox] Activité inconnue '$type' de $actorUrl");
        }

        http_response_code(202);
        echo json_encode(['status' => 'accepted']);
    }

    // ── POST /api/ap/enable ────────────────────────────────────────────────────

    public function enable(): void
    {
        header('Content-Type: application/json');

        $raw  = file_get_contents('php://input');
        $body = json_decode($raw, true);

        $networkCid      = (string) ($body['network_cid'] ?? '');
        $networkCidShort = (string) ($body['network_cid_short'] ?? '');
        $timestamp       = (int)    ($body['timestamp'] ?? 0);
        $signature       = (string) ($body['signature'] ?? '');

        if (!$networkCid || !$networkCidShort || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        $db = $this->db();
        if (!$db) {
            http_response_code(503);
            echo json_encode(['error' => 'service_indisponible']);
            return;
        }

        // Récupérer la clé publique Ed25519 du réseau depuis le registre
        $rows = $db->exec(
            "SELECT admin_pubkey, network_name FROM networks WHERE network_cid = ?",
            [$networkCid]
        );
        if (!$rows || empty($rows[0])) {
            http_response_code(404);
            echo json_encode(['error' => 'reseau_inconnu']);
            return;
        }
        $adminPubkey = (string) $rows[0]['admin_pubkey'];
        $networkName = (string) $rows[0]['network_name'];

        // Vérifier la signature Ed25519
        $canonical = "{$networkCid}|ap_enable|{$timestamp}";
        try {
            $pubkeyBytes = ApiController::base58_decode($adminPubkey);
            $sigBytes    = ApiController::base58_decode($signature);
        } catch (\Exception $e) {
            http_response_code(422);
            echo json_encode(['error' => 'encodage_invalide', 'detail' => $e->getMessage()]);
            return;
        }

        if (!function_exists('sodium_crypto_sign_verify_detached')) {
            http_response_code(500);
            echo json_encode(['error' => 'sodium_non_disponible']);
            return;
        }

        if (!sodium_crypto_sign_verify_detached($sigBytes, $canonical, $pubkeyBytes)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        // Vérifier que l'AP n'est pas déjà activé
        $existing = $db->exec(
            "SELECT network_cid FROM ap_actors WHERE network_cid = ?",
            [$networkCid]
        );
        if ($existing && !empty($existing[0])) {
            $actorUrl = $this->actorUrl($networkCidShort);
            echo json_encode(['status' => 'deja_active', 'actor_url' => $actorUrl]);
            return;
        }

        // Générer une paire de clés RSA-2048
        $rsaKey = openssl_pkey_new([
            'private_key_bits' => 2048,
            'private_key_type' => OPENSSL_KEYTYPE_RSA,
        ]);
        if (!$rsaKey) {
            http_response_code(500);
            echo json_encode(['error' => 'erreur_generation_cle']);
            return;
        }

        openssl_pkey_export($rsaKey, $privkeyPem);
        $details   = openssl_pkey_get_details($rsaKey);
        $pubkeyPem = $details['key'];

        // Stocker dans ap_actors
        $db->exec(
            "INSERT INTO ap_actors (network_cid, network_cid_short, rsa_privkey, rsa_pubkey)
             VALUES (?, ?, ?, ?)",
            [$networkCid, $networkCidShort, $privkeyPem, $pubkeyPem]
        );

        $actorUrl = $this->actorUrl($networkCidShort);
        echo json_encode([
            'status'    => 'active',
            'actor_url' => $actorUrl,
        ]);
    }

    // ── POST /api/ap/post ──────────────────────────────────────────────────────

    public function post(): void
    {
        header('Content-Type: application/json');

        $raw  = file_get_contents('php://input');
        $body = json_decode($raw, true);

        $networkCid = (string) ($body['network_cid'] ?? '');
        $noteId     = (string) ($body['note_id'] ?? '');
        $content    = (string) ($body['content'] ?? '');
        $timestamp  = (int)    ($body['timestamp'] ?? 0);
        $signature  = (string) ($body['signature'] ?? '');

        if (!$networkCid || !$noteId || !$content || !$timestamp || !$signature) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        $db = $this->db();
        if (!$db) {
            http_response_code(503);
            echo json_encode(['error' => 'service_indisponible']);
            return;
        }

        // Vérifier signature Ed25519
        $rows = $db->exec(
            "SELECT admin_pubkey FROM networks WHERE network_cid = ?",
            [$networkCid]
        );
        if (!$rows || empty($rows[0])) {
            http_response_code(404);
            echo json_encode(['error' => 'reseau_inconnu']);
            return;
        }

        $canonical = "{$networkCid}|{$noteId}|{$content}|{$timestamp}";
        try {
            $pubkeyBytes = ApiController::base58_decode((string) $rows[0]['admin_pubkey']);
            $sigBytes    = ApiController::base58_decode($signature);
        } catch (\Exception $e) {
            http_response_code(422);
            echo json_encode(['error' => 'encodage_invalide']);
            return;
        }

        if (!sodium_crypto_sign_verify_detached($sigBytes, $canonical, $pubkeyBytes)) {
            http_response_code(403);
            echo json_encode(['error' => 'signature_invalide']);
            return;
        }

        // Récupérer l'acteur AP
        $actorRows = $db->exec(
            "SELECT network_cid_short, rsa_privkey FROM ap_actors WHERE network_cid = ?",
            [$networkCid]
        );
        if (!$actorRows || empty($actorRows[0])) {
            http_response_code(403);
            echo json_encode(['error' => 'ap_non_active']);
            return;
        }

        $cidShort    = (string) $actorRows[0]['network_cid_short'];
        $privkeyPem  = (string) $actorRows[0]['rsa_privkey'];
        $actorUrl    = $this->actorUrl($cidShort);
        $keyId       = $actorUrl . '#main-key';
        $noteUrl     = $actorUrl . '/posts/' . rawurlencode($noteId);
        $published   = gmdate('Y-m-d\TH:i:s\Z', $timestamp);

        // Construire l'activité Create + Note
        $note = [
            'id'           => $noteUrl,
            'type'         => 'Note',
            'attributedTo' => $actorUrl,
            'content'      => htmlspecialchars($content, ENT_QUOTES | ENT_SUBSTITUTE, 'UTF-8'),
            'published'    => $published,
            'to'           => ['https://www.w3.org/ns/activitystreams#Public'],
            'cc'           => [$actorUrl . '/followers'],
        ];

        $activityId = $actorUrl . '/activities/' . rawurlencode($noteId);
        $activity   = [
            '@context'  => 'https://www.w3.org/ns/activitystreams',
            'id'        => $activityId,
            'type'      => 'Create',
            'actor'     => $actorUrl,
            'published' => $published,
            'to'        => ['https://www.w3.org/ns/activitystreams#Public'],
            'cc'        => [$actorUrl . '/followers'],
            'object'    => $note,
        ];

        $activityJson = json_encode($activity, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);

        // Stocker dans ap_outbox
        try {
            $db->exec(
                "INSERT IGNORE INTO ap_outbox (network_cid, activity_id, activity_json)
                 VALUES (?, ?, ?)",
                [$networkCid, $activityId, $activityJson]
            );
        } catch (\Exception $e) {
            // Doublon éventuel — on continue la livraison
        }

        // Récupérer les abonnés et livrer
        $followersRows = $db->exec(
            "SELECT inbox_url, shared_inbox FROM ap_followers WHERE network_cid = ?",
            [$networkCid]
        );

        $inboxes   = [];
        foreach ($followersRows ?: [] as $f) {
            $inbox = !empty($f['shared_inbox']) ? $f['shared_inbox'] : $f['inbox_url'];
            $inboxes[$inbox] = true;
        }

        $delivered = 0;
        foreach (array_keys($inboxes) as $inboxUrl) {
            if (HttpSignature::deliver($inboxUrl, $activityJson, $privkeyPem, $keyId)) {
                $delivered++;
            }
        }

        // Marquer comme livré si au moins un succès (ou pas d'abonnés)
        if ($delivered > 0 || empty($inboxes)) {
            $db->exec(
                "UPDATE ap_outbox SET delivered = 1 WHERE activity_id = ?",
                [$activityId]
            );
        }

        echo json_encode([
            'status'       => 'publié',
            'activity_id'  => $activityId,
            'delivered_to' => $delivered,
        ]);
    }

    // ── Helpers internes ───────────────────────────────────────────────────────

    private function actorExists(string $cidShort): bool
    {
        $db = $this->db();
        if (!$db || !$cidShort) return false;
        $rows = $db->exec(
            "SELECT 1 FROM ap_actors WHERE network_cid_short = ? LIMIT 1",
            [$cidShort]
        );
        return !empty($rows);
    }

    private function networkCidFull(string $cidShort): ?string
    {
        $db = $this->db();
        if (!$db || !$cidShort) return null;
        $rows = $db->exec(
            "SELECT network_cid FROM ap_actors WHERE network_cid_short = ?",
            [$cidShort]
        );
        return !empty($rows[0]) ? (string) $rows[0]['network_cid'] : null;
    }

    private function buildActor(string $cidShort): ?array
    {
        $db = $this->db();
        if (!$db || !$cidShort) return null;

        $rows = $db->exec(
            "SELECT a.network_cid, a.rsa_pubkey, n.network_name
             FROM ap_actors a
             JOIN networks n ON n.network_cid = a.network_cid
             WHERE a.network_cid_short = ?",
            [$cidShort]
        );
        if (!$rows || empty($rows[0])) return null;

        $networkName = (string) $rows[0]['network_name'];
        $pubkeyPem   = (string) $rows[0]['rsa_pubkey'];
        $actorUrl    = $this->actorUrl($cidShort);

        return [
            '@context' => [
                'https://www.w3.org/ns/activitystreams',
                'https://w3id.org/security/v1',
            ],
            'id'                 => $actorUrl,
            'type'               => 'Group',
            'preferredUsername'  => $cidShort,
            'name'               => $networkName,
            'summary'            => 'Réseau Civium public — ' . htmlspecialchars($networkName, ENT_QUOTES, 'UTF-8'),
            'inbox'              => $actorUrl . '/inbox',
            'outbox'             => $actorUrl . '/outbox',
            'followers'          => $actorUrl . '/followers',
            'following'          => $actorUrl . '/following',
            'url'                => $actorUrl,
            'manuallyApprovesFollowers' => false,
            'publicKey'          => [
                'id'           => $actorUrl . '#main-key',
                'owner'        => $actorUrl,
                'publicKeyPem' => $pubkeyPem,
            ],
        ];
    }

    private function handleFollow(string $cidShort, array $activity, string $followerUrl): void
    {
        $db         = $this->db();
        $networkCid = $this->networkCidFull($cidShort);
        if (!$db || !$networkCid) return;

        // Récupérer l'inbox du follower
        $actorData = HttpSignature::fetchJson($followerUrl);
        $inboxUrl  = $actorData['inbox'] ?? $followerUrl . '/inbox';
        $sharedInbox = $actorData['endpoints']['sharedInbox'] ?? null;

        // Enregistrer l'abonné
        $db->exec(
            "INSERT IGNORE INTO ap_followers (network_cid, actor_url, inbox_url, shared_inbox)
             VALUES (?, ?, ?, ?)",
            [$networkCid, $followerUrl, $inboxUrl, $sharedInbox]
        );

        // Envoyer Accept
        $actorRow = $db->exec(
            "SELECT rsa_privkey, network_cid_short FROM ap_actors WHERE network_cid = ?",
            [$networkCid]
        );
        if (!$actorRow || empty($actorRow[0])) return;

        $actorUrl   = $this->actorUrl((string) $actorRow[0]['network_cid_short']);
        $privkeyPem = (string) $actorRow[0]['rsa_privkey'];
        $keyId      = $actorUrl . '#main-key';

        $accept = json_encode([
            '@context' => 'https://www.w3.org/ns/activitystreams',
            'id'       => $actorUrl . '#accepts/follows/' . uniqid(),
            'type'     => 'Accept',
            'actor'    => $actorUrl,
            'object'   => $activity,
        ], JSON_UNESCAPED_SLASHES);

        HttpSignature::deliver($inboxUrl, $accept, $privkeyPem, $keyId);
    }

    private function handleUnfollow(string $cidShort, string $followerUrl): void
    {
        $db         = $this->db();
        $networkCid = $this->networkCidFull($cidShort);
        if (!$db || !$networkCid) return;

        $db->exec(
            "DELETE FROM ap_followers WHERE network_cid = ? AND actor_url = ?",
            [$networkCid, $followerUrl]
        );
    }
}
