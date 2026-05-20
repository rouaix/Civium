# ROADMAP.md — Civium

Suivi du développement, phase par phase. Chaque tâche cochée = code mergé sur `master`.

**Spec de référence :** [README.md](README.md) | **Stack :** [STACK.md](STACK.md)

---

## Statut global

| Phase | Nom | Statut |
|---|---|---|
| — | Site web de présentation | 🚧 En cours (code ✅ — domaine/hébergement à décider) |
| — | Infrastructure centrale (RCC + client web) | 🚧 En cours (Blocs 1-6 ✅ — déploiement PHP en attente) |
| 0 | MVP | ✅ Terminé |
| 1 | Transport P2P réel | ✅ Terminé |
| 2 | Gouvernance & Annuaires | ✅ Terminé |
| 3 | Services & Intégrations | ✅ Terminé |
| 4 | Applications & Écosystème | ✅ Terminé (1 critère nécessite déploiement PHP) |
| 5 | Maturité | 🚧 En cours (certification ✅, SDK ✅, docs ✅, white-label ✅, hub agenda ✅, hub gouvernance ✅ — audit externe + gouvernance associative à planifier) |

---

## Priorité — Site web de présentation

> Promouvoir le projet et attirer contributeurs, partenaires et premiers utilisateurs avant même la sortie du MVP.

- [ ] Définir le domaine et l'hébergement (civium.net ou civium.fr — Scaleway) *(non-code — décision à prendre)*
- [x] Page d'accueil : hero, proposition de valeur, problème → solution, fonctionnalités, CTA
- [x] Section "Pourquoi Civium" (problème → solution → contrôle) — intégrée dans `home.html`
- [x] Section "Comment ça marche" (cercles, réseaux, plugins — vulgarisé) — `how.html`
- [x] Section "Cas d'usage" (famille, asso, quartier, entreprise) — `usecases.html`
- [x] Feuille de route publique — `roadmap.html`
- [x] Formulaire d'inscription liste d'attente — `home.html` (Alpine.js + `POST /inscription`)
- [x] Lien vers le dépôt GitHub — dans nav et footer du `layout.html`
- [x] Page "Contribuer" (comment participer) — `contribute.html`
- [x] SEO de base + Open Graph — `layout.html` (`<meta name="description">`, `og:title/description/image/type`)
- [x] Stack PHP F3 + Alpine.js — en place

---

## Infrastructure centrale — Registre Central Civium (RCC) + Client web

> Indépendante des phases numérotées. Peut démarrer dès que Phase 3 S7-8 (Documents) est terminée, en parallèle de Phase 3 S9+.

### Bloc 1 — Registre Central Civium (RCC) — côté serveur PHP ✅

> `website/` — PHP F3 + MySQL sur `https://www.rouaix.com/civium`

- [x] Système de migrations BDD automatiques (`src/models/Migration.php`) — appliqué au bootstrap dans `index.php`, jamais de plantage serveur sur changement de schéma
- [x] Migration `001_initial.sql` : `waitlist` (intégré depuis `civium.sql`) + `networks`, `alerts`, `magic_links` — `civium.sql` réduit au `CREATE DATABASE` uniquement
- [x] Table `networks` : network_cid (PK), network_name, admin_cid, admin_pubkey, admin_email, ip_address, registered_at, signature
- [x] Table `alerts` : id, type, description, network_cids (JSON), emitted_at, emitted_by
- [x] Table `magic_links` : token (PK), email, cid, expires_at, used
- [x] `POST /api/register` — reçoit l'enregistrement d'un réseau, vérifie la signature Ed25519, stocke
- [x] `POST /admin/alerte` — (ADMIN_TOKEN requis) enregistre une alerte fraude
- [x] `GET /api/networks` — liste publique paginée des réseaux enregistrés (`?page=` + `?per_page=`, max 200)
- [x] Envoi email alerte via SMTP — `src/models/Mailer.php` (F3 SMTP + fallback `mail()`), config `SMTP_*` dans `config.ini`

### Bloc 2 — Enregistrement automatique côté desktop (civium-core + Tauri) ✅ Phase 4 S3-4

- [x] `civium-core` : constante `RCC_URL = "https://www.rouaix.com/civium"` (codée en dur)
- [x] `civium-core` : fonction `register_network(network, admin_email) -> Result<()>`
  - Construit le payload JSON signé
  - HTTP POST vers `RCC_URL/api/register`
  - Retourne Ok ou une erreur avec le statut HTTP
