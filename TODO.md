# TODO.md — Civium

---

## 1 - Attention — Priorité Critique

- Ne jamais modifier le fichier civium/index.php c'est un symlink. Toutjours utiliser civium/src/www/index.php.
- J'ai corrigé le symlink en prod.
- Pour des questions de sécurité, on accède a cet espace via en local, http://civium/index.php (symlink vers src/www/index.php) et en prod via https://www.rouaix.com/civium/index.php (symlink vers src/www/index.php)- Il faut donc vérifier toutes les routes dans website pour qu'elles fonctionnent en local et en prod.
- ~~Le formulaire pour rejoindre la liste d'attente ne fonctionne plus~~ **Fait** : script waitlist déplacé en fichier JS externe (CSP), config.prod.ini chargé en production, feedback d'erreur ajouté.



## A Développer ou corriger dans desktop — Priorité haute

- Les messages peuvent contenir du texte, des fichiers, (audio, vidéo, images, pdf, etc.) et des événements (calendrier, tâches, etc.) — **Backend OK** (MessageKind::File, CalendarEvent), UI existante. ✓ types définis.
- ~~**Fichiers volumineux (images, vidéos)**~~ **Fait** : `message_send_file_path` ajouté (Rust + Tauri command) ; Dashboard.tsx routé vers ce path pour fichiers > 50 Mo via `tauri-plugin-fs` (à installer avec `npm install`).


## Webapp Civium — parité fonctionnelle avec le desktop — Priorité haute

> Le client web WASM actuel (`website/src/www/civium/app.html`) est une preuve de concept minimale il faut le développer et l'améliorer pour atteindre la parité fonctionnelle avec l'app desktop.
> L'objectif est une webapp complète offrant les mêmes fonctionnalités que l'app Tauri desktop.

- ~~**Identité & onboarding**~~ **Fait** : génération Ed25519 in-browser, import depuis `secret_b58`, PIN + AES-GCM + IndexedDB.
- ~~**Réseaux**~~ **Fait** : créer ✓, rejoindre ✓, lister ✓, quitter (DELETE /hub/member/leave + bouton sidebar) ✓ ; cercles de confiance affichés si disponibles dans les données membres.
- **Messagerie** : fil réseau ✓, ~~messages directs~~ **Fait** (hub DM + UI) ; E2E cercle 3, pièces jointes — à compléter. ~~Markdown~~ **Fait** : rendu marked.js (GFM, sauts de ligne) dans les bulles.
- ~~**Gouvernance**~~ **Fait** : créer proposition, voter Oui/Non/Abstention.
- ~~**Agenda**~~ **Fait** : créer/lister ✓ ; modifier/supprimer événements (PUT + DELETE /hub/agenda/event, boutons Modifier/Supprimer sur les cartes auteur) ✓.
- ~~**Documents**~~ **Fait** : créer/modifier/supprimer documents collaboratifs (hub + UI).
- ~~**Annuaire**~~ **Fait** : recherche membres par nom/CID, bouton "Message direct" (hub + UI).
- ~~**Notifications**~~ **Fait** : fil notifications hub (message, DM, document, nouveau membre), badge non-lus, marquer tout lu.
- ~~**Fil d'activité**~~ **Fait** : historique des derniers messages du réseau.
- **Paramètres** : profil ✓, session ✓ ; export données web, journaux — à compléter.
- ~~**RCC**~~ **Fait** : enregistrement automatique (POST /api/register) à la création d'un réseau depuis la webapp.
- **MCP** : exposition du serveur MCP depuis la webapp (si faisable via WebSocket) — backlog.
- **Stack** : Alpine.js (pas React — app déjà fonctionnelle, migration non prioritaire).
- **Déploiement** : servi par PHP F3 sur `https://www.rouaix.com/civium/app` ; build wasm-pack intégré dans le CI — à faire.

## Déploiement PHP / Infrastructure — Priorité haute

