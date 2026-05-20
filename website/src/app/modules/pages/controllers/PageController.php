<?php

class PageController
{
    protected Base $f3;

    public function __construct()
    {
        $this->f3 = Base::instance();
    }

    protected function render(string $view, string $title, array $data = []): void
    {
        $this->f3->set('title', $title);
        $this->f3->mset($data);
        $this->f3->set('content', Template::instance()->render($view . '.html'));
        echo Template::instance()->render('layout.html');
    }

    public function home(): void
    {
        $this->render('home', 'Civium : Des réseaux souverains, connectés par choix');
    }

    public function how(): void
    {
        $this->render('how', 'Comment ça marche : Civium', [
            'plugins' => [
                'Messagerie', 'Agenda', 'Documents', 'Marketplace', 'Visioconférence',
                'Wiki', 'Sondages', 'Facturation', 'Gestion de projet', '+ des centaines d\'autres…',
            ],
        ]);
    }

    public function usecases(): void
    {
        $this->render('usecases', 'Cas d\'usage Civium', [
            'cases' => [
                [
                    'id' => 'famille', 'emoji' => '👨‍👩‍👧‍👦', 'title' => 'Famille',
                    'subtitle' => 'Remplace Google Photos, WhatsApp, Dropbox',
                    'desc' => 'Un espace privé où coexistent album photo partagé, agenda familial, coffre-fort de documents (actes, contrats), messagerie E2E, caisse commune tout en local, sans dépendance aux GAFAM.',
                    'features' => ['Album photo sur votre NAS ou Raspberry Pi', 'Agenda partagé entre tous les membres', 'Documents importants (actes, contrats, ordonnances)', 'Messagerie E2E entre membres de la famille', 'Contrôle parental pour les enfants'],
                ],
                [
                    'id' => 'association', 'emoji' => '🤝', 'title' => 'Association',
                    'subtitle' => 'Remplace Facebook Groups, Slack, HelloAsso',
                    'desc' => 'Gestion des membres, votes, agenda, communication interne, comptabilité, appels à projets, marketplace de services entre membres avec une gouvernance réelle, pas simulée.',
                    'features' => ['Adhésions et liste des membres', 'Votes collectifs avec quorum configurable', 'Agenda des événements et réunions', 'Communication interne chiffrée', 'Connexion avec des associations partenaires'],
                ],
                [
                    'id' => 'quartier', 'emoji' => '🏘️', 'title' => 'Quartier',
                    'subtitle' => 'Remplace Nextdoor, Facebook Quartier',
                    'desc' => 'Annuaire de voisinage, troc et dons, signalement de problèmes urbains, concertation citoyenne, événements locaux, covoiturage, bibliothèque partagée, connecté au réseau de la mairie si elle le souhaite.',
                    'features' => ['Annuaire des habitants (opt-in)', 'Petites annonces, troc, dons', 'Événements et sorties de quartier', 'Concertation et budgets participatifs', 'Connexion avec la mairie (si accord)'],
                ],
                [
                    'id' => 'entreprise', 'emoji' => '🏢', 'title' => 'Entreprise',
                    'subtitle' => 'Remplace Slack, Notion, Teams',
                    'desc' => 'Gestion de projets, documents partagés, facturation, RH, communication interne, chaque connexion avec un prestataire externe contractualisée, chaque accès audité.',
                    'features' => ['Communication interne chiffrée', 'Gestion de projets et documents', 'Connexions sécurisées avec prestataires', 'Audit complet des accès', 'Synchronisation optionnelle avec Slack, Notion…'],
                ],
            ],
        ]);
    }

    public function roadmap(): void
    {
        $this->render('roadmap', 'Feuille de route Civium', [
            'phases' => [
                [
                    'number' => '0', 'label' => 'Phase 0', 'title' => 'Site web de présentation',
                    'status' => 'Terminée', 'is_first' => false, 'is_last' => false,
                    'items' => ['Page d\'accueil et proposition de valeur', 'Cas d\'usage par type de communauté', 'Feuille de route publique', 'Formulaire d\'inscription liste d\'attente'],
                ],
                [
                    'number' => '1', 'label' => 'Phase 1', 'title' => 'MVP : Protocole de base',
                    'status' => 'Terminée', 'is_first' => false, 'is_last' => false,
                    'items' => ['Identité cryptographique (CID Ed25519)', 'Transport P2P (libp2p / DHT)', 'Messagerie E2E dans un réseau', 'Connexion inter-réseaux avec accord signé', 'Application Desktop (Tauri) + CLI'],
                ],
                [
                    'number' => '2', 'label' => 'Phase 2', 'title' => 'Gouvernance & Annuaires',
                    'status' => 'Terminée', 'is_first' => false, 'is_last' => false,
                    'items' => ['Votes collectifs et quorum', 'Annuaire de réseaux et de membres', 'Fédération d\'annuaires', 'Contrôle parental (réseaux famille)'],
                ],
                [
                    'number' => '★', 'label' => 'Phase 3', 'title' => 'Services & Intégrations',
                    'status' => 'En cours', 'is_first' => true, 'is_last' => false,
                    'items' => ['Système de plugins (WASM sandbox)', 'Plugins : Agenda, Documents, Marketplace', 'Connecteurs SaaS (Google Calendar, Stripe…)', 'Accès IA via MCP', 'Registre de Services Civium (RSC)'],
                ],
                [
                    'number' => '4', 'label' => 'Phase 4', 'title' => 'Applications & Écosystème',
                    'status' => 'À venir', 'is_first' => false, 'is_last' => false,
                    'items' => ['Application mobile iOS / Android', 'Application web (PWA)', 'Interopérabilité ActivityPub (Mastodon, PeerTube…)', 'Cercle 3 (pair E2E) + récupération sociale'],
                ],
                [
                    'number' => '5', 'label' => 'Phase 5', 'title' => 'Maturité',
                    'status' => 'À venir', 'is_first' => false, 'is_last' => true,
                    'items' => ['Certification des plugins', 'Audit de sécurité externe', 'SDK pour intégrateurs tiers', 'Gouvernance du projet Civium'],
                ],
            ],
        ]);
    }

    public function inscription(): void
    {
        $email = filter_var($this->f3->get('POST.email'), FILTER_VALIDATE_EMAIL);
        if ($email) {
            $db = $this->f3->get('DB');
            if (!$db) {
                header('Content-Type: application/json');
                echo json_encode(['status' => 'error']);
                return;
            }
            try {
                $db->exec('INSERT INTO waitlist (email) VALUES (?)', [$email]);
                $status = 'ok';
            } catch (\Exception $e) {
                $status = strpos($e->getMessage(), 'Duplicate') !== false ? 'duplicate' : 'error';
            }
        } else {
            $status = 'invalid';
        }
        header('Content-Type: application/json');
        echo json_encode(['status' => $status]);
    }
}