- [x] `civium-core` : statut d'enregistrement (`Pending | Registered | Failed`) + table `rcc_registrations`
- [x] Store (Tauri) : persiste le statut d'enregistrement
- [x] Tauri `rcc_register` / `rcc_status` / `rcc_status_list` : déclenche l'enregistrement, expose le statut
- [x] Tâche retry en arrière-plan (Tauri) : toutes les 5 s → 30 s → 5 min → 30 min → 1 h pour les réseaux en `Pending`
- [x] Dashboard : badge "Enregistré ✓" / "En attente…" / "Échec ⚠" par réseau + section RCC
- [x] Saisie email admin obligatoire dans l'écran de création de réseau (Dashboard)

### Bloc 3 — Transport WebSocket pour nœuds desktop ✅

> Nécessaire pour que les clients web (WASM) puissent se connecter via libp2p

- [x] Ajouter `libp2p-websocket` (+ feature `dns`) comme transport dans `civium-core/node.rs`
- [x] `NodeConfig` : `listen_ws: Option<String>` (défaut activé `/ip4/0.0.0.0/tcp/0/ws`)
- [x] Les adresses d'écoute WebSocket sont affichées dans le Dashboard (label WS: en vert, filtrage `/ws`)
- [x] CLI `node start --listen-ws <addr>` (défaut activé ; `--listen-ws ""` pour désactiver)

### Bloc 4 — civium-core compilé en WebAssembly ✅

- [x] `desktop/rust-toolchain.toml` : target `wasm32-unknown-unknown` déclarée (+ cibles Android/iOS)
- [x] Feature flag `wasm` dans `civium-core` : active wasm-bindgen, désactive le module `node` (TCP/QUIC/mDNS)
- [x] `desktop/build-wasm.sh` + `wasm-pack build --target web --features wasm` → `website/src/www/wasm/`
- [x] Bindings JS : `civium_version`, `generate_identity`, `load_identity`, `group_key_*`, `network_create`, `message_build`, `message_decrypt`, `proposal_create`, `vote_cast`, `vote_compute`, `agenda_event_build`, `document_build`, `document_decrypt_body`, `pairing_complete`
- [x] 6 tests `wasm-bindgen-test` : identity roundtrip, group key, network create, message roundtrip, vote cast, document roundtrip
- [x] CI `.github/workflows/wasm.yml` : build + test headless Chrome

### Bloc 5 — Client web PHP + WASM ✅

- [x] `website/src/controllers/AuthController.php` : magic link (generate_token, validate_token, session)
- [x] `website/src/controllers/AppController.php` : sert `civium/index.html` (shell Alpine.js)
- [x] `website/src/www/civium/index.html` : charge `civium_core.wasm`, init Alpine.js
- [x] Alpine.js : SPA avec onglets (Identité, Réseaux, Messages, Gouvernance, Agenda, Documents)
- [x] Flux auth :
  - Saisie email → envoi magic link → clic → session
  - 1re connexion : saisie `secret_b58` → chiffrement PIN (PBKDF2 + AES-GCM) → stockage IndexedDB
  - Connexions suivantes : magic link → PIN → déchiffrement clé depuis IndexedDB
- [x] Affichage statut P2P (connecté / hors ligne) — connexion WebSocket native vers nœud desktop
- [x] Page d'admin RCC (protégée par token admin) : liste des réseaux, envoi alerte fraude

### Bloc 6 — Alertes fraude reçues côté desktop ✅

- [x] `civium-core` : vérifie les alertes RCC reçues via P2P (`verify_rcc_alert` + `CiviumRequest::BroadcastAlert` + `NodeEvent::FraudAlertReceived`)
- [x] Clé publique RCC codée en dur dans `civium-core` (`RCC_PUBLIC_KEY_B58` — à remplacer avant release production)
- [x] Dashboard Tauri : bandeau d'alerte rouge si alerte active reçue sur le nœud

### Critères de succès Infrastructure centrale

- [x] Créer un réseau dans l'app desktop → enregistrement RCC automatique avec retry exponentiel *(côté desktop — serveur PHP en cours)*
- [ ] Créer un réseau dans l'app desktop → il apparaît dans `GET /api/networks` dans les 5 s *(nécessite PHP déployé)*
- [ ] Couper Internet lors de la création → le réseau se ré-enregistre automatiquement à la reconnexion *(retry desktop OK — PHP à déployer)*
- [ ] Un utilisateur peut se connecter au client web avec son email, accéder à ses réseaux, envoyer un message visible sur l'app desktop
- [ ] Une alerte fraude émise depuis l'admin RCC s'affiche dans l'app desktop ET est reçue par email

