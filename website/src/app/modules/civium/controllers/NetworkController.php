<?php

/**
 * Réseau(x) Civium — interface membre.
 *
 * Toutes les routes /civium/* requièrent une session PHP (magic link).
 * Les routes /civium/api/* retournent du JSON.
 */
class NetworkController
{
    protected Base $f3;

    const PRINCIPAL_NETWORK_ID = 'civium-principal-000000000000000000000000000000000';

    public function __construct()
    {
        $this->f3 = Base::instance();
    }

    // ── Helpers ────────────────────────────────────────────────────────────────

    private function startSession(): void
    {
        if (session_status() === PHP_SESSION_NONE) {
            session_name('civium_sess');
            session_start();
        }
    }

    private function db(): ?\DB\SQL
    {
        return $this->f3->get('DB') ?: null;
    }

    private function email(): ?string
    {
        return $_SESSION['civium_email'] ?? null;
    }

    /** Réseau actif dans la session (ou réseau principal par défaut). */
    private function currentNetworkId(): string
    {
        return $_SESSION['civium_network_id'] ?? self::PRINCIPAL_NETWORK_ID;
    }

    /** Change le réseau actif dans la session. */
    private function setCurrentNetwork(string $networkId): void
    {
        $this->startSession();
        $_SESSION['civium_network_id'] = $networkId;
    }

    private function requireAuth(bool $api = false): bool
    {
        $this->startSession();
        if (!$this->email()) {
            if ($api) {
                header('Content-Type: application/json');
                http_response_code(401);
                echo json_encode(['error' => 'non_connecte']);
            } else {
                $this->f3->reroute('/auth');
            }
            return false;
        }
        return true;
    }

    /** Retourne la ligne web_members de l'utilisateur dans le réseau courant. */
    private function currentMember(?string $networkId = null): ?array
    {
        $db = $this->db();
        if (!$db || !$this->email()) return null;
        $nid = $networkId ?? $this->currentNetworkId();

        $rows = $db->exec(
            "SELECT * FROM web_members WHERE email = ? AND network_id = ? LIMIT 1",
            [$this->email(), $nid]
        );
        return $rows[0] ?? null;
    }

    /** Retourne tous les réseaux dont l'utilisateur est membre actif. */
    private function memberNetworks(): array
    {
        $db = $this->db();
        if (!$db || !$this->email()) return [];

        $rows = $db->exec(
            "SELECT n.id, n.name, n.description, n.is_public, n.admin_cid, n.created_at,
                    m.role, m.status, m.circle
             FROM web_networks n
             JOIN web_members m ON m.network_id = n.id
             WHERE m.email = ? AND m.status IN ('active','pending')
             ORDER BY n.created_at ASC",
            [$this->email()]
        );
        return $rows ?: [];
    }

    private function requireActiveMember(bool $api = false): ?array
    {
        $member = $this->currentMember();

        if (!$member) {
            if ($api) {
                header('Content-Type: application/json');
                http_response_code(403);
                echo json_encode(['error' => 'acces_refuse']);
            } else {
                $this->f3->error(403, 'Vous n\'êtes pas membre de ce réseau.');
            }
            return null;
        }

        if ($member['status'] === 'suspended') {
            if ($api) {
                header('Content-Type: application/json');
                http_response_code(403);
                echo json_encode(['error' => 'compte_suspendu']);
            } else {
                $this->f3->error(403, 'Votre compte a été suspendu.');
            }
            return null;
        }

        return $member;
    }

    // ── GET /civium/network ────────────────────────────────────────────────────