> Serveur PHP déployé sur `https://www.rouaix.com/civium`.

- Vérifier que `POST /api/register` reçoit bien les enregistrements desktop et les stocke
- Vérifier que `GET /api/networks` retourne les réseaux enregistrés
- Tester le flux complet : créer un réseau dans l'app desktop → apparaît dans `/api/networks` en < 5 s
- Tester le retry exponentiel : couper Internet lors de la création → ré-enregistrement automatique à la reconnexion
- Tester le flux magic link de bout en bout : email → lien → session → saisie `secret_b58` + PIN → clé dans IndexedDB
- Tester l'émission d'une alerte fraude depuis `/admin` → réception email admin + affichage bandeau dans l'app desktop
- Configurer les variables SMTP dans `config.ini` (SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASS) — variables présentes dans config.ini, déjà configurées dans config.prod.ini
- Configurer ADMIN_TOKEN en production (variable d'environnement ou `config.ini`)
- ~~Corriger l'URL dans les mails magic link~~ **Fait** : MagicLink::send() utilise maintenant APP_URL comme base (élimine double-préfixe) et SMTP configuré via config.ini au lieu de mail() brut.


## Chiffrement de la base de données locale — CRITIQUE

- ~~**Impact critique** : colonne `secret_b58` en texte clair~~ **Résolu** : toute la base est chiffrée avec AES-256-CBC (SQLCipher défaut), clé jamais stockée sur disque.
- ~~**Note build** : `bundled-sqlcipher-vendored-openssl` requiert `cmake` et `perl` sur la machine de build. À documenter dans CONTRIBUTING.md.~~ **Fait** : `scripts/setup-windows-dev.ps1` installe Strawberry Perl + cmake via winget et configure le PATH système. `.github/workflows/tauri.yml` ajoute automatiquement Strawberry Perl en tête du PATH sur `windows-latest`. Erreurs Rust corrigées (`anyhow`, permissions fs Tauri v2, `Direct`/`PairKey`). Build `cargo build` : exit 0.


## Sandbox WASM pour les plugins — Priorité haute

- ~~**Reste à faire** : brancher le `execute_cil_stub` sur le vrai store~~ **Fait** : `CilExecutor = Arc<dyn Fn(&str,&str)->String+Send+Sync>` ajouté dans `sandbox.rs` ; `WasmEngine::load()` accepte `Option<CilExecutor>` ; `cil_call` utilise le callback quand disponible, stub sinon. Le Tauri side injectera le vrai executor lors du chargement des plugins.


## Modération des contenus — Priorité haute

- ~~Côté PHP : ajouter une commande admin pour supprimer un message signalé sur le hub~~ **Fait** : `DELETE /admin/hub/message` dans AdminController + bouton "Supprimer" dans admin.html.


## ~~Pièces jointes dans les messages~~ — **Fait**

- ~~`MessageKind::File { filename, mime_type, size_bytes, chunks }`~~ : défini dans `civium-core/src/messaging/message.rs`.
- ~~Chunking + chiffrement~~ : `message_send_file` / `message_send_file_path` + `GroupKey::encrypt_chunk` (base64 O(n)).
- ~~Taille max~~ : 50 Mo via IPC, 500 Mo via temp path. Validé côté Tauri.
- ~~UI Dashboard~~ : bouton 📎, preview image inline, lecteur audio/vidéo, PDF iframe, téléchargement.


## Export des données utilisateur — Priorité haute

- ~~Ajouter un bouton "Exporter mes données" dans la section Identité du Dashboard~~ **Fait** : bouton "Télécharger mes données (.json)" dans Paramètres → Identité, appelle `export_data` (JSON complet messages/réseaux/membres/propositions).
- ~~Côté web : proposer un export similaire depuis l'interface `/app`~~ **Fait** : bouton "Télécharger mes données (.json)" dans Paramètres → Identité, génère un JSON blob navigateur (identité, réseaux, messages, documents, propositions, agenda, notifications).


## Nœuds bootstrap Civium — Priorité basse

- Les constantes `CIVIUM_ROOT_NODE_ADDR` et `CIVIUM_ROOT_NETWORK_CID_FULL` dans `civium-core/src/bootstrap.rs` sont actuellement vides (`""`) — un nœud fraîchement installé ne peut pas rejoindre le DHT Civium sans adresse manuelle
- Déployer au moins un nœud bootstrap permanent (ex. `bootstrap.civium.net` ou `www.rouaix.com`) et renseigner son adresse multiaddr dans ces constantes avant la release publique
- Documenter la procédure de démarrage d'un nœud bootstrap et son rôle dans la CONTRIBUTING.md


## Synchronisation web ↔ desktop — Priorité moyenne

- Documenter et implémenter la procédure de migration d'un réseau créé via le client web WASM vers l'app desktop (actuellement aucune procédure — l'utilisateur doit extraire le JSON depuis l'IndexedDB manuellement)
- Ajouter un bouton "Exporter vers l'app desktop" dans le client web qui génère un lien `civium://pair/<b58>` ou un fichier importable
- Ajouter dans l'onboarding desktop une option "Importer depuis le client web" en complément de "Restaurer depuis secret_b58"


## Messagerie — fonctionnalités manquantes — Priorité moyenne

- **Accusé de réception** : ajouter un statut "envoyé / livré / lu" sur les messages directs — le type `MessageDisplay` n'a pas de champ `read` ou `receipt` (actuellement aucun feedback visuel sur la réception)
- **Indicateur de frappe** : ajouter un "en train d'écrire..." visible par le destinataire lors de la saisie d'un message direct (typing indicator via événement P2P éphémère non persisté)
- **Réactions** : ajouter la possibilité de réagir à un message avec un emoji (persisté en CRDT G-Set pour éviter les conflits)
- ~~**Réponse à un message**~~ : **Fait** — champ `reply_to_id: Option<String>` sur `Message` (serde-compatible, ancien messages = None) ; bouton ↩ sur chaque message du fil, bandeau "En réponse à…" dans le composer, citation inline sous le nom de l'auteur. Commande `message_send` accepte `reply_to_id`.


## Agenda — fonctionnalités manquantes — Priorité moyenne

- **Récurrence dans l'UI** : le modèle `AgendaEvent` possède un champ `recurrence` mais le formulaire de création dans le Dashboard n'expose aucun champ de récurrence (quotidien, hebdomadaire, mensuel…) — à ajouter
- **Fuseaux horaires** : les événements sont stockés en Unix timestamp sans indication de fuseau — ajouter un sélecteur de timezone dans le formulaire et afficher les heures converties dans le fuseau local de l'utilisateur
- **Vue calendrier** : le Dashboard affiche les événements en liste chronologique — ajouter une vue calendrier mensuel/hebdomadaire pour une meilleure lisibilité
- **Rappels / notifications** : déclencher une notification OS (Tauri) X minutes avant un événement de l'agenda


## Préférences utilisateur persistées — Priorité basse

- Ajouter une table `user_preferences` dans `store.rs` (clé/valeur) pour persister les préférences UI : thème clair/sombre, langue, notifications activées, taille de police
- Ajouter un panneau "Préférences" dans le Dashboard avec un toggle thème clair/sombre (Tailwind `dark:` classes à activer)
- Persister le réseau et l'onglet sélectionnés au dernier usage pour restaurer l'état au redémarrage


## Client web — PWA et découvrabilité — Priorité basse

- ~~Ajouter un `manifest.json` pour rendre le client web installable (PWA)~~ **Fait** : `website/src/public/manifest.json` (icône, thème indigo, `start_url=/civium/app`) + `<link rel="manifest">` + `<meta name="theme-color">` dans `app.html`.
- ~~Ajouter un service worker pour le fonctionnement hors-ligne du client web~~ **Fait** : `website/src/www/sw.js` — cache-first pour assets statiques, network-first pour `/civium/api/`, enregistré dans `app.html`.


## ~~NAT traversal — Circuit Relay~~ — **Partiellement fait**

- ~~AutoNAT~~ : `libp2p::autonat` activé dans `CiviumBehaviour` — détection du type de NAT.
- ~~`relay` et `autonat` dans Cargo.toml~~ : features présentes.
- ~~Documenter workaround Cloudflare Tunnel~~ : **Fait** dans `CONTRIBUTING.md`.
- **Circuit relay natif** : différé — incompatible avec `with_websocket()` dans libp2p 0.55 (voir commentaire dans `behaviour.rs`). À activer quand libp2p ≥ 0.56 résoudra le conflit.

## Support Linux — Priorité haute

- ~~`tauri.conf.json` déclare `"targets": "all"` mais aucun build CI ni packaging Linux n'existe~~  **Fait** : `.github/workflows/tauri.yml` inclut `ubuntu-latest` (AppImage + `.deb` uploadés comme artifacts CI via `actions/upload-artifact@v4`).
- ~~Mentionner Linux comme plateforme supportée dans le README et la ROADMAP~~ **Fait** : ROADMAP.md Phase 5 mis à jour.


## Code signing des builds — Priorité haute

> Sans signature, macOS affiche "développeur non vérifié" et Windows SmartScreen bloque l'installation — bloquant pour tout déploiement public.

- Obtenir un certificat Apple Developer ID et configurer la signature macOS dans `.github/workflows/` + `tauri.conf.json`
- Obtenir un certificat Authenticode (ex. Sectigo EV) et configurer la signature Windows
- Intégrer la signature dans le workflow CI Tauri (à créer — voir section CI/CD) : build signé à chaque push sur `master`
- Configurer la notarisation Apple (`xcrun notarytool`) pour macOS 10.15+ (obligation depuis Catalina)


## Deep links `civium://` — Priorité haute

- ~~Déclarer le protocole `civium://`~~ **Fait** : `tauri-plugin-deep-link@2` ajouté (Cargo + npm), `plugins.deep-link.desktop[0].schemes=["civium"]` dans `tauri.conf.json`, `deep-link:default` dans capabilities, `.plugin(tauri_plugin_deep_link::init())` dans `lib.rs`.
- ~~Implémenter le handler~~ **Fait** : `App.tsx` utilise `onOpenUrl()` — parse `civium://<action>/<param>` et dispatch un `CustomEvent("civium:deep-link", {action, param})` à l'écoute de `Dashboard.tsx` (à brancher sur `pair` et `join`).
- ~~**Reste** : dans `Dashboard.tsx`, écouter `civium:deep-link`~~ **Fait** : `civium://join/<b58>` → `setJoinInviteLink` + `setShowJoinForm(true)` ; `civium://pair/<b58>` → `setPairLink` + `setShowPairCompleteForm(true)`.


## Consentement pour la publication dans l'annuaire — Priorité moyenne

- N'importe quel admin d'un réseau annuaire peut publier un membre (`directory_publish`) sans son consentement — aucun champ `consent_given` dans `DirectoryEntry`, aucune notification envoyée au membre concerné
- Ajouter un mécanisme de consentement : le membre publié reçoit une notification et doit accepter pour que l'entrée devienne visible
- Ajouter un bouton "Me retirer de cet annuaire" accessible à chaque membre depuis la section Annuaire du Dashboard


## Archivage d'un réseau — Priorité moyenne

- Il n'existe pas de mode "archivé" pour un réseau — la seule option est la suppression complète (`network_delete`), ce qui détruit tout l'historique
- Ajouter un champ `archived: bool` sur `NetworkData` : un réseau archivé est en lecture seule (consultation des messages autorisée, nouvelles publications et votes bloqués)
- Afficher les réseaux archivés dans une section séparée du Dashboard avec un badge "Archivé"
- Utile pour les associations dissoutes, projets terminés ou réseaux saisonniers


## Avatars et logos — Priorité moyenne

- Ajouter un champ `avatar_b58: Option<String>` (image encodée en base58 ou URL IPFS) sur `MemberRecord` dans `civium-core/src/network/member.rs` pour les photos de profil
- Ajouter un champ `logo_b58: Option<String>` sur `NetworkData` pour le logo d'un réseau
- Chiffrer les avatars avec la clé de groupe (cercles 0-2) — seuls les membres du réseau voient les photos
- Afficher l'avatar ou une initiale colorée dans la liste des membres, le fil de messages et le fil d'activité du Dashboard


## Interface admin RCC — fonctionnalités manquantes — Priorité moyenne

- Ajouter une recherche et un filtrage des réseaux enregistrés dans la page admin (`/admin`) : actuellement liste brute sans WHERE dynamique ni pagination avancée
- Ajouter une page de statistiques globales : nombre total de réseaux, évolution dans le temps, répartition par tier white-label
- Ajouter des actions de modération sur un réseau : suspension temporaire, suppression, signalement comme malveillant (alimentation automatique du RRM Global)
- Exposer les logs d'erreur PHP récents dans l'interface admin (lecture du fichier `tmp/php_error.log`) pour faciliter le débogage en production


## Unicité des noms de réseau — Priorité basse

- Deux réseaux indépendants peuvent avoir le même nom (ex. deux "Famille Martin" sur des nœuds différents) — le RCC indexe par `network_cid`, pas par `network_name`
- Ajouter une recherche de noms similaires lors de l'enregistrement RCC et avertir (sans bloquer) l'admin si un réseau du même nom existe déjà : "Un réseau nommé 'X' est déjà enregistré — assurez-vous que votre réseau est bien distinct"


## WebRTC — P2P direct navigateur-à-navigateur — Priorité moyenne

- Ajouter `libp2p-webrtc` comme transport dans `civium-core/Cargo.toml` (feature `wasm`) pour permettre des connexions P2P directes entre deux clients web WASM sans passer par un nœud desktop relay
- Actuellement les clients web ne peuvent se connecter qu'à un nœud desktop via WebSocket — deux navigateurs ne peuvent pas communiquer directement
- Nécessite un serveur STUN/TURN public (ou self-hosted) pour la négociation ICE


## Webhooks / intégrations tierces — Priorité moyenne

- Ajouter un système de webhooks dans `civium-core` : un réseau peut enregistrer une URL externe qui reçoit un POST JSON à chaque événement (nouveau message, nouveau membre, nouvelle proposition, alerte RCC)
- La signature HMAC du payload permet au service destinataire de vérifier l'authenticité
- Exposer la gestion des webhooks dans le Dashboard (ajouter, tester, supprimer) et dans le CLI
- Utile pour connecter Civium à des outils tiers : Zapier, Make (ex-Integromat), outils no-code, notifications Slack/Discord


## Support proxy SOCKS5 / Tor — Priorité moyenne

- Ajouter un champ `socks_proxy: Option<String>` dans `NodeConfig` (`civium-core/src/node/config.rs`) pour router le trafic libp2p via un proxy SOCKS5 ou Tor
- Exposer ce paramètre dans le panneau "Paramètres du nœud" du Dashboard
- Utile pour les utilisateurs dans des pays à censure ou souhaitant un anonymat réseau renforcé


## Rôles intermédiaires dans un réseau — Priorité moyenne

- Ajouter un rôle `Moderator` dans `civium-core/src/network/member.rs` : peut supprimer des messages et mettre en sourdine des membres, mais ne peut pas admettre/exclure des membres ni créer des propositions
- Ajouter un rôle `Observer` : voit les messages et les propositions, ne peut ni poster ni voter — utile pour inviter un auditeur externe ou un tuteur légal
- Exposer le changement de rôle dans le Dashboard (actuellement seul `Admin`/`Member` via `member_set_role`)


## Reconnexion automatique P2P — Priorité moyenne

- Implémenter une boucle de reconnexion avec backoff exponentiel vers les pairs connus qui viennent de se déconnecter (actuellement libp2p utilise un timeout idle de 60 s sans logique de retry custom)
- Persister la liste des pairs de confiance connus entre redémarrages (actuellement `MemoryStore` Kademlia = perdu à l'arrêt) dans une table `known_peers` SQLite
- Afficher dans le Dashboard un indicateur "Reconnexion en cours..." quand le nœud tente de rejoindre un pair connu

## Plugin Tâches — Priorité moyenne

- Créer un modèle `Task` dans `civium-core/src/` : titre, description, assigné (`assigned_to_cid`), échéance, statut (À faire / En cours / Terminé), priorité
- Ajouter la table `tasks` dans `store.rs` avec les fonctions CRUD
- Ajouter les commandes Tauri `task_create`, `task_list`, `task_update`, `task_delete`
- Ajouter une section "Tâches" dans le Dashboard avec vue liste et vue Kanban (colonnes par statut)
- Lier une tâche à un événement agenda ou à un message (référence croisée par ID)

## Export iCal (calendrier externe) — Priorité moyenne

- Ajouter une URL d'abonnement CalDAV ou webcal pour une synchronisation en continu (optionnel — backlog)


## ~~Multi-compte / multi-identité~~ — **Fait**

- ~~Migration 005~~ : nouvelle table `identities` (id AUTOINCREMENT, active, display_name) ; ligne existante migrée automatiquement depuis la table `identity`.
- ~~Store~~ : `list_identities`, `add_identity`, `switch_active_identity`, `delete_account`, `set_identity_display_name`.
- ~~Commandes Tauri~~ : `identity_list`, `identity_add_from_secret`, `identity_switch`, `identity_delete_account`, `identity_set_display_name`.
- ~~Dashboard~~ : sélecteur de compte dans la sidebar (▾ à côté du CID), switch immédiat + rechargement des réseaux, formulaire "Ajouter un compte" (import par secret_b58), suppression des comptes inactifs.


## Mode observateur dans un réseau — Priorité basse

- Ajouter un mode d'invitation "observateur" : la personne rejoint le réseau avec le rôle `Observer` — voit les messages et propositions, ne peut pas en créer (voir section Rôles intermédiaires)
- Utile pour : inviter un notaire, un auditeur, un membre fondateur qui ne participe plus activement, ou un parent surveillant le réseau de son enfant



## BIP39 — phrase mnémonique pour la clé — Priorité basse

- Ajouter le crate `bip39` dans `civium-core` pour dériver une phrase de 24 mots depuis la clé Ed25519
- Afficher la phrase mnémonique dans la section Identité du Dashboard en complément du `secret_b58` — plus facile à noter sur papier pour un utilisateur non-technique
- Permettre la restauration d'une identité depuis la phrase mnémonique dans l'onboarding

## Notifications push web — Priorité basse

- Remplacer le polling HTTP toutes les 30 s dans le client web par l'API Web Push (Service Worker + `PushManager`) pour recevoir les notifications même onglet fermé
- Ajouter la table `web_push_subscriptions` dans les migrations PHP pour stocker les endpoints Push par utilisateur
- Envoyer une notification push lors d'un nouveau message, invitation, ou alerte fraude


## ~~Révocation de clé publique (CID compromis)~~ — **Partiellement fait** — Priorité haute

- ~~Implémenter `RevocationRecord { cid_full, pub_key_b58, reason, revoked_at, signature_b58 }`~~ **Fait** : struct + vérification Ed25519 dans `civium-core/src/revocation.rs`. Export depuis `lib.rs`.
- ~~Store SQLite~~ **Fait** : migration 006, `save_revocation` / `list_revocations` / `is_revoked` dans `civium-tauri/src/store.rs`.
- ~~Commandes Tauri~~ **Fait** : `revocation_add` (valide + stocke), `revocation_list`, `revocation_check` enregistrées dans `lib.rs`.
- **Reste** : propagation P2P/DHT ; filtrage des messages des CID révoqués à la réception ; UI dans Dashboard ; lien avec rotation de clé de groupe (Forward secrecy)


## Synchronisation des cercles de confiance — Priorité moyenne

- Les cercles de confiance sont définis une seule fois à l'admission et ne sont jamais modifiables ni synchronisés entre appareils — chaque client maintient sa propre copie locale sans merge (`network.rs` : aucune fonction `set_member_circle()`)
- Ajouter une commande `member_change_circle` (déjà listée dans la section Cercles de confiance) et propager les changements via CRDT (Last-Write-Wins sur le champ `circle` avec timestamp)
- Synchroniser les changements de cercle entre appareils jumelés au même compte


## Optimisation des syncs P2P — delta plutôt que full — Priorité moyenne

- À chaque sync P2P, `SyncData` retourne la liste complète des membres (`Vec<MemberRecord>`) sans diff — pour un réseau de 1 000 membres, cela représente ~350 Ko transférés intégralement à chaque sync, même si rien n'a changé
- Ajouter un mécanisme de sync incrémentale : le nœud initiateur envoie un hash ou un numéro de version de sa liste de membres ; le nœud distant ne retourne que les enregistrements modifiés depuis ce hash
- Réduire drastiquement la bande passante pour les réseaux actifs avec de nombreux membres


## Intégrité des assets WASM et CDN — Priorité haute (CRITIQUE)

- ~~Alpine.js SRI hash cassé~~ **Fait** : `app.html` — Alpine 3.14.9 remplacé par **@3.14.8** sans `integrity` (version 3.14.9 introuvable sur jsdelivr, hash SRI invalide bloquait Alpine → boutons cassés, popups vides en local et prod). TweetNaCl 1.0.3 avec SRI conservé.
- **Reste à faire** : `esm.sh/blake3` (module ESM dynamique — SRI non applicable nativement) ; WASM integrity (hash SHA-256 du .wasm vérifié avant `WebAssembly.instantiate` — dépend du pipeline wasm-pack CI).
- Fichier : `website/src/app/modules/civium/views/app.html`

## Tests E2E frontend (Tauri + web) — Priorité haute (CRITIQUE)

- Zéro test E2E pour l'interface Tauri : pas de Playwright, pas de Cypress, pas de `tauri-driver` — toute régression UI est détectable seulement manuellement
- Les flux critiques (création identité, création réseau, invitation membre, vote, messagerie) ne sont jamais vérifiés automatiquement
- Ajouter `tauri-driver` + WebDriver (`@tauri-apps/api/mocks`) pour les tests Tauri, Playwright pour le client web
- Couvrir en priorité : onboarding complet, admission d'un membre, cycle de vote, envoi/réception message, connexion web (magic link)


## ActivityPub — implémentation manquante — Priorité haute (CRITIQUE)

- `desktop/civium-core/src/activitypub/mod.rs` ne contient que 38 lignes de structures de données (`ApStatus`, `ApFollower`, `ApPost`) — zéro logique implémentée
- Il n'existe pas d'endpoint `GET /.well-known/webfinger` : Mastodon/PeerTube ne peuvent pas découvrir le réseau
- Pas d'inbox ni d'outbox : impossible de recevoir ou d'envoyer des activités vers d'autres serveurs
- **CRITIQUE sécurité** : aucune vérification de signature HTTP sur les activités entrantes — un acteur malveillant peut usurper l'identité d'un serveur Mastodon en POSTant directement sur l'inbox
- Les tables SQLite `ap_followers` et `ap_posts` existent (`store.rs` lignes 131-145) mais ne sont jamais peuplées ni lues
- À implémenter : signature HTTP (draft-cavage-http-signatures-12), `webfinger`, `actor.json`, inbox/outbox, `Accept`/`Follow`/`Create Note` activi types
- Fichier : `desktop/civium-core/src/activitypub/mod.rs`

## ~~FFI mobile — fonctions manquantes~~ — **Fait** — Priorité haute

- ~~`network_create`, `member_admit`~~ **Fait** : créer un réseau local, admettre un membre.
- ~~`message_send_direct`~~ **Fait** : message direct chiffré avec clé de groupe.
- ~~`document_list/create`~~ **Fait** : lister et créer des documents chiffrés ; tables `proposals`, `votes`, `agenda_events`, `documents` ajoutées au SCHEMA FFI.
- ~~`agenda_list`~~ **Fait** : lister les événements agenda d'un réseau.
- ~~`proposal_list/create`~~ **Fait** : lister et créer des propositions.
- ~~`vote_cast`~~ **Fait** : voter sur une proposition (choix par index).
- **Reste** : `agenda_create` (FFI), synchronisation P2P mobile ; UniFFI `.udl` optionnel (macros `#[uniffi::export]` suffisent pour MVP).


## ~~Documents — absence de CRDT~~ — **Fait (LWW)**

- ~~LWW-Register~~ : champ `lamport_clock: u64` ajouté à `Document` ; méthode `Document::merge()` (clock plus élevé gagne, tiebreak par id lexicographique) ; `Document::update()` incrémente le clock.
- ~~`last_edited_by`~~ : nouveau champ exposé dans `DocumentInfo`.
- `document_update` utilise maintenant `doc.update(...)` au lieu de mutation directe.
- Limite de taille : non encore implémentée (document complet en mémoire).


## Dashboard — chargement mémoire non borné — Priorité haute

- ~~`message_list_paged`~~ : **Fait** — déjà implémenté (cursor rowid, limit 50 par défaut).
- ~~`agenda_list_paged`, `document_list_paged`, `proposal_list_paged`~~ : **Fait** — nouvelles commandes Tauri avec paramètres `limit`/`offset` (max 500) ; `list_*` existantes délèguent à `list_*_paged` avec limit=500.
- **Reste** : virtualisation DOM côté frontend (`@tanstack/react-virtual`) pour les listes très longues — actuellement tous les éléments sont montés dans le DOM même si la majorité est hors-écran.


## Agenda — fonctionnalités manquantes — Priorité moyenne

- `desktop/civium-core/src/agenda/mod.rs` : le champ `recurrence: Option<String>` existe mais est toujours `None` à la création — les événements récurrents ne sont pas implémentés
- Pas de validation des timestamps : `start_at > end_at` ou événement dans le passé ne génèrent aucune erreur
- Pas de gestion des conflits de plage horaire, pas d'invitation de membres à un événement (pas de champ `attendees`), pas de rappels/notifications
- Les événements ne sont pas synchronisés via CRDT inter-réseaux — locaux uniquement

## Partage de contenu entre réseaux (APC étendu) — Priorité basse

- L'APC (Accord de Partage Civium) est actuellement limité à la visibilité de l'annuaire des membres (`expose_member_directory: bool`) — deux réseaux connectés ne peuvent pas partager de messages, documents ou événements dans un espace commun
- Étendre `ShareTerms` dans `civium-core/src/connection/record.rs` pour inclure des permissions de partage de contenu : `share_messages`, `share_documents`, `share_events` par type
- Créer un "espace partagé" inter-réseaux chiffré avec une clé dérivée des deux clés de groupe


## Demandes du concepteur - Priorité basse

  ---
  Mobile

  - Parité fonctionnelle avec desktop/website (mêmes plugins, ergonomie tactile)

  ---
  Plugin futur (backlog)

  - Partage de ressources matérielles : distribution de calcul entre machines (rendu 3D, LLM distribué…) — à planifier après les points précédents

---