---

## Phase 0 — MVP `~12 semaines` ✅

> Valider que deux réseaux souverains peuvent se connecter sans serveur central (sur machine locale, base SQLite partagée).

### Semaines 1–2 — Protocole de base (`civium-core`)

- [x] Génération de paires de clés Ed25519
- [x] Dérivation du CID depuis la clé publique (blake3 + base58)
- [x] Création et démarrage d'un nœud local
- [x] Transport libp2p TCP + QUIC
- [x] Découverte de pairs via DHT Kademlia + mDNS
- [x] Chiffrement des connexions via Noise Protocol

### Semaines 3–4 — Identité et membres

- [x] Création de compte (CID membre + clé Ed25519)
- [x] Format identifiant réseau (`<cid_membre_court>@<cid_réseau_court>`)
- [x] Nom affiché par réseau (choix libre, unique dans le réseau)
- [x] Invitation d'un membre (lien ou CID)
- [x] Validation d'admission par l'admin
- [x] Cercles de confiance 0, 1 et 2

### Semaines 5–6 — Messagerie

- [x] Chiffrement E2E applicatif (clé de groupe ChaCha20-Poly1305 — cercles 0-2)
- [x] Messages directs entre membres d'un même réseau
- [x] Fils de discussion dans le réseau
- [x] Synchronisation CRDT (G-Set mailbox) entre membres connectés
- [x] Queue locale + resync à la reconnexion (offline-first)

### Semaines 7–8 — Connexion inter-réseaux

- [x] Handshake : `CONNECT_REQUEST` / `CONNECT_RESPONSE`
- [x] États de connexion (Demandée → En validation → Active / Refusée / Bloquée)
- [x] Validation par l'admin (acceptation, refus simple, refus motivé, blocage)
- [x] Accord de Partage Civium (APC) signé cryptographiquement
- [x] Partage d'annuaire membres entre réseaux connectés
- [x] Révocation unilatérale d'une connexion

### Semaines 9–10 — Interface Desktop + CLI

- [x] Application Tauri (interface React + Tailwind — base)
- [x] Onboarding : création de compte + réseau en < 5 min
- [x] CLI : commandes essentielles (`node start`, `network create/connect`, `member invite`)
- [x] Stockage local SQLite (schéma commun CLI + Tauri ; upgrade SQLCipher via feature flag)
- [x] Adressage hybride : direct (IP/URL) + P2P (CID via DHT) *(complété Phase 1 — `node join-p2p --via <multiaddr>` + DHT announce/discover)*

### Semaines 11–12 — Stabilisation

- [x] Rejoindre un réseau via lien d'invitation (Tauri — onboarding)
- [x] Demandes d'admission en attente — admit/reject admin
- [x] Pairing multi-appareils (QR code + sous-clés dérivées) *(complété Phase 3 S11-12)*
- [x] Mode hors-ligne avancé : cache local + resync CRDT complet *(complété Phase 3 S13-14)*
- [x] Documentation utilisateur v0.1 *(complété Phase 5)*
- [x] Publication protocole v0.1 *(complété Phase 5)*

### Critères de succès MVP

- [x] Deux réseaux indépendants peuvent se connecter sans serveur central *(validé Phase 1)*
- [x] Un réseau peut refuser ou bloquer une connexion
- [x] Les messages sont chiffrés et illisibles hors des nœuds destinataires
- [x] Le nœud fonctionne hors-ligne et se resynchronise à la reconnexion
- [x] Un utilisateur non technique peut créer un réseau et inviter un membre en moins de 5 minutes
- [x] Aucune donnée ne transite par un serveur Civium central

---

## Phase 1 — Transport P2P réel `~10 semaines` ✅

> Faire communiquer deux nœuds sur des machines distinctes sans base de données partagée.

### Semaines 1–2 — Protocole applicatif

- [x] Protocole `/civium/1.0.0` (CBOR request-response sur libp2p)
- [x] `CiviumRequest` : Join, Sync, Ping / `CiviumResponse` : JoinAccepted, SyncData, Pong
- [x] `CiviumNode` refactorisé avec API canaux (`NodeCommand` / `NodeEvent`)
- [x] DHT announce (`/civium/net/<cid_short>`) + discover (`get_record`)
- [x] CLI : `node start --announce`, `node join-p2p --via <multiaddr>`

