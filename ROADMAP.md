# ROADMAP.md — Civium

Suivi du développement, phase par phase. Chaque tâche cochée = code mergé sur `master`.

**Spec de référence :** [README.md](README.md) | **Stack :** [STACK.md](STACK.md)

---

## Statut global

| Phase | Nom | Statut |
|---|---|---|
| — | Site web de présentation | 🔲 Non démarré |
| — | Infrastructure centrale (RCC + client web) | 🔲 Non démarré |
| 0 | MVP | ✅ Terminé |
| 1 | Transport P2P réel | ✅ Terminé |
| 2 | Gouvernance & Annuaires | ✅ Terminé |
| 3 | Services & Intégrations | 🚧 En cours |
| 4 | Applications & Écosystème | ⏳ En attente Phase 3 |
| 5 | Maturité | ⏳ En attente Phase 4 |

---

## Priorité — Site web de présentation

> Promouvoir le projet et attirer contributeurs, partenaires et premiers utilisateurs avant même la sortie du MVP.

- [ ] Définir le domaine et l'hébergement (civium.net ou civium.fr — Scaleway)
- [ ] Page d'accueil : accroche, proposition de valeur, visuel du protocole
- [ ] Section "Pourquoi Civium" (problème → solution → contrôle)
- [ ] Section "Comment ça marche" (cercles, réseaux, plugins — vulgarisé)
- [ ] Section "Cas d'usage" (famille, asso, quartier, entreprise)
- [ ] Feuille de route publique (basée sur ce ROADMAP)
- [ ] Formulaire d'inscription liste d'attente / newsletter
- [ ] Lien vers le dépôt GitHub
- [ ] Page "Contribuer" (comment participer au projet)
- [ ] SEO de base + Open Graph (partage réseaux sociaux)
- [ ] Stack : PHP Fat-Free + Alpine.js (cohérent avec l'app web Civium — infra Scaleway existante)

---

## Infrastructure centrale — Registre Central Civium (RCC) + Client web

> Indépendante des phases numérotées. Peut démarrer dès que Phase 3 S7-8 (Documents) est terminée, en parallèle de Phase 3 S9+.

### Bloc 1 — Registre Central Civium (RCC) — côté serveur PHP

> `website/` — PHP F3 + MySQL sur `https://www.rouaix.com/civium`

- [ ] Système de migrations BDD automatiques (`src/models/Migration.php`) — appliqué au bootstrap, jamais de plantage serveur sur changement de schéma
- [ ] Migration `001_initial.sql` : tables `schema_migrations`, `networks`, `alerts`, `magic_links`
- [ ] Table `networks` : network_cid (PK), network_name, admin_cid, admin_pubkey, admin_email, ip_address, registered_at, signature
- [ ] Table `alerts` : id, type, description, network_cids_json, emitted_at, emitted_by
- [ ] Table `magic_links` : token (PK), email, cid, expires_at, used
- [ ] `POST /api/register` — reçoit l'enregistrement d'un réseau, vérifie la signature Ed25519, stocke
- [ ] `POST /api/alert` — (clé admin RCC requise) émet une alerte fraude
- [ ] `GET /api/networks` — liste publique paginée des réseaux enregistrés (CID + nom uniquement)
- [ ] Envoi email alerte via SMTP (PHP Mailer ou F3 mailer)

### Bloc 2 — Enregistrement automatique côté desktop (civium-core + Tauri)

- [ ] `civium-core` : constante `RCC_URL = "https://www.rouaix.com/civium"` (codée en dur)
- [ ] `civium-core` : fonction `register_network(network, admin_email) -> Result<()>`
  - Construit le payload JSON signé
  - HTTP POST vers `RCC_URL/api/register`
  - Retourne Ok ou une erreur avec le statut HTTP
- [ ] `civium-core` : statut d'enregistrement dans `NetworkData` : `Pending | Registered | Failed`
- [ ] Store (Tauri + CLI) : persiste le statut d'enregistrement
- [ ] Tauri `network_create` : déclenche l'enregistrement après création, stocke le statut
- [ ] Tâche retry en arrière-plan (Tauri) : toutes les 5 s → 30 s → 5 min → 30 min → 1 h pour les réseaux en `Pending`
- [ ] Dashboard : badge "Enregistré ✓" / "En attente…" / "Échec ⚠" sur chaque réseau
- [ ] Saisie email admin obligatoire dans l'écran de création de réseau (Onboarding + Dashboard)

### Bloc 3 — Transport WebSocket pour nœuds desktop

> Nécessaire pour que les clients web (WASM) puissent se connecter via libp2p

- [ ] Ajouter `libp2p-websocket` comme transport dans `civium-core/node.rs`
- [ ] `NodeConfig` : `listen_ws: String` (défaut `/ip4/0.0.0.0/tcp/0/ws`)
- [ ] Les adresses d'écoute WebSocket sont annoncées dans la DHT et affichées dans le Dashboard
- [ ] CLI `node start --listen-ws <addr>`

### Bloc 4 — civium-core compilé en WebAssembly

- [ ] Ajouter target `wasm32-unknown-unknown` au workspace Cargo
- [ ] Feature flag `wasm` dans `civium-core` : active les transports WebSocket/WebRTC, désactive TCP/QUIC
- [ ] `wasm-pack build` génère `website/src/www/wasm/civium_core.js` + `.wasm`
- [ ] Bindings JS (`civium_core.js`) exposent : init_identity, load_identity, list_networks, join_network, send_message, list_messages, create_proposal, vote_cast, list_agenda_events…
- [ ] Tests WASM basiques (wasm-bindgen-test)

### Bloc 5 — Client web PHP + WASM

- [ ] `website/src/controllers/AuthController.php` : magic link (generate_token, validate_token, session)
- [ ] `website/src/controllers/AppController.php` : sert `civium/index.html` (shell Alpine.js)
- [ ] `website/src/www/civium/index.html` : charge `civium_core.wasm`, init Alpine.js
- [ ] Alpine.js : même UX que le Dashboard Tauri (messagerie, gouvernance, agenda, annuaire, notifications)
- [ ] Flux auth :
  - Saisie email → envoi magic link → clic → session
  - 1re connexion : saisie `secret_b58` → chiffrement PIN → stockage IndexedDB
  - Connexions suivantes : magic link → PIN → déchiffrement clé depuis IndexedDB
- [ ] Affichage statut P2P (connecté / hors ligne) dans l'interface web
- [ ] Page d'admin RCC (protégée par token admin) : liste des réseaux, envoi alerte fraude

### Bloc 6 — Alertes fraude reçues côté desktop

- [ ] `civium-core` : vérifie et affiche les alertes RCC reçues via P2P (signées par la clé publique RCC)
- [ ] Clé publique RCC codée en dur dans `civium-core` (updated lors des releases)
- [ ] Dashboard Tauri : bandeau d'alerte rouge si alerte active sur un réseau membre

### Critères de succès Infrastructure centrale

- [ ] Créer un réseau dans l'app desktop → il apparaît dans `GET /api/networks` dans les 5 s
- [ ] Couper Internet lors de la création → le réseau se ré-enregistre automatiquement à la reconnexion
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
- [ ] Adressage hybride : direct (IP/URL) + P2P (CID via DHT) *(reporté Phase 1)*

### Semaines 11–12 — Stabilisation

- [x] Rejoindre un réseau via lien d'invitation (Tauri — onboarding)
- [x] Demandes d'admission en attente — admit/reject admin
- [ ] Pairing multi-appareils (QR code + sous-clés dérivées) *(reporté Phase 3)*
- [ ] Mode hors-ligne avancé : cache local + resync CRDT complet *(reporté Phase 3)*
- [ ] Documentation utilisateur v0.1 *(reporté fin Phase 2)*
- [ ] Publication protocole v0.1 *(reporté fin Phase 2)*

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

### Semaines 7–8 — Plugin Documents

- [ ] `Document` : id, title, body (chiffré), version, network_cid, created_by, updated_at
- [ ] Table `documents` dans les deux stores
- [ ] CLI : `doc create/list/show/update/delete`
- [ ] Tauri : commandes + Dashboard section Documents

### Semaines 9–10 — Serveur MCP (accès IA)

- [ ] Serveur MCP intégré exposant les données Civium en lecture
- [ ] Resources : networks, members, messages, proposals, directory entries
- [ ] CIL appliqué sur chaque requête MCP
- [ ] Configuration dans `NodeConfig` (port MCP, token d'accès)
- [ ] Tauri : `mcp_start`/`mcp_stop` + affichage token dans Dashboard

### Semaines 11–12 — Pairing multi-appareils

- [ ] Dérivation de sous-clés depuis le secret primaire (HKDF)
- [ ] QR code de pairing (deep link `civium://pair/<token>`)
- [ ] Protocole de transfert d'identité chiffré entre deux nœuds P2P
- [ ] Révocation d'un appareil secondaire

### Semaines 13–14 — Mode hors-ligne avancé

- [ ] CRDT G-Set pour members et messages (merge complet sans conflits)
- [ ] Queue de messages en attente de sync (outbox P2P)
- [ ] Indicateur de "messages non synchronisés" dans la sidebar
- [ ] Résolution automatique des conflits sur reconnexion

### Critères de succès Phase 3

- [ ] Un plugin tiers peut lire les membres d'un réseau via le CIL sans accès direct au store
- [ ] L'Agenda et les Documents sont utilisables en autonomie dans le Dashboard
- [ ] Un assistant IA peut interroger un réseau via MCP (lecture seule, CIL appliqué)
- [ ] L'app fonctionne hors-ligne et se resynchronise automatiquement à la reconnexion

---

## Phase 4 — Applications & Écosystème

- [ ] Application mobile iOS / Android (React Native ou Flutter + Rust FFI)
- [ ] Client web PWA installable (Progressive Web App — basé sur le client web WASM, voir Infrastructure centrale)
- [ ] Pairing QR code desktop ↔ web (approuver une session web depuis l'app desktop — comme WhatsApp Web)
- [ ] Cercle 3 (pair E2E) + récupération sociale
- [ ] Interopérabilité ActivityPub (Mastodon, PeerTube…)
- [ ] Notarisation (OpenTimestamps / Bitcoin) — axe monétisation
- [ ] Badge légal (vérification Sirene / JO) — axe monétisation

---

## Phase 5 — Maturité

- [ ] Programme de certification des plugins (niveaux Minimal / RSC / Certifié)
- [ ] Audit de sécurité externe
- [ ] SDK Civium (intégrateurs tiers)
- [ ] Documentation développeur complète
- [ ] Documentation utilisateur v0.1 *(reporté de Phase 0)*
- [ ] Publication protocole v0.1 *(reporté de Phase 0)*
- [ ] White-label (licence par taille d'organisation)
- [ ] Gouvernance du projet Civium lui-même (association ou fondation)

---

## Décisions techniques en suspens

| Décision | Options | Échéance |
|---|---|---|
| Framework mobile | React Native vs Flutter | Avant Phase 4 |
| Hébergement nœuds bootstrap | À définir (`bootstrap.civium.net`) | Avant fin Phase 1 ✅ → à planifier Phase 3 |

---

*Dernière mise à jour : 2026-05-15 (Phase 2 semaines 9-10 — fédération d'annuaires)*
