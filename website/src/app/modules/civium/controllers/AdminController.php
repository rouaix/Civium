<?php

/**
 * Administration RCC — réservée à l'opérateur.
 *
 * Deux modes d'authentification :
 *   1. Session navigateur — login/mot de passe via GET|POST /admin/login
 *   2. Token API         — header X-Admin-Token ou ?token= pour les appels programmatiques
 *
 * GET  /admin/login    → formulaire de connexion
 * POST /admin/login    → traitement du formulaire
 * GET  /admin/logout   → destruction de la session
 * GET  /admin          → page HTML de gestion
 * POST /admin/alerte   → enregistre une alerte fraude + envoie emails
 */
class AdminController
{
    protected Base $f3;

    public function __construct()
    {
        $this->f3 = Base::instance();
        if (session_status() === PHP_SESSION_NONE) {
            session_start();
        }
    }

    // ── Authentification ──────────────────────────────────────────────────────

    private function isSessionAuthed(): bool
    {
        return !empty($_SESSION['civium_admin']);
    }

    private function isTokenAuthed(): bool
    {
        $configured = (string) $this->f3->get('ADMIN_TOKEN');
        if (!$configured) return false;

        $provided = $_SERVER['HTTP_X_ADMIN_TOKEN']
                 ?? $this->f3->get('GET.token')
                 ?? '';

        return hash_equals($configured, (string) $provided);
    }

    private function isAuthed(): bool
    {
        return $this->isSessionAuthed() || $this->isTokenAuthed();
    }

    private function requireAuth(): bool
    {
        if ($this->isAuthed()) return true;

        $this->f3->reroute('/admin/login');
        return false;
    }

    private function db(): ?\DB\SQL
    {
        return $this->f3->get('DB') ?: null;
    }

    // ── GET /admin/login ──────────────────────────────────────────────────────

    public function loginPage(): void
    {
        if ($this->isSessionAuthed()) {
            $this->f3->reroute('/admin');
            return;
        }

        $this->f3->set('title', 'Connexion — Administration Civium');
        $this->f3->set('login_error', '');
        echo Template::instance()->render('admin-login.html');
    }

    // ── POST /admin/login ─────────────────────────────────────────────────────

    public function loginSubmit(): void
    {
        $configuredUser = (string) $this->f3->get('ADMIN_USER');
        $configuredHash = (string) $this->f3->get('ADMIN_PASS');

        $user = trim((string) ($this->f3->get('POST.username') ?? ''));
        $pass = (string) ($this->f3->get('POST.password') ?? '');

        $ok = $configuredUser
           && $configuredHash
           && hash_equals($configuredUser, $user)
           && password_verify($pass, $configuredHash);

        if ($ok) {
            session_regenerate_id(true);
            $_SESSION['civium_admin'] = true;
            $this->f3->reroute('/admin');
            return;
        }

        // Délai anti-bruteforce
        sleep(1);

        $this->f3->set('title', 'Connexion — Administration Civium');
        $this->f3->set('login_error', 'Identifiants incorrects.');
        echo Template::instance()->render('admin-login.html');
    }

    // ── GET /admin/logout ─────────────────────────────────────────────────────

    public function logout(): void
    {
        $_SESSION = [];
        session_destroy();
        $this->f3->reroute('/admin/login');
    }

    // ── GET /admin ─────────────────────────────────────────────────────────────

    public function page(): void
    {
        if (!$this->requireAuth()) return;

        $db       = $this->db();
        $networks = $db
            ? ($db->exec("SELECT * FROM networks ORDER BY registered_at DESC LIMIT 500") ?: [])
            : [];
        $alerts   = $db
            ? ($db->exec("SELECT * FROM alerts ORDER BY emitted_at DESC LIMIT 100") ?: [])
            : [];

        $this->f3->set('title', 'Administration RCC — Civium');
        $this->f3->set('networks', $networks);
        $this->f3->set('alerts', $alerts);
        $this->f3->set('via_session', $this->isSessionAuthed());
        $this->f3->set('admin_token', (string) $this->f3->get('ADMIN_TOKEN'));

        echo Template::instance()->render('admin.html');
    }