### Semaines 3–4 — Nœud Tauri background

- [x] `civium-tauri/src-tauri/src/node.rs` : démarrage P2P en background au lancement Tauri
- [x] Event loop : Listening → auto-announce, PeersDiscovered → dial + sync, PeerConnected → SyncRequest
- [x] Ticker périodique 60 s (DiscoverPeers)
- [x] `AppState { node_tx, listen_addrs }` géré dans Tauri
- [x] Commandes Tauri `node_status` et `node_sync`
- [x] `store::merge_sync_data`, `load_messages`, `find_network_by_full_cid`

### Semaines 5–6 — Indicateurs P2P dans l'UI

- [x] `civium://sync-completed` émis après chaque merge réussi (payload = cid_short)
- [x] `types.ts` : `NodeStatus { running, listen_addrs }`
- [x] Dashboard : indicateur En ligne/Hors ligne + 1re adresse d'écoute dans la sidebar
- [x] Bouton "Synchroniser" par réseau (appelle `node_sync`)
- [x] Listener `civium://sync-completed` → rafraîchit membres + compteur réseau
- [x] Polling `node_status` toutes les 5 s (useRef pour éviter closure stale)

### Semaines 7–8 — Messagerie chiffrée dans l'UI

- [x] `store::save_message` — INSERT OR IGNORE d'un seul message
- [x] `commands::message_list` — déchiffre via `GroupKey::decrypt`, résout `author_name`
- [x] `commands::message_send` — chiffre via `GroupKey::encrypt`, INSERT, retourne `MessageDisplay`
- [x] Dashboard : section "Fil de discussion" — liste scrollable, auto-scroll, formulaire Entrée/Maj+Entrée
- [x] Mise à jour optimiste + rafraîchissement sur `sync-completed`
- [x] `types.ts` : `MessageDisplay`

### Semaines 9–10 — Join P2P réel dans l'UI

- [x] `commands::network_join_p2p` : nœud P2P temporaire, dial pair, `CiviumRequest::Join`, timeout 30 s
- [x] Sauvegarde réseau sur `JoinAccepted`
- [x] `Onboarding.tsx` : champ "Adresse du pair" (optionnel), routage P2P si renseigné
- [x] Clé secrète affichée pour tous les modes d'onboarding

---

## Phase 2 — Gouvernance & Annuaires `~14+ semaines` 🚧

> Doter chaque réseau d'une gouvernance collective et d'une capacité de découverte inter-réseaux.

### Semaines 1–2 — Votes collectifs ✅

- [x] `civium-core/src/governance/mod.rs` : `Proposal`, `Vote`, `ProposalStatus`, `VoteResult`
- [x] `compute_result()` avec quorum configurable
- [x] Tables SQLite `proposals` + `votes` dans les deux stores (CLI + Tauri)
- [x] CLI : `civium governance propose/list/vote/results`
- [x] Tauri : `proposal_list`, `proposal_create`, `vote_cast`, `vote_results`
- [x] Dashboard : section Propositions — formulaire, boutons de vote, barre de progression

### Semaines 3–4 — Garde-fou majoritaire ✅

- [x] `AdminAction`, `AdminActionKind`, `AdminActionStatus`, `add_contest()` dans civium-core
- [x] Tables `admin_actions` dans les deux stores
- [x] `member_admit` enregistre automatiquement une `AdminAction`
- [x] CLI : `governance actions`, `governance contest` (auto-crée Proposal si seuil majoritaire)
- [x] Tauri : `admin_action_list`, `admin_action_contest`
- [x] Dashboard : section Garde-fou avec countdown, bouton Contester, badge SUSPENDU

### Semaines 5–6 — Délégation de vote ✅

- [x] `VoteDelegation` + `compute_result_with_delegations()` dans civium-core
- [x] Vote direct prioritaire sur le délégué ; 1 seul niveau de délégation ; réseau entier ou par proposition
- [x] Tables `vote_delegations` dans les deux stores
- [x] CLI : `governance delegate/revoke-delegation/delegations`
- [x] Tauri : `vote_delegate`, `vote_revoke_delegation`, `vote_list_delegations`
- [x] Dashboard : délégation globale + par proposition, badge délégué actif

### Semaines 7–8 — Annuaire de réseaux et de membres ✅