    public function dashboard(): void
    {
        if (!$this->requireAuth()) return;

        $member = $this->currentMember();

        if (!$member) {
            // Vérifier si l'utilisateur a d'autres réseaux
            $myNetworks = $this->memberNetworks();
            if (!empty($myNetworks)) {
                // Switcher vers le premier réseau disponible
                $this->setCurrentNetwork($myNetworks[0]['id']);
                $this->f3->reroute('/network');
                return;
            }
            // Pas de réseau → page de création / accès refusé
            $this->f3->set('titre', 'Aucun réseau');
            $this->f3->set('message', 'Vous n\'êtes membre d\'aucun réseau. Créez le vôtre ou demandez une invitation.');
            $this->f3->set('base', $this->f3->get('BASE'));
            $this->f3->set('show_create', true);
            echo Template::instance()->render('network-acces-refuse.html');
            return;
        }

        if ($member['status'] === 'suspended') {
            $this->f3->set('titre', 'Compte suspendu');
            $this->f3->set('message', 'Votre accès a été suspendu. Contactez l\'administrateur.');
            $this->f3->set('base', $this->f3->get('BASE'));
            echo Template::instance()->render('network-acces-refuse.html');
            return;
        }

        if (!$member['cid_full']) {
            $this->f3->set('member_email', $this->email());
            $this->f3->set('member_name', $member['display_name'] ?? '');
            $this->f3->set('base', $this->f3->get('BASE'));
            echo Template::instance()->render('network-identity.html');
            return;
        }

        // Charger le réseau courant
        $db = $this->db();
        $nets = $db->exec(
            "SELECT * FROM web_networks WHERE id = ? LIMIT 1",
            [$this->currentNetworkId()]
        );
        $network = $nets[0] ?? null;

        $myNetworks = $this->memberNetworks();
        $this->f3->set('member', $member);
        $this->f3->set('network', $network);
        $this->f3->set('network_id', htmlspecialchars($network['id'] ?? $this->currentNetworkId(), ENT_QUOTES));
        $this->f3->set('network_name', htmlspecialchars($network['name'] ?? 'Réseau Civium', ENT_QUOTES));
        $this->f3->set('my_networks', $myNetworks);
        $this->f3->set('my_networks_json', json_encode($myNetworks, JSON_UNESCAPED_UNICODE));
        $this->f3->set('base', $this->f3->get('BASE'));
        echo Template::instance()->render('network-dashboard.html');
    }

    // ── POST /civium/api/network/switch ───────────────────────────────────────

    public function switchNetwork(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;

        $raw  = file_get_contents('php://input');
        $body = json_decode($raw, true) ?? [];
        $nid  = trim((string) ($body['network_id'] ?? ''));

        if (!$nid) {
            http_response_code(422);
            echo json_encode(['error' => 'network_id_manquant']);
            return;
        }

        // Vérifie que l'utilisateur est bien membre de ce réseau
        $member = $this->currentMember($nid);
        if (!$member || $member['status'] === 'suspended') {
            http_response_code(403);
            echo json_encode(['error' => 'acces_refuse']);
            return;
        }

        $this->setCurrentNetwork($nid);
        echo json_encode(['ok' => true, 'network_id' => $nid]);
    }

    // ── POST /civium/api/network/create ───────────────────────────────────────

    public function createNetwork(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;

        $raw  = file_get_contents('php://input');
        $body = json_decode($raw, true) ?? [];

        $name        = trim((string) ($body['name']        ?? ''));
        $description = trim((string) ($body['description'] ?? ''));
        $isPublic    = (bool) ($body['is_public'] ?? false);

        if (!$name || mb_strlen($name) > 255) {
            http_response_code(422);
            echo json_encode(['error' => 'nom_invalide']);
            return;
        }

        // Récupère le CID de l'utilisateur depuis n'importe quel de ses réseaux
        $db = $this->db();
        $rows = $db->exec(
            "SELECT cid_short, cid_full, display_name FROM web_members WHERE email = ? AND cid_short IS NOT NULL LIMIT 1",
            [$this->email()]
        );
        $identity = $rows[0] ?? null;

        if (!$identity) {
            http_response_code(403);
            echo json_encode(['error' => 'identite_requise', 'message' => 'Configurez votre identité avant de créer un réseau.']);
            return;
        }

        // Crée le réseau
        $networkId = $this->generateUuid();
        $db->exec(
            "INSERT INTO web_networks (id, name, description, admin_cid, admin_email, is_public) VALUES (?, ?, ?, ?, ?, ?)",
            [$networkId, $name, $description ?: null, $identity['cid_short'], $this->email(), $isPublic ? 1 : 0]
        );

        // Ajoute le créateur comme admin actif
        $db->exec(
            "INSERT INTO web_members (network_id, email, cid_short, cid_full, display_name, role, status, joined_at) VALUES (?, ?, ?, ?, ?, 'admin', 'active', NOW())",
            [$networkId, $this->email(), $identity['cid_short'], $identity['cid_full'], $identity['display_name']]
        );

        // Événement d'activité
        $db->exec(
            "INSERT INTO web_activity (network_id, kind, actor_cid, actor_name, summary) VALUES (?, 'network_created', ?, ?, ?)",
            [$networkId, $identity['cid_short'], $identity['display_name'], $identity['display_name'] . ' a créé le réseau « ' . $name . ' »']
        );

        // Switcher vers ce nouveau réseau
        $this->setCurrentNetwork($networkId);

        echo json_encode(['ok' => true, 'network_id' => $networkId, 'name' => $name]);
    }

