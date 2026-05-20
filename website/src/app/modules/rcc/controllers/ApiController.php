<?php

/**
 * Registre Central Civium — endpoints API publics
 *
 * POST /api/register  — enregistre un réseau Civium (signature Ed25519 requise)
 * GET  /api/networks  — liste publique paginée des réseaux enregistrés
 * GET  /api/status    — healthcheck
 */
class ApiController
{
    protected Base $f3;

    public function __construct()
    {
        $this->f3 = Base::instance();
    }

    // ── GET /api/status ────────────────────────────────────────────────────────

    public function status(): void
    {
        header('Content-Type: application/json');
        echo json_encode(['status' => 'ok', 'version' => '1']);
    }

    // ── POST /api/register ─────────────────────────────────────────────────────

    public function register(): void
    {
        header('Content-Type: application/json');

        // Limite de taille : 64 Ko max pour éviter les attaques DoS par payload massif
        $raw = file_get_contents('php://input', false, null, 0, 65537);
        if (strlen($raw) > 65536) {
            http_response_code(413);
            echo json_encode(['error' => 'payload_too_large']);
            return;
        }

        $data = json_decode($raw, true);
        if (!is_array($data)) {
            http_response_code(400);
            echo json_encode(['error' => 'invalid_json']);
            return;
        }

        $required = ['network_cid', 'network_name', 'admin_cid', 'admin_pubkey',
                     'admin_email', 'registered_at', 'signature'];
        foreach ($required as $field) {
            if (empty($data[$field])) {
                http_response_code(422);
                echo json_encode(['error' => 'missing_field', 'field' => $field]);
                return;
            }
        }

        $network_cid   = (string) $data['network_cid'];
        $network_name  = (string) $data['network_name'];
        $admin_cid     = (string) $data['admin_cid'];
        $admin_pubkey  = (string) $data['admin_pubkey'];
        $admin_email   = filter_var((string) $data['admin_email'], FILTER_VALIDATE_EMAIL);
        $registered_at = (int) $data['registered_at'];
        $signature     = (string) $data['signature'];

        if (!$admin_email) {
            http_response_code(422);
            echo json_encode(['error' => 'invalid_email']);
            return;
        }

        // Vérification de la signature Ed25519
        try {
            $pubkey_bytes = self::base58_decode($admin_pubkey);
            $sig_bytes    = self::base58_decode($signature);
        } catch (\Exception $e) {
            http_response_code(422);
            echo json_encode(['error' => 'invalid_encoding', 'detail' => $e->getMessage()]);
            return;
        }

        if (strlen($pubkey_bytes) !== 32 || strlen($sig_bytes) !== 64) {
            http_response_code(422);
            echo json_encode(['error' => 'invalid_key_or_signature_length']);
            return;
        }

        $canonical = implode('|', [
            $network_cid, $network_name, $admin_cid,
            $admin_pubkey, $admin_email, $registered_at,
        ]);

        if (!function_exists('sodium_crypto_sign_verify_detached')) {
            http_response_code(500);
            echo json_encode(['error' => 'sodium_not_available']);
            return;
        }

        if (!sodium_crypto_sign_verify_detached($sig_bytes, $canonical, $pubkey_bytes)) {
            http_response_code(403);
            echo json_encode(['error' => 'invalid_signature']);
            return;
        }

        $db = $this->f3->get('DB');
        if (!$db) {
            http_response_code(503);
            echo json_encode(['error' => 'service_indisponible']);
            return;
        }

        // Vérification des limites white-label (max_networks)
        $wl = WhiteLabelController::loadSettings($db);
        $tierLimits  = WhiteLabelController::tierLimits($wl['license_tier']);
        $maxNetworks = (int) $wl['max_networks'] ?: $tierLimits['max_networks'];

        if ($maxNetworks > 0) {
            $countRow = $db->exec("SELECT COUNT(*) AS n FROM networks");
            $current  = (int) ($countRow[0]['n'] ?? 0);

            // Ne pas compter ce réseau s'il est déjà enregistré (mise à jour)
            $existsRow = $db->exec(
                "SELECT 1 FROM networks WHERE network_cid = ? LIMIT 1",
                [$network_cid]
            );
            $isUpdate = !empty($existsRow);

            if (!$isUpdate && $current >= $maxNetworks) {
                http_response_code(429);
                echo json_encode([
                    'error'        => 'limite_atteinte',
                    'max_networks' => $maxNetworks,
                    'license_tier' => $wl['license_tier'],
                ]);
                return;
            }
        }

        $ip = $_SERVER['REMOTE_ADDR'] ?? '0.0.0.0';

        // Rate limiting : max 10 enregistrements par IP par heure
        $rateRow = $db->exec(
            "SELECT COUNT(*) AS n FROM networks WHERE ip_address = ? AND server_registered_at >= DATE_SUB(NOW(), INTERVAL 1 HOUR)",
            [$ip]
        );
        if ((int) ($rateRow[0]['n'] ?? 0) >= 10) {
            http_response_code(429);
            echo json_encode(['error' => 'rate_limit_exceeded', 'retry_after' => 3600]);
            return;
        }

        try {
            $db->exec(
                "INSERT INTO networks
                 (network_cid, network_name, admin_cid, admin_pubkey, admin_email, ip_address, registered_at, signature)
                 VALUES (?, ?, ?, ?, ?, ?, FROM_UNIXTIME(?), ?)
                 ON DUPLICATE KEY UPDATE
                     network_name  = VALUES(network_name),
                     admin_email   = VALUES(admin_email),
                     ip_address    = VALUES(ip_address),
                     registered_at = VALUES(registered_at),
                     signature     = VALUES(signature)",
                [$network_cid, $network_name, $admin_cid, $admin_pubkey,
                 $admin_email, $ip, $registered_at, $signature]
            );
        } catch (\Exception $e) {
            error_log('[RCC register] DB error: ' . $e->getMessage());
            http_response_code(500);
            echo json_encode(['error' => 'database_error']);
            return;
        }

        http_response_code(201);
        echo json_encode(['status' => 'registered', 'network_cid' => $network_cid]);
    }