- [x] `civium-core/src/directory/mod.rs` : `DirectoryEntry`, `EntryKind` (network/member), `matches()`
- [x] `NetworkKind` (Standard/Directory) ajouté à `NetworkData`
- [x] Table `directory_entries` dans les deux stores ; fonctions save/list/search/delete
- [x] CLI : `civium directory create/list/publish/search/remove`
- [x] Tauri : `directory_create`, `directory_list_networks`, `directory_publish`, `directory_list`, `directory_search`, `directory_remove`
- [x] `NetworkInfo` enrichi de `is_directory: bool`
- [x] Dashboard : section Annuaire — formulaire de publication, recherche, liste avec badge kind

### Semaines 9–10 — Fédération d'annuaires ✅

- [x] `FederatedDirectory` dans civium-core (host/peer CID, peer_name, peer_addr)
- [x] Table `directory_federations` dans les deux stores
- [x] CLI : `directory federate/unfederate/federations` + `--federated` sur search
- [x] Tauri : `directory_federate`, `directory_unfederate`, `directory_federations`, `directory_search` enrichi (`include_federated`, `source_dir_name`)
- [x] Dashboard : section Fédérations (add/remove), checkbox "inclure fédérés", badge "via <nom>" sur résultats fédérés

### Semaines 11–12 — RRM (Registre des Réseaux Malveillants) ✅

- [x] `NetworkKind::Rrm` dans civium-core
- [x] `RrmEntry` (network_cid, reason, evidence_url, reported_by, reported_at) + `TrustedRrm`
- [x] Tables `rrm_entries` + `trusted_rrms` dans les deux stores
- [x] Logique de confiance configurable : chaque réseau choisit quels RRM il consulte
- [x] CLI : `rrm create/list/report/search/remove` + `network trust-rrm/untrust-rrm/trusted-rrms/check-rrm`
- [x] Tauri : 8 commandes IPC + Dashboard section RRM + section RRM de confiance
- [x] `check_rrm_warnings` — avertissement si réseau listé dans un RRM de confiance

### Semaines 13–14 — Profils enfants et contrôle parental

- [x] Flag `is_minor` sur un compte membre
- [x] Restrictions configurables : cercles accessibles, contacts autorisés (`MinorRestrictions`)
- [x] Compte tuteur : lien parent-enfant (`GuardianLink`)
- [x] CLI : `member set-minor/unset-minor/set-guardian/remove-guardian/guardians/wards/set-restrictions`
- [x] Tauri : badge mineur + panneau admin expandable (toggle, gestion tuteurs)
- [x] Blocage automatique des interactions cercle 2-3 avec non-tuteurs pour les comptes mineurs (`check_minor_interaction` + enforcement CLI `msg send --to` + commande Tauri `message_send_direct`)

### Critères de succès Phase 2

- [x] Un réseau peut créer et voter une proposition avec quorum et délégation
- [x] Une action admin contestée par la majorité est automatiquement suspendue
- [x] Un annuaire dédié peut référencer des réseaux et être fédéré avec d'autres annuaires
- [x] Un réseau peut consulter un RRM et refuser automatiquement les connexions listées
- [x] Un compte mineur est isolé des contenus et contacts non autorisés par son tuteur

---

## Phase 3 — Services & Intégrations `~14 semaines` 🚧

> Ajouter la couche plugin (manifeste + CIL + sandbox), les plugins préinstallés, l'accès IA via MCP, et la robustesse hors-ligne.

### Semaines 1–2 — Fondations plugin (manifeste + CIL + registre) ✅

- [x] `PluginManifest` : id, name, version, permissions, is_system
- [x] `PluginRecord` : état (Enabled/Disabled), installed_at
- [x] `PluginPermission` enum : ReadMembers, ReadMessages, WriteMessages, ReadGovernance…
- [x] `CIL` : `check_cil(plugin, action)` — applique les permissions déclarées
- [x] Table `plugins` dans les deux stores ; seed des plugins système au démarrage
- [x] Plugins préinstallés : Gouvernance, CIL, Messagerie, Annuaire (enabled par défaut)
- [x] CLI : `plugin list/info/enable/disable/install`
- [x] Tauri : `plugin_list`, `plugin_enable`, `plugin_disable`
- [x] Dashboard : panneau Plugins dans la sidebar (liste, badges permissions, toggle)

### Semaines 3–4 — Plugin Agenda ✅