    // ── GET /civium/api/network/list ──────────────────────────────────────────

    public function listMyNetworks(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;

        echo json_encode($this->memberNetworks());
    }

    // ── GET /civium/join ───────────────────────────────────────────────────────

    public function joinPage(): void
    {
        $this->startSession();
        $token = (string) ($this->f3->get('GET.token') ?? '');

        $db = $this->db();
        if (!$db || !$token) {
            $this->f3->error(400, 'Lien d\'invitation invalide.');
            return;
        }

        $rows = $db->exec(
            "SELECT * FROM web_invitations WHERE token = ? AND used = 0 AND expires_at > NOW() LIMIT 1",
            [$token]
        );

        if (empty($rows)) {
            $this->f3->set('titre', 'Invitation invalide');
            $this->f3->set('message', 'Ce lien d\'invitation a expiré ou a déjà été utilisé.');
            $this->f3->set('base', $this->f3->get('BASE'));
            echo Template::instance()->render('network-acces-refuse.html');
            return;
        }

        $invitation = $rows[0];
        $email      = $invitation['email'];
        $networkId  = $invitation['network_id'];

        $db->exec(
            "INSERT IGNORE INTO web_members (network_id, email, role, status, invited_by) VALUES (?, ?, 'member', 'pending', ?)",
            [$networkId, $email, $invitation['invited_by']]
        );

        $db->exec(
            "UPDATE web_invitations SET used = 1, used_at = NOW() WHERE token = ?",
            [$token]
        );

        if ($this->email() === $email) {
            $this->setCurrentNetwork($networkId);
            $this->f3->reroute('/network');
            return;
        }

        $this->f3->set('invitation_email', htmlspecialchars($email, ENT_QUOTES));
        $this->f3->set('base', $this->f3->get('BASE'));
        echo Template::instance()->render('network-join.html');
    }

    // ── POST /civium/api/identity ──────────────────────────────────────────────

    public function setIdentity(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;

        $member = $this->currentMember();
        if (!$member) {
            http_response_code(403);
            echo json_encode(['error' => 'acces_refuse']);
            return;
        }

        $raw  = file_get_contents('php://input');
        $body = json_decode($raw, true) ?? [];

        $cidShort    = trim((string) ($body['cid_short']    ?? ''));
        $cidFull     = trim((string) ($body['cid_full']     ?? ''));
        $displayName = trim((string) ($body['display_name'] ?? ''));

        if (!$cidShort || !$cidFull || !$displayName) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        if (!preg_match('/^civ1[1-9A-HJ-NP-Za-km-z]+$/', $cidFull)) {
            http_response_code(422);
            echo json_encode(['error' => 'cid_invalide']);
            return;
        }

        $db        = $this->db();
        $networkId = $this->currentNetworkId();

        // Vérifie que le CID n'est pas déjà utilisé dans ce réseau par un autre membre
        $conflict = $db->exec(
            "SELECT id FROM web_members WHERE cid_full = ? AND email != ? AND network_id = ? LIMIT 1",
            [$cidFull, $this->email(), $networkId]
        );
        if (!empty($conflict)) {
            http_response_code(409);
            echo json_encode(['error' => 'cid_deja_utilise']);
            return;
        }

        $db->exec(
            "UPDATE web_members SET cid_short = ?, cid_full = ?, display_name = ?, status = 'active', joined_at = COALESCE(joined_at, NOW())
             WHERE email = ? AND network_id = ?",
            [$cidShort, $cidFull, $displayName, $this->email(), $networkId]
        );

        $db->exec(
            "INSERT INTO web_activity (network_id, kind, actor_cid, actor_name, summary) VALUES (?, 'member_joined', ?, ?, ?)",
            [$networkId, $cidShort, $displayName, $displayName . ' a rejoint le réseau']
        );

        echo json_encode(['ok' => true, 'cid_short' => $cidShort]);
    }