    // ── GET /admin/hub/main ───────────────────────────────────────────────────

    public function hubMain(): void
    {
        header('Content-Type: application/json');
        if (!$this->isAuthed()) { http_response_code(401); echo json_encode(['error' => 'unauthorized']); return; }

        $db = $this->db();
        if (!$db) { http_response_code(503); echo json_encode(['error' => 'service_indisponible']); return; }

        $main = $db->exec("SELECT network_cid, network_name, created_at FROM hub_main_network WHERE id = 1");
        if (!$main || empty($main[0])) {
            echo json_encode(['error' => 'no_main_network']); return;
        }
        $cid = $main[0]['network_cid'];

        $memberCount = $db->exec("SELECT COUNT(*) AS n FROM hub_members WHERE network_cid = ?", [$cid]);
        $msgCount    = $db->exec("SELECT COUNT(*) AS n FROM hub_messages WHERE network_cid = ?", [$cid]);

        echo json_encode([
            'network_cid'   => $cid,
            'network_name'  => $main[0]['network_name'],
            'created_at'    => $main[0]['created_at'],
            'member_count'  => (int) ($memberCount[0]['n'] ?? 0),
            'message_count' => (int) ($msgCount[0]['n'] ?? 0),
        ]);
    }

    // ── GET /admin/hub/members ────────────────────────────────────────────────

    public function hubMembers(): void
    {
        header('Content-Type: application/json');
        if (!$this->isAuthed()) { http_response_code(401); echo json_encode(['error' => 'unauthorized']); return; }

        $db = $this->db();
        if (!$db) { http_response_code(503); echo json_encode(['error' => 'service_indisponible']); return; }

        $main = $db->exec("SELECT network_cid FROM hub_main_network WHERE id = 1");
        if (!$main || empty($main[0])) { echo json_encode([]); return; }
        $cid = $main[0]['network_cid'];

        $members = $db->exec(
            "SELECT member_cid, display_name, joined_at FROM hub_members
             WHERE network_cid = ? ORDER BY joined_at DESC",
            [$cid]
        ) ?: [];
        echo json_encode($members);
    }

    // ── POST /admin/hub/kick ──────────────────────────────────────────────────

    public function hubKick(): void
    {
        header('Content-Type: application/json');
        if (!$this->isAuthed()) { http_response_code(401); echo json_encode(['error' => 'unauthorized']); return; }

        $db = $this->db();
        if (!$db) { http_response_code(503); echo json_encode(['error' => 'service_indisponible']); return; }

        $body = json_decode(file_get_contents('php://input'), true);
        $memberCid = trim((string) ($body['member_cid'] ?? ''));
        if (!$memberCid) { http_response_code(422); echo json_encode(['error' => 'member_cid_requis']); return; }

        $main = $db->exec("SELECT network_cid FROM hub_main_network WHERE id = 1");
        if (!$main || empty($main[0])) { http_response_code(404); echo json_encode(['error' => 'no_main_network']); return; }
        $cid = $main[0]['network_cid'];

        $db->exec("DELETE FROM hub_members WHERE network_cid = ? AND member_cid = ?", [$cid, $memberCid]);
        echo json_encode(['status' => 'expelled']);
    }

    // ── GET /admin/hub/messages ───────────────────────────────────────────────

    public function hubMessages(): void
    {
        header('Content-Type: application/json');
        if (!$this->isAuthed()) { http_response_code(401); echo json_encode(['error' => 'unauthorized']); return; }

        $db = $this->db();
        if (!$db) { http_response_code(503); echo json_encode(['error' => 'service_indisponible']); return; }

        $main = $db->exec("SELECT network_cid FROM hub_main_network WHERE id = 1");
        if (!$main || empty($main[0])) { echo json_encode([]); return; }
        $cid = $main[0]['network_cid'];

        // Les messages sont chiffrés — on expose uniquement les métadonnées
        $messages = $db->exec(
            "SELECT m.message_id, m.sender_cid, m.received_at,
                    COALESCE(mem.display_name, m.sender_cid) AS sender_name
             FROM hub_messages m
             LEFT JOIN hub_members mem
               ON mem.network_cid = m.network_cid AND mem.member_cid = m.sender_cid
             WHERE m.network_cid = ?
             ORDER BY m.received_at DESC LIMIT 200",
            [$cid]
        ) ?: [];
        echo json_encode($messages);
    }