- [x] Modèle de données : `AgendaEvent` (id, title, description, start_at, end_at, recurrence, network_cid, created_by)
- [x] Table `agenda_events` dans les deux stores
- [x] CLI : `agenda create/list/delete` + `run_agenda()`
- [x] Tauri : `agenda_create/list/update/delete` + Dashboard section Agenda

### Semaines 5–6 — Fil d'activité + Notifications ✅

- [x] `ActivityEvent` : kind (MemberJoined, MessagePosted, ProposalCreated, VoteCast, AgendaEventCreated…), actor, summary, occurred_at
- [x] Table `activity_feed` dans les deux stores ; auto-émission sur chaque action clé
- [x] `Notification` : source_event_id, target_cid, read, created_at
- [x] CLI : `activity list [--unread]`
- [x] Tauri : `activity_list` + `notification_list` + `notification_mark_read` + badge non-lu dans la sidebar

### Semaines 7–8 — Plugin Documents ✅

- [x] `Document` : id, title, body (chiffré), version, network_cid, created_by, updated_at
- [x] Table `documents` dans les deux stores
- [x] CLI : `doc create/list/show/update/delete`
- [x] Tauri : `document_create/list/update/delete` + Dashboard section Documents (liste, formulaire create/delete)

### Semaines 9–10 — Serveur MCP (accès IA) ✅