    // ── GET /civium/api/members ────────────────────────────────────────────────

    public function getMembers(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;
        if (!$this->requireActiveMember(true)) return;

        $db  = $this->db();
        $nid = $this->currentNetworkId();
        $rows = $db->exec(
            "SELECT cid_short, display_name, role, circle, status, joined_at FROM web_members WHERE network_id = ? AND status = 'active' ORDER BY joined_at ASC",
            [$nid]
        );
        echo json_encode($rows ?: []);
    }

    // ── GET /civium/api/messages ───────────────────────────────────────────────

    public function getMessages(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;
        if (!$this->requireActiveMember(true)) return;

        $db    = $this->db();
        $nid   = $this->currentNetworkId();
        $since = (string) ($this->f3->get('GET.since') ?? '');

        $sql    = "SELECT id, author_cid, author_name, body, sent_at FROM web_messages WHERE network_id = ?";
        $params = [$nid];
        if ($since) {
            $sql    .= " AND sent_at > ?";
            $params[] = $since;
        }
        $sql .= " ORDER BY sent_at DESC LIMIT 50";

        $rows = $db->exec($sql, $params);
        echo json_encode($rows ?: []);
    }

    // ── POST /civium/api/message ───────────────────────────────────────────────

    public function postMessage(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;
        $member = $this->requireActiveMember(true);
        if (!$member) return;

        $raw  = file_get_contents('php://input');
        $body = json_decode($raw, true) ?? [];
        $text = trim((string) ($body['body'] ?? ''));

        if (!$text || mb_strlen($text) > 5000) {
            http_response_code(422);
            echo json_encode(['error' => 'message_invalide']);
            return;
        }

        $db  = $this->db();
        $nid = $this->currentNetworkId();
        $db->exec(
            "INSERT INTO web_messages (network_id, author_cid, author_name, body) VALUES (?, ?, ?, ?)",
            [$nid, $member['cid_short'], $member['display_name'], $text]
        );

        $db->exec(
            "INSERT INTO web_activity (network_id, kind, actor_cid, actor_name, summary) VALUES (?, 'message_posted', ?, ?, ?)",
            [$nid, $member['cid_short'], $member['display_name'], $member['display_name'] . ' a posté un message']
        );

        echo json_encode(['ok' => true]);
    }

    // ── GET /civium/api/dms ────────────────────────────────────────────────────

    public function getDms(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;
        $member = $this->requireActiveMember(true);
        if (!$member) return;

        $db  = $this->db();
        $nid = $this->currentNetworkId();
        $cid = $member['cid_short'];

        $rows = $db->exec(
            "SELECT id, from_cid, to_cid, body, sent_at FROM web_direct_messages WHERE network_id = ? AND (from_cid = ? OR to_cid = ?) ORDER BY sent_at DESC LIMIT 100",
            [$nid, $cid, $cid]
        );
        echo json_encode($rows ?: []);
    }

    // ── POST /civium/api/dm ────────────────────────────────────────────────────

    public function postDm(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;
        $member = $this->requireActiveMember(true);
        if (!$member) return;

        $raw   = file_get_contents('php://input');
        $body  = json_decode($raw, true) ?? [];
        $toCid = trim((string) ($body['to_cid'] ?? ''));
        $text  = trim((string) ($body['body']   ?? ''));

        if (!$toCid || !$text || mb_strlen($text) > 5000) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_invalides']);
            return;
        }