    // ── DELETE /admin/hub/message ─────────────────────────────────────────────

    public function hubDeleteMessage(): void
    {
        header('Content-Type: application/json');
        if (!$this->isAuthed()) { http_response_code(401); echo json_encode(['error' => 'unauthorized']); return; }

        $db = $this->db();
        if (!$db) { http_response_code(503); echo json_encode(['error' => 'service_indisponible']); return; }

        $body = json_decode(file_get_contents('php://input'), true);
        $messageId = trim((string) ($body['message_id'] ?? ''));
        if (!$messageId) { http_response_code(422); echo json_encode(['error' => 'message_id_requis']); return; }

        $main = $db->exec("SELECT network_cid FROM hub_main_network WHERE id = 1");
        if (!$main || empty($main[0])) { http_response_code(404); echo json_encode(['error' => 'no_main_network']); return; }
        $cid = $main[0]['network_cid'];

        $db->exec("DELETE FROM hub_messages WHERE network_cid = ? AND message_id = ?", [$cid, $messageId]);
        echo json_encode(['status' => 'deleted']);
    }

    // ── POST /admin/hub/rename ────────────────────────────────────────────────

    public function hubRename(): void
    {
        header('Content-Type: application/json');
        if (!$this->isAuthed()) { http_response_code(401); echo json_encode(['error' => 'unauthorized']); return; }

        $db = $this->db();
        if (!$db) { http_response_code(503); echo json_encode(['error' => 'service_indisponible']); return; }

        $body = json_decode(file_get_contents('php://input'), true);
        $name = trim((string) ($body['name'] ?? ''));
        if (!$name) { http_response_code(422); echo json_encode(['error' => 'nom_requis']); return; }
        if (strlen($name) > 128) { http_response_code(422); echo json_encode(['error' => 'nom_trop_long']); return; }

        $db->exec("UPDATE hub_main_network SET network_name = ? WHERE id = 1", [$name]);
        echo json_encode(['status' => 'renamed', 'name' => $name]);
    }

    // ── GET /admin/web-users ──────────────────────────────────────────────────

    public function webUsers(): void
    {
        header('Content-Type: application/json');
        if (!$this->isAuthed()) { http_response_code(401); echo json_encode(['error' => 'unauthorized']); return; }

        $db = $this->db();
        if (!$db) { http_response_code(503); echo json_encode(['error' => 'service_indisponible']); return; }

        $pwUsers = $db->exec(
            "SELECT email, created_at, 1 AS has_password FROM web_users ORDER BY created_at DESC LIMIT 500"
        ) ?: [];

        $magicOnly = $db->exec(
            "SELECT email, MIN(created_at) AS created_at, 0 AS has_password
             FROM web_members
             WHERE email NOT IN (SELECT email FROM web_users)
             GROUP BY email
             ORDER BY MIN(created_at) DESC LIMIT 500"
        ) ?: [];

        $counts = $db->exec("SELECT email, COUNT(*) AS n FROM web_members GROUP BY email") ?: [];
        $countMap = array_column($counts, 'n', 'email');

        $users = array_merge($pwUsers, $magicOnly);
        foreach ($users as &$u) {
            $u['has_password']   = (bool) $u['has_password'];
            $u['network_count']  = (int) ($countMap[$u['email']] ?? 0);
        }
        unset($u);

        usort($users, fn($a, $b) => strcmp((string)$b['created_at'], (string)$a['created_at']));

        echo json_encode(array_values($users));
    }

    // ── DELETE /admin/web-user ────────────────────────────────────────────────

