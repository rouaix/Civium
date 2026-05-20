<?php

/**
 * White-label Civium — personnalisation et licences par taille d'organisation.
 *
 * GET  /api/info                → info publique de l'instance (branding)
 * GET  /admin/white-label       → page de gestion admin (protégée)
 * POST /admin/white-label       → enregistre les paramètres (protégée)
 *
 * Tiers de licence :
 *   open        — pas de limite (instance publique Civium)
 *   famille     — max 3 réseaux, 50 membres
 *   association — max 20 réseaux, 500 membres
 *   entreprise  — limites configurables
 */
class WhiteLabelController
{
    protected Base $f3;

    public function __construct()
    {
        $this->f3 = Base::instance();
        if (session_status() === PHP_SESSION_NONE) {
            session_start();
        }
    }

    private function db(): ?\DB\SQL
    {
        return $this->f3->get('DB') ?: null;
    }

    private function isAuthed(): bool
    {
        if (!empty($_SESSION['civium_admin'])) return true;

        $token = (string) $this->f3->get('ADMIN_TOKEN');
        if (!$token) return false;

        $provided = $_SERVER['HTTP_X_ADMIN_TOKEN']
                 ?? $this->f3->get('GET.token')
                 ?? '';

        return hash_equals($token, (string) $provided);
    }

    // ── Lecture des paramètres depuis la BDD ──────────────────────────────────

    public static function loadSettings(\DB\SQL $db): array
    {
        $defaults = [
            'app_name'      => 'Civium',
            'app_tagline'   => 'Votre réseau souverain',
            'app_logo_url'  => '',
            'license_tier'  => 'open',
            'max_networks'  => '0',
            'max_members'   => '0',
            'contact_email' => '',
            'support_url'   => '',
            'primary_color' => '#6366f1',
            'org_name'      => '',
            'custom_css'    => '',
        ];

        try {
            $rows = $db->exec("SELECT setting_key, setting_value FROM white_label_settings") ?: [];
            foreach ($rows as $row) {
                $defaults[$row['setting_key']] = $row['setting_value'];
            }
        } catch (\Exception $e) {
            // Table absente si migration non encore appliquée — on retourne les defaults
        }

        return $defaults;
    }

    // ── Limites par tier ──────────────────────────────────────────────────────

    public static function tierLimits(string $tier): array
    {
        return match ($tier) {
            'famille'     => ['max_networks' => 3,  'max_members' => 50],
            'association' => ['max_networks' => 20, 'max_members' => 500],
            default       => ['max_networks' => 0,  'max_members' => 0],   // open / entreprise = illimité (ou config manuelle)
        };
    }

    // ── GET /api/info ─────────────────────────────────────────────────────────

    public function info(): void
    {
        header('Content-Type: application/json');
        header('Access-Control-Allow-Origin: *');

        $db = $this->db();
        if (!$db) {
            echo json_encode([
                'app_name'     => 'Civium',
                'app_tagline'  => 'Votre réseau souverain',
                'license_tier' => 'open',
                'max_networks' => 0,
                'max_members'  => 0,
            ]);
            return;
        }

        $s = self::loadSettings($db);
        $limits = self::tierLimits($s['license_tier']);

        // max_networks / max_members : la config manuelle override le tier
        $maxNetworks = (int) $s['max_networks'] ?: $limits['max_networks'];
        $maxMembers  = (int) $s['max_members']  ?: $limits['max_members'];

        echo json_encode([
            'app_name'      => $s['app_name'],
            'app_tagline'   => $s['app_tagline'],
            'app_logo_url'  => $s['app_logo_url'],
            'primary_color' => $s['primary_color'],
            'org_name'      => $s['org_name'],
            'support_url'   => $s['support_url'],
            'contact_email' => $s['contact_email'],
            'license_tier'  => $s['license_tier'],
            'max_networks'  => $maxNetworks,
            'max_members'   => $maxMembers,
        ]);
    }

    // ── GET /admin/white-label ────────────────────────────────────────────────

    public function settingsPage(): void
    {
        if (!$this->isAuthed()) {
            $this->f3->reroute('/admin/login');
            return;
        }

        $db       = $this->db();
        $settings = $db ? self::loadSettings($db) : [];

        // Statistiques courantes
        $networkCount = 0;
        if ($db) {
            try {
                $row = $db->exec("SELECT COUNT(*) AS n FROM networks");
                $networkCount = (int) ($row[0]['n'] ?? 0);
            } catch (\Exception $e) {}
        }

        $this->f3->set('title', 'White-label — Administration Civium');
        $this->f3->set('wl', $settings);
        $this->f3->set('network_count', $networkCount);
        $this->f3->set('tiers', [
            'open'        => ['label' => 'Open (illimité)',    'icon' => '🌐'],
            'famille'     => ['label' => 'Famille (3 / 50)',   'icon' => '🏠'],
            'association' => ['label' => 'Association (20 / 500)', 'icon' => '🤝'],
            'entreprise'  => ['label' => 'Entreprise (config)', 'icon' => '🏢'],
        ]);

        echo Template::instance()->render('admin-whitelabel.html');
    }

    // ── POST /admin/white-label ───────────────────────────────────────────────

    public function settingsUpdate(): void
    {
        header('Content-Type: application/json');

        if (!$this->isAuthed()) {
            http_response_code(401);
            echo json_encode(['error' => 'unauthorized']);
            return;
        }

        $db = $this->db();
        if (!$db) {
            http_response_code(503);
            echo json_encode(['error' => 'service_indisponible']);
            return;
        }

        $raw  = file_get_contents('php://input');
        $body = json_decode($raw, true);
        if (!is_array($body)) {
            http_response_code(400);
            echo json_encode(['error' => 'invalid_json']);
            return;
        }

        $allowed = [
            'app_name', 'app_tagline', 'app_logo_url', 'license_tier',
            'max_networks', 'max_members', 'contact_email', 'support_url',
            'primary_color', 'org_name', 'custom_css',
        ];

        $validTiers = ['open', 'famille', 'association', 'entreprise'];
        if (isset($body['license_tier']) && !in_array($body['license_tier'], $validTiers, true)) {
            http_response_code(422);
            echo json_encode(['error' => 'tier_invalide']);
            return;
        }

        try {
            $db->begin();
            foreach ($allowed as $key) {
                if (!array_key_exists($key, $body)) continue;
                $value = (string) $body[$key];
                $db->exec(
                    "INSERT INTO white_label_settings (setting_key, setting_value)
                     VALUES (?, ?)
                     ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value), updated_at = NOW()",
                    [$key, $value]
                );
            }
            $db->commit();
        } catch (\Exception $e) {
            $db->rollback();
            error_log('[WhiteLabel] DB error: ' . $e->getMessage());
            http_response_code(500);
            echo json_encode(['error' => 'database_error']);
            return;
        }

        echo json_encode(['status' => 'saved']);
    }
}
