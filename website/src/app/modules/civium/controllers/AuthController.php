<?php

/**
 * Authentification web via magic link.
 *
 * GET  /auth            → formulaire de connexion
 * POST /auth            → demande de lien (JSON)
 * GET  /auth/verify     → validation du token, création de session
 * GET  /auth/deconnexion → destruction de session
 */
class AuthController
{
    protected Base $f3;

    public function __construct()
    {
        $this->f3 = Base::instance();
    }

    private function startSession(): void
    {
        if (session_status() === PHP_SESSION_NONE) {
            $isHttps = (!empty($_SERVER['HTTPS']) && $_SERVER['HTTPS'] !== 'off')
                    || (($_SERVER['SERVER_PORT'] ?? 80) == 443);
            session_name('civium_sess');
            session_set_cookie_params([
                'lifetime' => 0,
                'path'     => '/',
                'secure'   => $isHttps,
                'httponly' => true,
                'samesite' => $isHttps ? 'Strict' : 'Lax',
            ]);
            session_start();
        }
    }

    private function csrfToken(): string
    {
        if (empty($_SESSION['csrf_token'])) {
            $_SESSION['csrf_token'] = bin2hex(random_bytes(32));
        }
        return $_SESSION['csrf_token'];
    }

    private function verifyCsrf(array $body): bool
    {
        $token = (string) ($body['_csrf'] ?? '');
        return !empty($_SESSION['csrf_token'])
            && hash_equals($_SESSION['csrf_token'], $token);
    }

    private function db(): ?\DB\SQL
    {
        return $this->f3->get('DB') ?: null;
    }

    // ── GET /auth ──────────────────────────────────────────────────────────────

    public function loginPage(): void
    {
        $this->startSession();

        if (!empty($_SESSION['civium_email'])) {
            $this->f3->reroute('/app');
            return;
        }

        $this->csrfToken(); // génère le token dans la session

        $erreur = (string) ($this->f3->get('GET.erreur') ?? '');
        $messages = [
            'lien_expire'          => 'Ce lien de connexion a expiré ou a déjà été utilisé.',
            'lien_invalide'        => 'Lien de connexion invalide.',
            'service_indisponible' => 'Service temporairement indisponible.',
        ];

        $this->f3->set('title', 'Connexion — Civium');
        $this->f3->set('erreur', $erreur);
        $this->f3->set('erreur_msg', htmlspecialchars($messages[$erreur] ?? $erreur, ENT_QUOTES));
        $this->f3->set('csrf_token', $this->csrfToken());

        echo Template::instance()->render('auth.html');
    }

    // ── POST /auth ─────────────────────────────────────────────────────────────

    public function request(): void
    {
        header('Content-Type: application/json');

        $raw      = file_get_contents('php://input', false, null, 0, 8192);
        $body     = json_decode($raw, true);

        $this->startSession();
        if (!$this->verifyCsrf($body ?? [])) {
            http_response_code(403);
            echo json_encode(['error' => 'csrf_invalide']);
            return;
        }

        $email    = filter_var(trim((string) ($body['email']    ?? '')), FILTER_VALIDATE_EMAIL);
        $password = (string) ($body['password'] ?? '');
        $mode     = (string) ($body['mode']     ?? 'magic');

        if (!$email) {
            http_response_code(422);
            echo json_encode(['error' => 'email_invalide']);
            return;
        }

        $db = $this->db();
        if (!$db) {
            http_response_code(503);
            echo json_encode(['error' => 'service_indisponible']);
            return;
        }

        // Rate limiting par IP : max 20 requêtes par heure
        $ip = $_SERVER['REMOTE_ADDR'] ?? '0.0.0.0';
        $action = $mode === 'password' ? 'password' : 'magic_link';
        try {
            $countRow = $db->exec(
                "SELECT COUNT(*) AS n FROM auth_attempts WHERE ip = ? AND action = ? AND created_at >= DATE_SUB(NOW(), INTERVAL 1 HOUR)",
                [$ip, $action]
            );
            if ((int) ($countRow[0]['n'] ?? 0) >= 20) {
                http_response_code(429);
                echo json_encode(['error' => 'trop_de_demandes', 'retry_after' => 3600]);
                return;
            }
            $db->exec("INSERT INTO auth_attempts (ip, action) VALUES (?, ?)", [$ip, $action]);
        } catch (\Exception $e) {
            // Table pas encore créée (migration en cours) — on continue sans rate limit
            error_log('[Auth] rate limit check failed: ' . $e->getMessage());
        }

        if ($mode === 'password') {
            $this->handlePasswordAuth($db, $email, $password);
            return;
        }

        // Mode magic link (défaut)
        try {
            $token = MagicLink::create($db, $email);
        } catch (\RuntimeException $e) {
            if ($e->getMessage() === 'rate_limited') {
                http_response_code(429);
                echo json_encode(['error' => 'trop_de_demandes', 'retry_after' => 3600]);
                return;
            }
            throw $e;
        }
        MagicLink::send($this->f3, $email, $token);

        echo json_encode(['status' => 'lien_envoye']);
    }