        $db  = $this->db();
        $nid = $this->currentNetworkId();
        $db->exec(
            "INSERT INTO web_direct_messages (network_id, from_cid, to_cid, body) VALUES (?, ?, ?, ?)",
            [$nid, $member['cid_short'], $toCid, $text]
        );

        echo json_encode(['ok' => true]);
    }

    // ── GET /civium/api/proposals ──────────────────────────────────────────────

    public function getProposals(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;
        if (!$this->requireActiveMember(true)) return;

        $db   = $this->db();
        $nid  = $this->currentNetworkId();
        $rows = $db->exec(
            "SELECT p.*, (SELECT COUNT(*) FROM web_votes v WHERE v.proposal_id = p.id) AS vote_count
             FROM web_proposals p WHERE p.network_id = ? ORDER BY p.created_at DESC LIMIT 50",
            [$nid]
        );
        foreach ($rows as &$r) {
            $r['options'] = json_decode($r['options_json'], true);
            unset($r['options_json']);
        }
        echo json_encode($rows ?: []);
    }

    // ── POST /civium/api/proposal ──────────────────────────────────────────────

    public function createProposal(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;
        $member = $this->requireActiveMember(true);
        if (!$member) return;

        $raw  = file_get_contents('php://input');
        $body = json_decode($raw, true) ?? [];

        $title       = trim((string) ($body['title']       ?? ''));
        $description = trim((string) ($body['description'] ?? ''));
        $options     = $body['options']    ?? [];
        $daysOpen    = max(1, min(30, (int) ($body['days_open'] ?? 7)));
        $quorum      = max(1, min(100, (int) ($body['quorum']  ?? 50)));

        if (!$title || count($options) < 2) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_invalides']);
            return;
        }

        $db       = $this->db();
        $nid      = $this->currentNetworkId();
        $closesAt = date('Y-m-d H:i:s', strtotime("+{$daysOpen} days"));

        $db->exec(
            "INSERT INTO web_proposals (network_id, title, description, options_json, created_by, closes_at, quorum_percent) VALUES (?, ?, ?, ?, ?, ?, ?)",
            [$nid, $title, $description, json_encode(array_values($options)), $member['cid_short'], $closesAt, $quorum]
        );

        $db->exec(
            "INSERT INTO web_activity (network_id, kind, actor_cid, actor_name, summary) VALUES (?, 'proposal_created', ?, ?, ?)",
            [$nid, $member['cid_short'], $member['display_name'], $member['display_name'] . ' a créé une proposition : ' . $title]
        );

        echo json_encode(['ok' => true]);
    }

    // ── POST /civium/api/vote ──────────────────────────────────────────────────

    public function castVote(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;
        $member = $this->requireActiveMember(true);
        if (!$member) return;

        $raw  = file_get_contents('php://input');
        $body = json_decode($raw, true) ?? [];

        $proposalId = trim((string) ($body['proposal_id'] ?? ''));
        $optionIdx  = (int) ($body['option_idx'] ?? -1);

        if (!$proposalId || $optionIdx < 0) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_invalides']);
            return;
        }

        $db  = $this->db();
        $nid = $this->currentNetworkId();

        $prop = $db->exec(
            "SELECT * FROM web_proposals WHERE id = ? AND network_id = ? AND status = 'open' AND closes_at > NOW() LIMIT 1",
            [$proposalId, $nid]
        );
        if (empty($prop)) {
            http_response_code(404);
            echo json_encode(['error' => 'proposition_introuvable_ou_fermee']);
            return;
        }

        try {
            $db->exec(
                "INSERT INTO web_votes (network_id, proposal_id, voter_cid, option_idx) VALUES (?, ?, ?, ?)",
                [$nid, $proposalId, $member['cid_short'], $optionIdx]
            );
        } catch (\Exception $e) {
            http_response_code(409);
            echo json_encode(['error' => 'deja_vote']);
            return;
        }

        echo json_encode(['ok' => true]);
    }

    // ── GET /civium/api/directory ──────────────────────────────────────────────

    public function getDirectory(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;
        if (!$this->requireActiveMember(true)) return;

        $db  = $this->db();
        $nid = $this->currentNetworkId();
        $q   = trim((string) ($this->f3->get('GET.q') ?? ''));

        $sql    = "SELECT * FROM web_directory_entries WHERE network_id = ?";
        $params = [$nid];
        if ($q) {
            $sql    .= " AND (subject_name LIKE ? OR description LIKE ? OR tags LIKE ?)";
            $like    = '%' . $q . '%';
            $params  = [$nid, $like, $like, $like];
        }
        $sql .= " ORDER BY published_at DESC LIMIT 100";

        $rows = $db->exec($sql, $params);
        foreach ($rows as &$r) {
            $r['tags'] = $r['tags'] ? array_map('trim', explode(',', $r['tags'])) : [];
        }
        echo json_encode($rows ?: []);
    }

    // ── POST /civium/api/directory ─────────────────────────────────────────────

    public function publishEntry(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;
        $member = $this->requireActiveMember(true);
        if (!$member) return;

        $raw  = file_get_contents('php://input');
        $body = json_decode($raw, true) ?? [];

        $kind        = in_array($body['kind'] ?? '', ['network', 'member']) ? $body['kind'] : null;
        $subjectCid  = trim((string) ($body['subject_cid']  ?? ''));
        $subjectName = trim((string) ($body['subject_name'] ?? ''));
        $description = trim((string) ($body['description']  ?? ''));
        $tags        = trim((string) ($body['tags']         ?? ''));

        if (!$kind || !$subjectCid || !$subjectName) {
            http_response_code(422);
            echo json_encode(['error' => 'champs_manquants']);
            return;
        }

        $db  = $this->db();
        $nid = $this->currentNetworkId();
        $db->exec(
            "INSERT INTO web_directory_entries (network_id, kind, subject_cid, subject_name, description, tags, published_by) VALUES (?, ?, ?, ?, ?, ?, ?)",
            [$nid, $kind, $subjectCid, $subjectName, $description, $tags, $member['cid_short']]
        );

        echo json_encode(['ok' => true]);
    }

    // ── GET /civium/api/notifications ─────────────────────────────────────────

    public function getNotifications(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;
        $member = $this->requireActiveMember(true);
        if (!$member) return;

        $db  = $this->db();
        $nid = $this->currentNetworkId();
        $rows = $db->exec(
            "SELECT n.id, n.read_flag, n.created_at, a.kind, a.actor_name, a.summary
             FROM web_notifications n
             JOIN web_activity a ON a.id = n.activity_id
             WHERE n.network_id = ? AND n.member_cid = ?
             ORDER BY n.created_at DESC LIMIT 50",
            [$nid, $member['cid_short']]
        );
        echo json_encode($rows ?: []);
    }

    // ── POST /civium/api/notifications/read ───────────────────────────────────

    public function markRead(): void
    {
        header('Content-Type: application/json');
        if (!$this->requireAuth(true)) return;
        $member = $this->requireActiveMember(true);
        if (!$member) return;

        $raw  = file_get_contents('php://input');
        $body = json_decode($raw, true) ?? [];
        $id   = trim((string) ($body['id'] ?? ''));

        $db  = $this->db();
        $nid = $this->currentNetworkId();
        if ($id) {
            $db->exec(
                "UPDATE web_notifications SET read_flag = 1 WHERE id = ? AND member_cid = ? AND network_id = ?",
                [$id, $member['cid_short'], $nid]
            );
        } else {
            $db->exec(
                "UPDATE web_notifications SET read_flag = 1 WHERE member_cid = ? AND network_id = ?",
                [$member['cid_short'], $nid]
            );
        }

        echo json_encode(['ok' => true]);
    }

    // ── Helpers ────────────────────────────────────────────────────────────────

    private function generateUuid(): string
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
}
