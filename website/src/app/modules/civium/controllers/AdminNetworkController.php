<?php

/**
 * Administration du réseau principal web.
 *
 * GET  /admin/network                       → liste des membres
 * POST /admin/network/invite                → inviter un membre
 * POST /admin/network/member/suspend        → suspendre un membre
 * POST /admin/network/member/reactivate     → réactiver un membre
 */
class AdminNetworkController
{
    protected Base $f3;

    const PRINCIPAL_NETWORK_ID = 'civium-principal-000000000000000000000000000000000';

    public function __construct()
    {
        $this->f3 = Base::instance();
        if (session_status() === PHP_SESSION_NONE) {
            session_start();
        }
    }

    private function isAuthed(): bool
    {
        $admin = !empty($_SESSION['civium_admin']);
        $token = (string) $this->f3->get('ADMIN_TOKEN');
        $provided = $_SERVER['HTTP_X_ADMIN_TOKEN'] ?? $this->f3->get('GET.token') ?? '';
        return $admin || ($token && hash_equals($token, (string) $provided));
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

    private function jsonError(int $code, string $error): void
    {
        header('Content-Type: application/json');
        http_response_code($code);
        echo json_encode(['error' => $error]);
    }

    // ── GET /admin/network ────────────────────────────────────────────────────

    public function index(): void
    {
        if (!$this->requireAuth()) return;

        $db = $this->db();
        $members = $db ? $db->exec(
            "SELECT m.*, n.name AS network_name
             FROM web_members m
             JOIN web_networks n ON n.id = m.network_id
             WHERE m.network_id = ?
             ORDER BY m.created_at DESC",
            [self::PRINCIPAL_NETWORK_ID]
        ) : [];

        $this->f3->set('title', 'Gestion du réseau — Administration');
        $this->f3->set('members', $members ?: []);
        $this->f3->set('base', $this->f3->get('BASE'));
        echo Template::instance()->render('admin.html');
    }

    // ── POST /admin/network/invite ─────────────────────────────────────────────

    public function invite(): void
    {
        header('Content-Type: application/json');
        if (!$this->isAuthed()) { $this->jsonError(403, 'non_autorise'); return; }

        $db = $this->db();
        if (!$db) { $this->jsonError(503, 'service_indisponible'); return; }

        $email = filter_var(trim((string) $this->f3->get('POST.email')), FILTER_VALIDATE_EMAIL);
        if (!$email) { $this->jsonError(422, 'email_invalide'); return; }

        $exists = $db->exec(
            "SELECT COUNT(*) AS n FROM web_members WHERE email = ? AND network_id = ?",
            [$email, self::PRINCIPAL_NETWORK_ID]
        );
        if ((int) ($exists[0]['n'] ?? 0) > 0) {
            $this->jsonError(409, 'membre_existant');
            return;
        }

        $db->exec(
            "INSERT INTO web_members (network_id, email, role, status) VALUES (?, ?, 'member', 'pending')",
            [self::PRINCIPAL_NETWORK_ID, $email]
        );

        echo json_encode(['status' => 'invite_ajoute']);
    }

    // ── POST /admin/network/member/suspend ────────────────────────────────────

    public function suspend(): void
    {
        header('Content-Type: application/json');
        if (!$this->isAuthed()) { $this->jsonError(403, 'non_autorise'); return; }

        $db = $this->db();
        if (!$db) { $this->jsonError(503, 'service_indisponible'); return; }

        $email = trim((string) $this->f3->get('POST.email'));
        if (!$email) { $this->jsonError(422, 'email_manquant'); return; }

        $db->exec(
            "UPDATE web_members SET status = 'suspended' WHERE email = ? AND network_id = ?",
            [$email, self::PRINCIPAL_NETWORK_ID]
        );

        echo json_encode(['status' => 'suspendu']);
    }

    // ── POST /admin/network/member/reactivate ─────────────────────────────────

    public function reactivate(): void
    {
        header('Content-Type: application/json');
        if (!$this->isAuthed()) { $this->jsonError(403, 'non_autorise'); return; }

        $db = $this->db();
        if (!$db) { $this->jsonError(503, 'service_indisponible'); return; }

        $email = trim((string) $this->f3->get('POST.email'));
        if (!$email) { $this->jsonError(422, 'email_manquant'); return; }

        $db->exec(
            "UPDATE web_members SET status = 'active' WHERE email = ? AND network_id = ?",
            [$email, self::PRINCIPAL_NETWORK_ID]
        );

        echo json_encode(['status' => 'reactivé']);
    }
}