    /**
     * Connexion ou inscription par mot de passe.
     * - Compte existant : vérifie le hash → crée la session
     * - Nouveau compte  : crée le compte, un réseau, envoie l'email de bienvenue
     */
    private function handlePasswordAuth(\DB\SQL $db, string $email, string $password): void
    {
        if (strlen($password) < 8) {
            http_response_code(422);
            echo json_encode(['error' => 'mot_de_passe_trop_court']);
            return;
        }

        $rows = $db->exec(
            "SELECT password_hash FROM web_users WHERE email = ? LIMIT 1",
            [$email]
        );

        if ($rows && !empty($rows[0]['password_hash'])) {
            // Compte existant — vérifier le mot de passe
            if (!password_verify($password, $rows[0]['password_hash'])) {
                sleep(1); // Anti-bruteforce
                http_response_code(401);
                echo json_encode(['error' => 'identifiants_invalides']);
                return;
            }

            $this->startSession();
            session_regenerate_id(true);
            $_SESSION['civium_email'] = $email;
            $this->ensureNetworkForNewUser($db, $email);
            echo json_encode(['status' => 'connecte']);
            return;
        }

        // Nouveau compte — créer l'utilisateur et son réseau
        $hash = password_hash($password, PASSWORD_BCRYPT);
        $db->exec(
            "INSERT INTO web_users (email, password_hash) VALUES (?, ?)",
            [$email, $hash]
        );

        $networkName = $this->networkNameFromEmail($email);
        $networkId   = $this->generateNetworkId();

        $db->exec(
            "INSERT INTO web_networks (id, name, admin_email, is_public) VALUES (?, ?, ?, 0)",
            [$networkId, $networkName, $email]
        );
        $db->exec(
            "INSERT INTO web_members (network_id, email, role, status) VALUES (?, ?, 'admin', 'pending')",
            [$networkId, $email]
        );

        $this->startSession();
        session_regenerate_id(true);
        $_SESSION['civium_email']    = $email;
        $_SESSION['civium_network_id'] = $networkId;

        // Email de bienvenue (non bloquant)
        try {
            Mailer::sendWelcome($this->f3, $email, $networkName);
        } catch (\Exception $e) {
            error_log('[Auth] Welcome email failed: ' . $e->getMessage());
        }

        echo json_encode(['status' => 'compte_cree']);
    }

    private function networkNameFromEmail(string $email): string
    {
        $local = explode('@', $email)[0];
        return mb_substr(ucfirst(preg_replace('/[^a-zA-Z0-9]/', ' ', $local)), 0, 80) . ' — réseau Civium';
    }

    // ── GET /auth/verify ───────────────────────────────────────────────────────

    public function verify(): void
    {
        $this->startSession();

        $token = (string) ($this->f3->get('GET.token') ?? '');
        if (!$token) {
            $this->f3->reroute('/auth?erreur=lien_invalide');
            return;
        }

        $db = $this->db();
        if (!$db) {
            $this->f3->reroute('/auth?erreur=service_indisponible');
            return;
        }

        $email = MagicLink::validate($db, $token);
        if (!$email) {
            $this->f3->reroute('/auth?erreur=lien_expire');
            return;
        }

        session_regenerate_id(true);
        $_SESSION['civium_email'] = $email;

        // Si l'utilisateur n'est membre d'aucun réseau, créer un réseau personnel
        $this->ensureNetworkForNewUser($db, $email);

        $this->f3->reroute('/network');
    }

    /**
     * Crée un réseau personnel pour un nouvel utilisateur s'il n'en a aucun.
     */
    private function ensureNetworkForNewUser(\DB\SQL $db, string $email): void
    {
        $existing = $db->exec(
            "SELECT COUNT(*) AS n FROM web_members WHERE email = ? AND status IN ('active','pending')",
            [$email]
        );
        if ((int) ($existing[0]['n'] ?? 0) > 0) {
            return;
        }

        $networkName = $this->networkNameFromEmail($email);
        $networkId   = $this->generateNetworkId();

        $db->exec(
            "INSERT INTO web_networks (id, name, admin_email, is_public) VALUES (?, ?, ?, 0)",
            [$networkId, mb_substr($networkName, 0, 128), $email]
        );

        $db->exec(
            "INSERT INTO web_members (network_id, email, role, status) VALUES (?, ?, 'admin', 'pending')",
            [$networkId, $email]
        );

        $_SESSION['civium_network_id'] = $networkId;
    }

    private function generateNetworkId(): string
    {
        return sprintf(
            '%04x%04x-%04x-%04x-%04x-%04x%04x%04x',
            mt_rand(0, 0xffff), mt_rand(0, 0xffff),
            mt_rand(0, 0xffff),
            mt_rand(0, 0x0fff) | 0x4000,
            mt_rand(0, 0x3fff) | 0x8000,
            mt_rand(0, 0xffff), mt_rand(0, 0xffff), mt_rand(0, 0xffff)
        );
    }

    // ── GET /auth/deconnexion ─────────────────────────────────────────────────

    public function logout(): void
    {
        $this->startSession();
        $_SESSION = [];
        session_destroy();
        $this->f3->reroute('/auth');
    }
}