    public function webUserDelete(): void
    {
        header('Content-Type: application/json');
        if (!$this->isAuthed()) { http_response_code(401); echo json_encode(['error' => 'unauthorized']); return; }

        $db = $this->db();
        if (!$db) { http_response_code(503); echo json_encode(['error' => 'service_indisponible']); return; }

        $body  = json_decode(file_get_contents('php://input'), true);
        $email = trim((string) ($body['email'] ?? ''));
        if (!$email || !filter_var($email, FILTER_VALIDATE_EMAIL)) {
            http_response_code(422); echo json_encode(['error' => 'email_invalide']); return;
        }

        $db->exec("DELETE FROM web_users   WHERE email = ?", [$email]);
        $db->exec("DELETE FROM web_members WHERE email = ?", [$email]);

        echo json_encode(['status' => 'deleted']);
    }

    // ── DELETE /admin/network ─────────────────────────────────────────────────

    public function networkDelete(): void
    {
        header('Content-Type: application/json');
        if (!$this->isAuthed()) { http_response_code(401); echo json_encode(['error' => 'unauthorized']); return; }

        $db = $this->db();
        if (!$db) { http_response_code(503); echo json_encode(['error' => 'service_indisponible']); return; }

        $body       = json_decode(file_get_contents('php://input'), true);
        $networkCid = trim((string) ($body['network_cid'] ?? ''));
        if (!$networkCid) { http_response_code(422); echo json_encode(['error' => 'network_cid_requis']); return; }

        $db->exec("DELETE FROM networks WHERE network_cid = ?", [$networkCid]);
        echo json_encode(['status' => 'deleted']);
    }

    // ── DELETE /admin/alert ───────────────────────────────────────────────────

    public function alertDelete(): void
    {
        header('Content-Type: application/json');
        if (!$this->isAuthed()) { http_response_code(401); echo json_encode(['error' => 'unauthorized']); return; }

        $db = $this->db();
        if (!$db) { http_response_code(503); echo json_encode(['error' => 'service_indisponible']); return; }

        $body    = json_decode(file_get_contents('php://input'), true);
        $alertId = (int) ($body['id'] ?? 0);
        if (!$alertId) { http_response_code(422); echo json_encode(['error' => 'id_requis']); return; }

        $db->exec("DELETE FROM alerts WHERE id = ?", [$alertId]);
        echo json_encode(['status' => 'deleted']);
    }

    // ── POST /admin/alerte ────────────────────────────────────────────────────

    public function sendAlert(): void
    {
        header('Content-Type: application/json');

        if (!$this->isAuthed()) {
            http_response_code(401);
            echo json_encode(['error' => 'unauthorized']);
            return;
        }

        $raw  = file_get_contents('php://input');
        $body = json_decode($raw, true);

        $type        = trim((string) ($body['type'] ?? ''));
        $description = trim((string) ($body['description'] ?? ''));
        $networks    = array_values(array_filter((array) ($body['networks_concerned'] ?? [])));
        $emitted_by  = trim((string) ($body['emitted_by'] ?? 'RCC'));

        if (!$type || !$description) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants', 'requis' => ['type', 'description']]);
            return;
        }

        $db = $this->db();
        if (!$db) {
            http_response_code(503);
            echo json_encode(['error' => 'service_indisponible']);
            return;
        }

        try {
            $db->exec(
                "INSERT INTO alerts (type, description, network_cids, emitted_at, emitted_by)
                 VALUES (?, ?, ?, NOW(), ?)",
                [$type, $description, json_encode($networks), $emitted_by]
            );
        } catch (\Exception $e) {
            error_log('[Admin sendAlert] DB error: ' . $e->getMessage());
            http_response_code(500);
            echo json_encode(['error' => 'database_error']);
            return;
        }

        try {
            Mailer::sendAlertToAdmins($db, $type, $description, $networks);
        } catch (\Exception $e) {
            error_log('[Admin sendAlert] Mailer error: ' . $e->getMessage());
        }

        echo json_encode(['status' => 'alerte_enregistree']);
    }
}