    // ── GET /api/networks ─────────────────────────────────────────────────────

    public function networks(): void
    {
        header('Content-Type: application/json');

        $db = $this->f3->get('DB');
        if (!$db) {
            http_response_code(503);
            echo json_encode(['error' => 'service_indisponible']);
            return;
        }

        $perPage = min(max((int) ($this->f3->get('GET.per_page') ?: 50), 1), 200);
        $page    = max((int) ($this->f3->get('GET.page') ?: 1), 1);
        $offset  = ($page - 1) * $perPage;

        try {
            $total = (int) $db->exec(
                "SELECT COUNT(*) AS n FROM networks"
            )[0]['n'];

            $rows = $db->exec(
                "SELECT network_cid, network_name, registered_at
                 FROM networks
                 ORDER BY registered_at DESC
                 LIMIT ? OFFSET ?",
                [$perPage, $offset]
            );

            echo json_encode([
                'networks'   => $rows ?: [],
                'total'      => $total,
                'page'       => $page,
                'per_page'   => $perPage,
                'last_page'  => (int) ceil($total / $perPage),
            ]);
        } catch (\Exception $e) {
            http_response_code(500);
            echo json_encode(['error' => 'database_error']);
        }
    }

    // ── DELETE /api/networks/<cid> ────────────────────────────────────────────