- [x] Serveur MCP intégré exposant les données Civium en lecture (JSON-RPC 2.0 sur HTTP — `mcp.rs`)
- [x] Resources : networks, members, messages, proposals, agenda, documents, directory entries
- [x] CIL appliqué sur chaque requête MCP (`check_cil` avec `mcp_plugin()` read-only)
- [x] Configuration : port + token Bearer auto-généré ; `mcp_port: Option<u16>` dans `NodeConfig`
- [x] Tauri : `mcp_start`/`mcp_stop`/`mcp_status` + affichage token dans Dashboard (toggle, copier l'URL)

### Semaines 11–12 — Pairing multi-appareils ✅

- [x] Chiffrement ChaCha20-Poly1305 + BLAKE3 derive_key pour sécuriser le lien de pairing
- [x] Deep link `civium://pair/<b58>` (payload : clé de pairing + nonce + secret chiffré, 10 min d'expiry)
- [x] `init_pairing()` / `complete_pairing()` dans civium-core
- [x] Tables `paired_devices` dans les deux stores ; `pair_init`, `pair_complete`, `pair_list`, `pair_revoke` Tauri
- [x] Dashboard : section Appareils couplés (générer un lien, coller un lien, liste avec révocation)

### Semaines 13–14 — Mode hors-ligne avancé ✅

- [x] CRDT G-Set pour members et messages (merge complet sans conflits — `store::merge_sync_data`)
- [x] Queue de messages en attente de sync (table `outbox_queue`, `enqueue_outbox` / `clear_outbox`)
- [x] Indicateur de messages non synchronisés dans la sidebar (badge ↑N en amber sur chaque réseau)
- [x] Résolution automatique sur reconnexion (`civium://outbox-cleared` déclenché après sync réussi)

### Critères de succès Phase 3

- [x] Un plugin tiers peut lire les membres d'un réseau via le CIL sans accès direct au store
- [x] L'Agenda et les Documents sont utilisables en autonomie dans le Dashboard
- [x] Un assistant IA peut interroger un réseau via MCP (lecture seule, CIL appliqué)
- [x] L'app fonctionne hors-ligne et se resynchronise automatiquement à la reconnexion

---

## Phase 4 — Applications & Écosystème `~14 semaines` 🚧

> Cercle 3 E2E, RCC, transport web, client WASM, interopérabilité, mobile.

### Semaines 1–2 — Cercle 3 Intime (chiffrement E2E de paire) ✅

- [x] `civium-core/src/e2e/mod.rs` : `PairKey` — dérivation X25519 DH depuis clés Ed25519 (SHA-512 + clamping RFC 7748 + BLAKE3 KDF)
- [x] `TrustCircle::Intime (3)` dans civium-core (enum mis à jour)
- [x] `MessageKind::E2E { to_cid_full }` dans civium-core/messaging
- [x] `pub_key_b58: Option<String>` sur `MemberRecord` / `PendingRecord` ; propagation dans `Network::create`, `submit_join_request`, `admit`
- [x] Tauri : `message_send_e2e` + déchiffrage E2E dans `message_list` (fallback gracieux)
- [x] Dashboard : toggle 🔒/🔓 par destinataire, icône 🔒 sur messages E2E, bordure violette

### Semaines 3–4 — Registre Central Civium (RCC) ✅

- [x] `civium-core/src/rcc/mod.rs` : `RccPayload`, `canonical_bytes()`, `RCC_URL` codée en dur
- [x] `CiviumKeypair::sign_bytes()` — signature Ed25519 du payload via libp2p
- [x] Table `rcc_registrations` SQLite dans Tauri ; commandes `rcc_register`, `rcc_status`, `rcc_status_list`
- [x] `rcc.rs` : retry exponentiel 5 s → 30 s → 5 min → 30 min → 1 h (max 10 tentatives)
- [x] Event `civium://rcc-status-changed` → rafraîchissement UI automatique
- [x] Dashboard : section RCC par réseau + badges ✓ / ↻ dans la sidebar
- [x] PHP (website) : `POST /api/register` (vérification signature Ed25519 + INSERT MySQL), `Migration.php`, bootstrap DB

### Semaines 5–6 — Transport WebSocket pour clients web ✅

- [x] `libp2p-websocket` + feature `dns` ajoutés à `civium-core` ; chaîne TCP + WebSocket dans `CiviumNode`
- [x] `NodeConfig` : `listen_ws: Option<String>` (défaut activé : `/ip4/0.0.0.0/tcp/0/ws`)
- [x] Dashboard : adresse WebSocket affichée en vert (label WS:), distincte de l'adresse TCP
- [x] CLI `node start --listen-ws <addr>` (défaut `/ip4/0.0.0.0/tcp/0/ws`, passer `""` pour désactiver)

### Semaines 7–8 — civium-core en WebAssembly ✅

- [x] `[lib] crate-type = ["cdylib", "rlib"]` ajouté à `civium-core/Cargo.toml`
- [x] Feature flag `wasm` dans `civium-core` : désactive le module `node` (TCP/QUIC/mDNS) pour wasm32
- [x] Refactoring `unix_now()` → `crate::time::unix_now()` (11 modules), `js_sys::Date::now()` sur wasm32
- [x] Refactoring des IDs custom (`thread::current`) → `Uuid::new_v4()` (4 modules)
- [x] `uuid` : feature `js` activée pour wasm32 ; `getrandom/js` comme backend browser
- [x] `wasm-pack build --target web -- --features wasm` → `website/src/www/wasm/civium_core{.js,_bg.wasm,.d.ts}`
- [x] Bindings JS : `generate_identity`, `load_identity`, `group_key_generate/encrypt/decrypt`, `network_create`, `message_build/decrypt`, `proposal_create`, `vote_compute`, `agenda_event_build`, `document_build/decrypt_body`
- [x] Script `desktop/build-wasm.sh` pour rebuilder avec la bonne destination
- [x] Tests WASM basiques inline (`wasm-bindgen-test`) : identity roundtrip, group key, network create, message encrypt/decrypt

### Semaines 9–10 — Client web PHP + WASM ✅

- [x] `GET /api/networks` — liste publique des réseaux enregistrés (CID + nom)
- [x] `AuthController.php` : magic link (POST /auth → token, GET /auth/verify → session, GET /auth/deconnexion)
- [x] `AppController.php` : vérifie la session, sert `app.html` (SPA Alpine.js + WASM)
- [x] `AdminController.php` : page admin RCC (protégée ADMIN_TOKEN), liste réseaux, formulaire alerte
- [x] `POST /admin/alerte` — enregistre une alerte fraude en BDD
- [x] `auth.html` — page de connexion Alpine.js (saisie email → magic link → message de confirmation)
- [x] `app.html` — SPA Alpine.js avec WASM : identité (setup/unlock PIN + IndexedDB + Web Crypto AES-GCM/PBKDF2), onglets Réseaux/Messages/Gouvernance/Agenda/Documents
- [x] Flux auth : email → magic link → session → saisie `secret_b58` + PIN → chiffrement AES-GCM → IndexedDB
- [x] Indicateur statut P2P (connecté / hors ligne) dans l'interface web
- [x] `admin.html` — tableau réseaux + alertes, formulaire émission alerte fraude

### Semaines 11–12 — Interopérabilité ActivityPub

- [x] Exposition des fils de discussion publics d'un réseau en tant qu'acteur ActivityPub
- [x] `GET /.well-known/webfinger` + `GET /users/<cid>` (profil Actor JSON-LD)
- [x] `POST /inbox` : réception de notes Mastodon/PeerTube (mapping → messages Civium)
- [x] `POST /outbox` : publication de messages Civium vers des abonnés ActivityPub
- [x] Configuration par réseau : activer/désactiver la fédération ActivityPub
- [x] Dashboard : section Fédération ActivityPub (adresse acteur, abonnés, statut)

### Semaines 13–14 — Application mobile (prototype)

- [x] Décision framework : React Native (cohérence avec le codebase React desktop)
- [x] Crate `civium-ffi` : bindings UniFFI exposant identité, réseaux, messages, jumelage
- [x] Prototype iOS + Android : onboarding (clé + réseau), liste des réseaux, messagerie
- [x] Pairing QR code desktop ↔ mobile (scan `civium://pair/<b58>` → `complete_pairing`)
- [x] Build CI : GitHub Actions — cargo-ndk (Android arm64) + cargo build iOS simulator

### Critères de succès Phase 4

- [x] Les messages en cercle 3 ne sont lisibles que par l'expéditeur et le destinataire
- [x] Créer un réseau → enregistrement automatique au RCC dans les 5 s (retry si hors ligne)
- [x] Un nœud desktop accepte les connexions WebSocket de clients web libp2p
- [x] Un utilisateur peut se connecter au client web (magic link + PIN), charger son identité depuis IndexedDB *(P2P non connecté — nœud desktop requis pour sync)*
- [ ] Un message Civium posté dans un réseau avec ActivityPub activé apparaît dans un fil Mastodon abonné
- [x] L'app mobile prototype permet l'onboarding et l'envoi d'un message sur iOS et Android

---

## Phase 5 — Maturité

- [x] Programme de certification des plugins (niveaux Minimal / RSC / Certifié) — `CertificationLevel` dans `civium-core/src/plugin/mod.rs`
- [ ] Audit de sécurité externe
- [x] Support Linux — workflow CI Tauri inclut `ubuntu-latest` (AppImage + .deb uploadés comme artifacts CI) ; libp2p TCP/QUIC/WebSocket/mDNS supporté sur Linux
- [x] SDK Civium (intégrateurs tiers) — crate `civium-sdk` avec `CiviumClient`, `ClientConfig`, types identity/network/messaging/governance
- [x] Documentation développeur complète — rustdoc complet sur `civium-sdk`, `civium-core::error`, modules network/messaging/governance ; exemples dans `lib.rs`
- [x] Documentation utilisateur v0.1 — site web PHP F3 (`home.html`, `how.html`, `usecases.html`) + `README.md` mis à jour avec statut v0.1 et critères MVP
- [x] Publication protocole v0.1 — `README.md` : bandeau v0.1, critères MVP cochés, feuille de route mise à jour Phases 0-4 ✅
- [x] White-label (licence par taille d'organisation) — tiers open/famille/association/entreprise, branding configurable, limites enforced sur `POST /api/register`, `GET /api/info` public, page admin `/admin/white-label`
- [x] Hub gouvernance web : `POST /hub/governance/proposal`, `GET /hub/governance/proposals`, `POST /hub/governance/vote` — app.html : onglet Gouvernance (propositions + votes Oui/Non/Abstention signés Ed25519, résultats en temps réel)
- [x] Hub agenda web : `POST /hub/agenda/event`, `GET /hub/agenda/events` — app.html : onglet Agenda (liste chronologique + créer événement)
- [x] Réseau Civium principal hébergé sur le hub (`ensureMainNetwork()`) — auto-rejoint à la connexion sur Desktop + web
- [x] Annuaire des réseaux publics (`GET /hub/network/public`) — modal "Rejoindre" avec onglets Annuaire/Invitation sur Desktop + web
- [x] Lien d'invitation email Desktop : option A (URL web `?join=CID`) + option B (deep link P2P desktop)
- [x] Auto-join depuis URL web (`/app?join=CID&jname=NOM`) — rejoindre un réseau en un clic depuis un email
- [ ] Gouvernance du projet Civium lui-même (association ou fondation)

---

## Décisions techniques en suspens

| Décision | Options | Échéance |
|---|---|---|
| Framework mobile | ~~React Native vs Flutter~~ → **React Native** (Phase 4 S13-14) | ✅ Décidé |
| Hébergement nœuds bootstrap | À définir (`bootstrap.civium.net`) | Avant fin Phase 1 ✅ → à planifier Phase 3 |

---

*Dernière mise à jour : 2026-05-20 (Phase 5 — support Linux CI ✅, autonat NAT traversal ✅, sandbox WASM CilExecutor ✅)*