    /**
     * Permet à l'admin d'un réseau de le dés-enregistrer du RCC.
     * Authentification : signature Ed25519 du message "unregister|<network_cid>|<timestamp>"
     * Body JSON : { admin_pubkey, timestamp, signature }
     */
    public function deleteNetwork(): void
    {
        header('Content-Type: application/json');

        $network_cid = $this->f3->get('PARAMS.cid');
        if (!$network_cid) {
            http_response_code(400);
            echo json_encode(['error' => 'missing_cid']);
            return;
        }

        $raw = file_get_contents('php://input', false, null, 0, 4097);
        if (strlen($raw) > 4096) {
            http_response_code(413);
            echo json_encode(['error' => 'payload_too_large']);
            return;
        }
        $data = json_decode($raw, true);
        if (!is_array($data) || empty($data['admin_pubkey']) || empty($data['timestamp']) || empty($data['signature'])) {
            http_response_code(422);
            echo json_encode(['error' => 'missing_fields', 'required' => ['admin_pubkey', 'timestamp', 'signature']]);
            return;
        }

        $timestamp = (int) $data['timestamp'];
        if (abs(time() - $timestamp) > 300) {
            http_response_code(400);
            echo json_encode(['error' => 'timestamp_expired', 'detail' => 'La requête a expiré (fenêtre de 5 minutes).']);
            return;
        }

        try {
            $pubkey_bytes = self::base58_decode((string) $data['admin_pubkey']);
            $sig_bytes    = self::base58_decode((string) $data['signature']);
        } catch (\Exception $e) {
            http_response_code(422);
            echo json_encode(['error' => 'invalid_encoding', 'detail' => $e->getMessage()]);
            return;
        }

        if (!function_exists('sodium_crypto_sign_verify_detached')) {
            http_response_code(500);
            echo json_encode(['error' => 'sodium_not_available']);
            return;
        }

        $canonical = 'unregister|' . $network_cid . '|' . $timestamp;
        if (!sodium_crypto_sign_verify_detached($sig_bytes, $canonical, $pubkey_bytes)) {
            http_response_code(403);
            echo json_encode(['error' => 'invalid_signature']);
            return;
        }

        $db = $this->f3->get('DB');
        if (!$db) {
            http_response_code(503);
            echo json_encode(['error' => 'service_indisponible']);
            return;
        }

        // Vérifier que la pubkey correspond bien à l'admin enregistré pour ce réseau
        $rows = $db->exec(
            "SELECT admin_pubkey FROM networks WHERE network_cid = ? LIMIT 1",
            [$network_cid]
        );
        if (empty($rows)) {
            http_response_code(404);
            echo json_encode(['error' => 'not_found']);
            return;
        }
        if ($rows[0]['admin_pubkey'] !== (string) $data['admin_pubkey']) {
            http_response_code(403);
            echo json_encode(['error' => 'not_authorized', 'detail' => 'La clé publique ne correspond pas à l\'admin enregistré.']);
            return;
        }

        $db->exec("DELETE FROM networks WHERE network_cid = ?", [$network_cid]);
        echo json_encode(['status' => 'unregistered', 'network_cid' => $network_cid]);
    }

    // ── GET /api/alerts ──────────────────────────────────────────────────────

    public function alerts(): void
    {
        header('Content-Type: application/json');
        header('Access-Control-Allow-Origin: *');

        $db = $this->f3->get('DB');
        if (!$db) { echo json_encode(['alerts' => []]); return; }

        try {
            $rows = $db->exec(
                "SELECT type, description, network_cids,
                        UNIX_TIMESTAMP(emitted_at) AS emitted_at, emitted_by
                 FROM alerts ORDER BY emitted_at DESC LIMIT 50"
            ) ?: [];
            foreach ($rows as &$row) {
                $row['network_cids'] = json_decode($row['network_cids'] ?? '[]', true) ?: [];
                $row['emitted_at']   = (int) $row['emitted_at'];
            }
            echo json_encode(['alerts' => $rows]);
        } catch (\Exception $e) {
            echo json_encode(['alerts' => []]);
        }
    }

    // ── Helpers ────────────────────────────────────────────────────────────────

    /**
     * Décode une chaîne Base58 (alphabet Bitcoin) en octets bruts.
     * @throws \Exception si le caractère est invalide
     */
    public static function base58_decode(string $input): string
    {
        $alphabet = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
        $base     = strlen($alphabet);

        $num = '0';
        for ($i = 0; $i < strlen($input); $i++) {
            $pos = strpos($alphabet, $input[$i]);
            if ($pos === false) {
                throw new \Exception("Caractère Base58 invalide : '{$input[$i]}'");
            }
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
