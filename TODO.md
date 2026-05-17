# TODO.md — Civium

---

## A Développer ou corriger dans desktop — Priorité haute
- L'app doit s'ouvrir sur un fil d'actualité qui affiche toutes les activités de tous les réseaux. Avec le type d'activité, le nom du réseau et le nom du user. Et le contenu de l'activité (message, événement, etc.)
- Les messages du serveur principal ne s'affichent pas dans l'app desktop.
- L'app doit pouvoir signaler un spam ou un abus etc. C'est à dire envoyer un message au serveur principal.
- ~~L'app doit pouvoir demander à rejoindre un réseau, au serveur principal avec un simple clic dans une liste des réseaux publics.~~ ✅ `hub_join_public_network` + modal Rejoindre (Dashboard.tsx:1976)
- ~~L'app doit afficher l'annuaire des réseaux publics (rejoindre sans invitation) et privés.~~ ✅ `hub_public_networks()` + modal Annuaire Civium (Dashboard.tsx:1930)
- ~~La messagerie est destinée à échanger des messages privés entre users et réseaux. On doit donc pouvoir choisir à qui on envoi les messages.~~ ✅ `message_send_direct` + sélection destinataire (commands.rs:726)
- Les messages peuvent contenir du texte, des fichiers, (audio, vidéo, images, pdf, etc.) et des événements (calendrier, tâches, etc.)
- Supprimer l'icône à côté du nom du réseau Civium.
- Dans les messages, afficher le nom du réseau et le nom du user qui envoie le message.
- La partie gouvernance est à revoir. Incompréhensible et inutilisable en l'état — trop complexe, le user doit juste cliquer sur des boutons.
- Le lien Fédération avec d'autres réseaux est faux : https://www.rouaix.com/civium/civium/users/civ18N7G42tR

## A Développer ou corriger dans websuite — Priorité haute
- Bugg dans /auth le lien de connexion est envoyé à n'importe qui et connecte le nouveau user à n'importe quel réseau sur le serveur principal.
- si un usesr se connecte via /auth et que sont email n'est connue, cela doit créer un nouveau réseaux (Noeud) et donc un nouveau user.
- On doit pouvoir supprimer les messages et alertes du seveur et elles doivent dans ce cas disparaitre dans l'app.
- dans admin on doit pouvoir dans la liste des réseaux, agir sur le réseau (supprimer, désactiver, etc.)
- dans admin on doit pouvoir dans la liste des users, agir sur le user (supprimer, désactiver, etc.)
- dans admin on doit pouvoir dans la liste des messages, agir sur le message (supprimer, désactiver, etc.)
- dans admin on doit pouvoir dans la liste des alertes, agir sur l'alerte (supprimer, désactiver, etc.)
- dans les mails envoyés par le serveur, l'url affichée est fausse : https://www.rouaix.com/civium/civium/auth/verify?token=2e652448fb6008edc4b36f658945983671a8c8eecc01b2af32dc9c75e4fa807d HTTP 404 (GET /civium/auth?erreur=lien_expire)
- dans website, je dois pouvoir me connecter ou créer un nouveau noeud(réseau) avec login et mot de passe. et que cela m'envoi par email les infos nécessaire pour se connecter à l'app desktop.



## Déploiement PHP / Infrastructure — Priorité haute

> Serveur PHP déployé sur `https://www.rouaix.com/civium`.

- Décider du domaine et de l'hébergement (civium.net ou civium.fr — Scaleway, ou rester sur rouaix.com/civium)
- Déployer le site PHP F3 en production (Apache/Nginx + PHP 8.x + MySQL)
- Vérifier que `POST /api/register` reçoit bien les enregistrements desktop et les stocke
- Vérifier que `GET /api/networks` retourne les réseaux enregistrés
- Tester le flux complet : créer un réseau dans l'app desktop → apparaît dans `/api/networks` en < 5 s
- Tester le retry exponentiel : couper Internet lors de la création → ré-enregistrement automatique à la reconnexion
- Tester le flux magic link de bout en bout : email → lien → session → saisie `secret_b58` + PIN → clé dans IndexedDB
- Tester l'émission d'une alerte fraude depuis `/admin` → réception email admin + affichage bandeau dans l'app desktop
- Configurer les variables SMTP dans `config.ini` (SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASS)
- Configurer ADMIN_TOKEN en production (variable d'environnement ou `config.ini`)
- Corriger l'URL dans les mails magic link (actuellement `/civium/auth/verify` → doit être `/civium/auth/verify`) — voir bug dans websuite


## Sécurité web (website PHP) — Priorité haute

- Ajouter une Content-Security-Policy (CSP) dans `website/src/www/index.php` — particulièrement important car Alpine.js est chargé depuis un CDN (jsdelivr.net)
- Ajouter du rate limiting sur `POST /auth` (magic link) : maximum N demandes par IP sur une fenêtre glissante, sinon blocage temporaire
- Ajouter du rate limiting sur `POST /api/register` : limiter les enregistrements par IP pour éviter le spam du RCC
- Ajouter un token CSRF sur le formulaire `/auth` (le formulaire JavaScript n'inclut aucun token actuellement)


## Backup et récupération de clé — Priorité haute

- **Desktop** : ajouter une fonction d'export de clé secrète vers fichier chiffré (ex. `.civium-backup` protégé par mot de passe) depuis la section Identité du Dashboard
- **Desktop** : ajouter une fonction d'import/restauration de clé depuis un fichier de backup
- **Desktop** : avertir l'utilisateur à la première connexion qu'il doit sauvegarder sa clé secrète (sans backup = perte définitive si machine perdue)
- **Web** : ajouter une procédure de récupération si l'IndexedDB est corrompue ou effacée (re-saisie du `secret_b58`)


## CI/CD — Priorité haute

- Créer `.github/workflows/rust.yml` : `cargo build`, `cargo test`, `cargo clippy` sur `civium-core` et `civium-cli` (actuellement aucun workflow pour le CLI Rust)
- Créer `.github/workflows/tauri.yml` : build de l'application desktop Tauri en CI (actuellement aucun build CI pour le desktop)
- Créer `.github/workflows/php.yml` : lint PHP (PHPStan ou PHP-CS-Fixer) + tests sur `website/src/` (actuellement aucun workflow PHP)
- Créer `.github/workflows/deploy.yml` : déploiement automatique sur push `master` vers le serveur de production (CD)


## Interopérabilité ActivityPub — Priorité moyenne

- Valider qu'un message Civium posté dans un réseau avec ActivityPub activé apparaît dans un fil Mastodon abonné (critère Phase 4 non coché)
- Tester `GET /.well-known/webfinger` et `GET /users/<cid>` depuis une instance Mastodon externe
- Tester la réception d'une note Mastodon via `POST /inbox` → apparaît en message Civium dans le Dashboard


## Phase 5 — Maturité — Priorité basse

- Planifier et mandater un audit de sécurité externe (cryptographie, P2P, CIL, plugin sandbox WASM)
- Créer la gouvernance du projet Civium lui-même (association loi 1901 ou fondation) — statuts, membres fondateurs, premier vote


## Suppression de compte et droit à l'oubli (RGPD) — Priorité haute

- Ajouter une commande Tauri `identity_delete` / `wipe_all_data` : supprime toute la BDD locale, les clés, et notifie les réseaux du départ du membre
- Ajouter une commande `network_leave` distincte de `network_delete` (quitter un réseau dont on n'est pas admin vs. supprimer un réseau qu'on administre)
- Ajouter dans le Dashboard une section "Danger" avec les actions destructrices (quitter réseau, supprimer compte, effacer toutes les données) — protégées par confirmation explicite
- Côté PHP RCC : ajouter `DELETE /api/networks/<cid>` (authentifié par signature) pour qu'un admin puisse dés-enregistrer son réseau du RCC


## UX Desktop — Priorité haute

- **Restauration de compte** : ajouter une option "J'ai déjà un compte" dans l'onboarding Tauri permettant de saisir son `secret_b58` pour restaurer une identité existante (actuellement l'onboarding crée toujours une nouvelle identité)
- **Gestion globale des erreurs** : ajouter un système de toasts/notifications dans le Dashboard pour afficher les erreurs Tauri visiblement à l'utilisateur (actuellement les erreurs du Dashboard tombent silencieusement dans `console.error`)
- **Notifications système (OS)** : implémenter les notifications natives Tauri (`tauri::notification`) pour alerter l'utilisateur en arrière-plan lors d'un nouveau message, invitation, ou alerte RCC (actuellement les notifications n'apparaissent que dans l'onglet Notifications du Dashboard)
- **Accessibilité (a11y)** : ajouter des attributs `aria-*` et des balises sémantiques HTML5 (`<nav>`, `<main>`, `<section>`) dans Dashboard.tsx et Onboarding.tsx (actuellement aucun attribut ARIA, interface entièrement basée sur des `<div>`)


## Chiffrement de la base de données locale — CRITIQUE

- Activer la feature `sqlcipher` dans `desktop/civium-tauri/src-tauri/Cargo.toml` (actuellement `rusqlite = { features = ["bundled"] }` — SQLite en clair)
- Ajouter `PRAGMA key = '<derived_key>'` à l'ouverture de la connexion dans `store.rs` pour chiffrer `civium.db`
- Dériver la clé SQLcipher depuis le PIN utilisateur (ou une clé stockée dans le trousseau OS via `keyring-rs`) — la clé ne doit pas être codée en dur
- **Impact critique** : actuellement la colonne `secret_b58` (clé privée Ed25519) dans la table `identity` est stockée en texte clair sur disque — quiconque accède au fichier `civium.db` peut voler l'identité


## Sandbox WASM pour les plugins — Priorité haute

- Implémenter une vraie sandbox d'exécution WASM pour les plugins tiers (wasmtime ou wasmer) dans `civium-core/src/plugin/`
- Actuellement le CIL (`civium-core/src/cil/mod.rs`) ne fait qu'une vérification de permissions en mémoire — un plugin installé tourne dans le même processus sans isolation
- Les plugins système (Gouvernance, CIL) peuvent rester natifs ; les plugins tiers doivent être exécutés dans la sandbox WASM avec accès limité au store via le CIL


## Sécurité et robustesse PHP — Priorité haute

- Valider la taille du payload sur `POST /api/register` avant `json_decode()` (limite ex. 64 Ko) — risque DoS via payload massif (actuellement `file_get_contents('php://input')` sans aucune limite)
- Ajouter un job de nettoyage périodique des magic links expirés (actuellement suppression uniquement au moment de la création d'un nouveau token pour le même email — les tokens d'autres emails expirent et restent en BDD indéfiniment)


## Performance desktop — Priorité moyenne

- Ajouter de la pagination sur la liste des messages dans le Dashboard (actuellement tous les messages d'un réseau sont chargés en mémoire React — à risque sur un réseau actif avec des milliers de messages)
- Ajouter de la pagination ou virtualisation (`react-window` ou équivalent) sur les listes longues : membres, documents, événements agenda, entrées annuaire
- Éviter de recharger toutes les données d'un réseau à chaque sélection — charger à la demande par onglet plutôt qu'en bloc (actuellement une dizaine de `refresh*` s'exécutent simultanément au changement de réseau)


## Mise à jour automatique de l'application — Priorité moyenne

- Configurer le plugin `tauri-plugin-updater` dans `desktop/civium-tauri/src-tauri/tauri.conf.json` pour permettre les mises à jour sans réinstallation manuelle
- Définir un endpoint de mise à jour (JSON signé) hébergé sur le serveur de production
- Afficher une notification dans le Dashboard quand une mise à jour est disponible


## Mobile — Parité fonctionnelle — Priorité moyenne

> Actuellement l'app mobile n'implémente que 4 écrans : Onboarding, Pairing, Réseaux, Messages.

- Ajouter l'écran Gouvernance mobile (liste des propositions, boutons de vote Oui/Non/Abstention)
- Ajouter l'écran Agenda mobile (liste des événements, création d'un événement)
- Ajouter l'écran Documents mobile (liste, lecture d'un document)
- Ajouter l'écran Annuaire mobile (recherche de membres et réseaux)
- Corriger le polling messages toutes les 5 s dans `MessagesScreen.tsx` — préférer un event listener P2P pour éviter la charge inutile


## Documentation développeur — Priorité moyenne

- Créer un `CONTRIBUTING.md` : comment builder le projet localement (`cargo tauri dev`, `wasm-pack build`, dépendances système), conventions de code, workflow de contribution
- Ajouter un guide de déploiement du serveur PHP (Apache/Nginx + PHP 8.x + MySQL + config.ini) — absent de toute la documentation actuelle


## Conformité RGPD et légale — Priorité haute

> Le site collecte des données personnelles (email admin dans le RCC, adresse IP). Obligation légale avant tout déploiement public.

- Créer une page `/mentions-legales` : éditeur, hébergeur, responsable de traitement
- Créer une page `/confidentialite` : quelles données sont collectées (email admin, IP, magic link token), pourquoi, durée de conservation, droits RGPD (accès, rectification, suppression)
- Ajouter un bandeau de consentement cookies si des cookies analytics sont utilisés (F3 session = cookie technique, exempt de consentement)
- Ajouter un lien "Mentions légales" et "Politique de confidentialité" dans le footer de toutes les pages (`layout.html`)
- Documenter clairement que l'email `admin_email` stocké dans le RCC est une donnée personnelle soumise au RGPD, et prévoir une procédure de suppression sur demande


## Modération des contenus — Priorité haute

- Ajouter une commande Tauri `message_delete(network_cid, message_id)` permettant à un admin de supprimer un message de n'importe quel membre dans son réseau (actuellement impossible — seul `member_remove` existe)
- Propager la suppression aux autres nœuds via P2P (message CRDT de type "tombstone")
- Ajouter un bouton "Signaler ce message" dans le fil de discussion, qui notifie les admins du réseau
- Côté PHP : ajouter une commande admin pour supprimer un message signalé sur le hub (voir item déjà listé dans la section websuite)


## Pièces jointes dans les messages — Priorité haute

> Item déjà listé dans la section Desktop. Détail technique pour l'implémentation :

- Ajouter `MessageKind::File { filename, mime_type, size_bytes, chunks: Vec<EncryptedChunk> }` dans `civium-core/src/messaging/message.rs` (actuellement seuls Thread, Direct, E2E existent)
- Implémenter le chunking et chiffrement des binaires avec la clé de groupe (cercles 0-2) ou la paire de clés (cercle 3)
- Définir une taille maximale de pièce jointe (suggestion : 50 Mo) et la faire respecter côté Tauri et PHP
- Ajouter l'UI d'envoi de fichier dans le Dashboard (bouton trombone, preview image, lecteur audio/vidéo inline)


## Backup automatique de la base de données — Priorité haute

- Implémenter une routine de backup périodique de `civium.db` : copie horodatée dans un sous-dossier `.backups/` à côté de la BDD, déclenchée au démarrage et toutes les X heures
- Limiter la rétention à N backups (garder les 7 derniers, supprimer les plus anciens)
- Afficher dans le Dashboard la date du dernier backup et un bouton "Sauvegarder maintenant"
- Avertir l'utilisateur au premier lancement qu'un backup régulier est indispensable (perte de clé privée = perte définitive de l'identité)


## Export des données utilisateur — Priorité haute

- Ajouter une commande Tauri `export_data` : génère un fichier ZIP ou JSON contenant messages, réseaux, membres, documents, événements agenda, propositions — dans un format portable lisible hors de l'app
- Ajouter un bouton "Exporter mes données" dans la section Identité du Dashboard
- Côté web : proposer un export similaire depuis l'interface `/app`


## Cercles de confiance — UI incomplète — Priorité moyenne

- Ajouter un sélecteur de cercle (0-3) au moment de l'admission d'un membre dans le Dashboard — actuellement le cercle est forcé à 1 (Connaissance) sans que l'admin puisse le choisir (`admitMember(p.cid_short, 1)` en dur dans `Dashboard.tsx:2224`)
- Ajouter une commande `member_change_circle(network_cid, member_cid, circle)` dans `commands.rs` pour modifier le cercle d'un membre déjà admis (inexistante actuellement)
- Afficher le cercle comme badge éditable sur chaque membre dans la liste, avec un menu déroulant pour le changer en un clic


## APC (Accord de Partage Civium) dans le Dashboard — Priorité moyenne

- Exposer les connexions inter-réseaux dans le Dashboard Tauri : actuellement `connection_list` et `connection_info` n'ont aucune commande Tauri — l'APC n'est visible que via le CLI (`civium connect info`)
- Ajouter les commandes Tauri `connection_list`, `connection_accept`, `connection_reject`, `connection_block`, `connection_info`
- Afficher dans le Dashboard la liste des réseaux connectés, le statut de chaque connexion (En attente / Active / Bloquée), et les termes de l'APC (réseaux concernés, date de signature, nonce)
- Permettre la renégociation ou la révocation d'un APC depuis le Dashboard


## Plugin marketplace — Priorité basse

- Actuellement les plugins s'installent uniquement depuis un fichier JSON local (`civium plugin install <path>`) — aucun dépôt central ni installation par URL
- Créer une spécification de dépôt de plugins (index JSON signé listant les plugins certifiés, leurs URLs de téléchargement et leurs hashes)
- Ajouter une commande `plugin install <url>` qui télécharge, vérifie le hash et installe le manifeste
- Ajouter une section "Catalogue de plugins" dans le Dashboard avec la liste des plugins disponibles, leur niveau de certification, et un bouton "Installer"


## Nœuds bootstrap Civium — Priorité basse

- Les constantes `CIVIUM_ROOT_NODE_ADDR` et `CIVIUM_ROOT_NETWORK_CID_FULL` dans `civium-core/src/bootstrap.rs` sont actuellement vides (`""`) — un nœud fraîchement installé ne peut pas rejoindre le DHT Civium sans adresse manuelle
- Déployer au moins un nœud bootstrap permanent (ex. `bootstrap.civium.net` ou `www.rouaix.com`) et renseigner son adresse multiaddr dans ces constantes avant la release publique
- Documenter la procédure de démarrage d'un nœud bootstrap et son rôle dans la CONTRIBUTING.md


## Mise en sourdine d'un membre — Priorité basse

- Ajouter une commande `member_mute(network_cid, member_cid, muted: bool)` : masque localement les messages d'un membre sans l'exclure du réseau (stocké uniquement en local, non propagé)
- Ajouter un bouton "Mettre en sourdine" sur chaque membre dans le Dashboard, distinct du bouton "Exclure"
- Ajouter une section "Membres en sourdine" dans les paramètres du réseau


## Limites white-label — enforcement `max_members` — Priorité basse

- Implémenter la vérification de `max_members` lors de `member_admit()` côté desktop (`civium-core` ou Tauri) : comparer le nombre de membres actifs avec la limite du tier du réseau récupérée via `GET /api/info`
- Actuellement `max_members` est retourné par l'API `/api/info` mais jamais vérifié à l'admission — les limites sont déclaratives sans enforcement


## Révocation des invitations — Priorité moyenne

- Ajouter une commande `invitation_revoke(network_cid, nonce_b58)` permettant à un admin de blacklister un lien d'invitation avant qu'il soit utilisé (actuellement les liens ne peuvent pas être révoqués, seulement expirer)
- Stocker les nonces révoqués dans une table `revoked_invitations` et les vérifier au moment de `submit_join_request`
- Afficher dans le Dashboard la liste des invitations actives (non expirées, non révoquées) avec bouton de révocation


## Protection du seul admin d'un réseau — Priorité moyenne

- Bloquer `member_remove` et `member_set_role(Member)` sur le dernier admin d'un réseau — actuellement un admin peut se retirer lui-même et laisser le réseau sans gouvernance
- Retourner une erreur explicite : "Vous êtes le seul admin de ce réseau. Nommez un autre admin avant de quitter."
- Appliquer la même protection dans le CLI (`civium member remove` et `civium member set-role`)


## Synchronisation web ↔ desktop — Priorité moyenne

- Documenter et implémenter la procédure de migration d'un réseau créé via le client web WASM vers l'app desktop (actuellement aucune procédure — l'utilisateur doit extraire le JSON depuis l'IndexedDB manuellement)
- Ajouter un bouton "Exporter vers l'app desktop" dans le client web qui génère un lien `civium://pair/<b58>` ou un fichier importable
- Ajouter dans l'onboarding desktop une option "Importer depuis le client web" en complément de "Restaurer depuis secret_b58"


## Configuration réseau avancée — Priorité moyenne

- Ajouter un panneau "Paramètres du nœud" dans le Dashboard permettant de modifier les ports TCP/WS, l'adresse externe annoncée au DHT, et les nœuds bootstrap — sans avoir à recompiler (actuellement tout est fixé au démarrage via `NodeConfig::default()`)
- Ajouter une commande Tauri `node_reconfigure(config)` qui redémarre le nœud P2P avec la nouvelle configuration
- Afficher l'état courant (ports actifs, adresses d'écoute, version du protocole) dans un onglet "Nœud" du Dashboard


## Versioning du protocole — Priorité moyenne

- Ajouter une négociation de version dans le handshake Civium : le nœud initiateur annonce sa version, le nœud distant répond avec la version commune maximale supportée
- Définir une politique de compatibilité (ex. mineur rétrocompatible, majeur incompatible) dans `civium-core/src/node/protocol.rs`
- Actuellement un nœud v0.2 connecté à un nœud v0.1 échoue silencieusement sans message d'erreur explicite


## Fil d'activité global — Priorité moyenne

- Ajouter une commande Tauri `activity_list_all` (sans `network_cid_short`) qui agrège les événements de tous les réseaux, triés par date décroissante
- Afficher ce fil global comme vue d'accueil de l'app (premier écran à l'ouverture — item déjà listé dans la section Desktop ci-dessus), avec le nom du réseau source sur chaque événement
- Ajouter un filtre par type d'événement dans le fil d'activité (messages, votes, admissions, alertes…)


## Recherche dans l'UI — Priorité moyenne

- Ajouter une barre de recherche dans le fil de messages du réseau sélectionné (actuellement aucune — seul l'annuaire dispose d'une recherche)
- Ajouter un filtre/recherche dans la liste des membres
- Ajouter une recherche full-text dans les documents du réseau


## Monitoring production — Priorité moyenne

- Connecter les erreurs PHP à un service externe (Sentry, ou un simple webhook vers un canal de monitoring) pour être alerté en temps réel des exceptions en production
- Les endpoints `/api/status` et `/hub/status` existent déjà — les brancher à un outil de monitoring externe (UptimeRobot ou équivalent) pour surveiller la disponibilité du RCC


## Tests unitaires civium-core — Priorité haute

- Sur les 40+ modules de `civium-core`, seul `wasm.rs` contient des tests (`#[cfg(test)]`) — les modules critiques `identity/`, `messaging/`, `network/`, `governance/`, `crypto/`, `node/`, `store` n'ont aucun test unitaire
- Ajouter des tests unitaires sur les fonctions critiques en priorité :
  - `identity/keypair.rs` : génération, sérialisation, désérialisation, signature/vérification
  - `crypto/group_key.rs` : chiffrement/déchiffrement, clés incorrectes, données corrompues
  - `network/network.rs` : `create`, `admit`, `remove_member`, `submit_join_request` — vérification des invariants
  - `governance/mod.rs` : `compute_result`, `compute_result_with_delegations`, garde-fou majoritaire
  - `messaging/mailbox.rs` : merge CRDT, déduplication, ordre des messages


## Tests et qualité — Priorité moyenne

- Écrire des tests d'intégration entre deux nœuds réels `civium-core` (lancer deux instances, échange de messages, vérification CRDT) — actuellement seuls des tests unitaires et WASM existent
- Ajouter `cargo test` dans le workflow CI pour `civium-core`
- Ajouter la commande `civium version` au CLI (affiche la version du binaire depuis `Cargo.toml`)
- Tester les migrations SQL en CI : lancer les migrations sur une BDD vierge et vérifier qu'elles passent toutes


## Messagerie — fonctionnalités manquantes — Priorité moyenne

- **Accusé de réception** : ajouter un statut "envoyé / livré / lu" sur les messages directs — le type `MessageDisplay` n'a pas de champ `read` ou `receipt` (actuellement aucun feedback visuel sur la réception)
- **Indicateur de frappe** : ajouter un "en train d'écrire..." visible par le destinataire lors de la saisie d'un message direct (typing indicator via événement P2P éphémère non persisté)
- **Réactions** : ajouter la possibilité de réagir à un message avec un emoji (persisté en CRDT G-Set pour éviter les conflits)
- **Réponse à un message** : ajouter un système de fil de réponse (`reply_to_id`) permettant de citer et répondre à un message spécifique


## Agenda — fonctionnalités manquantes — Priorité moyenne

- **Récurrence dans l'UI** : le modèle `AgendaEvent` possède un champ `recurrence` mais le formulaire de création dans le Dashboard n'expose aucun champ de récurrence (quotidien, hebdomadaire, mensuel…) — à ajouter
- **Fuseaux horaires** : les événements sont stockés en Unix timestamp sans indication de fuseau — ajouter un sélecteur de timezone dans le formulaire et afficher les heures converties dans le fuseau local de l'utilisateur
- **Vue calendrier** : le Dashboard affiche les événements en liste chronologique — ajouter une vue calendrier mensuel/hebdomadaire pour une meilleure lisibilité
- **Rappels / notifications** : déclencher une notification OS (Tauri) X minutes avant un événement de l'agenda


## Préférences utilisateur persistées — Priorité basse

- Ajouter une table `user_preferences` dans `store.rs` (clé/valeur) pour persister les préférences UI : thème clair/sombre, langue, notifications activées, taille de police
- Ajouter un panneau "Préférences" dans le Dashboard avec un toggle thème clair/sombre (Tailwind `dark:` classes à activer)
- Persister le réseau et l'onglet sélectionnés au dernier usage pour restaurer l'état au redémarrage


## Internationalisation (i18n) — Priorité basse

- Tous les textes de l'interface sont codés en dur en français dans `Dashboard.tsx` et `Onboarding.tsx` — aucune infrastructure i18n (pas de i18next ou équivalent)
- Ajouter `i18next` + `react-i18next` et extraire toutes les chaînes dans des fichiers de traduction `fr.json` / `en.json`
- Permettre à la communauté de contribuer des traductions via un fichier JSON versionné


## Client web — PWA et découvrabilité — Priorité basse

- Ajouter un `manifest.json` pour rendre le client web installable (PWA)
- Ajouter un service worker pour le fonctionnement hors-ligne du client web
- Ajouter un `sitemap.xml` pour le référencement du site de présentation


## Forward secrecy — rotation de la clé de groupe — CRITIQUE

- Quand un membre est exclu du réseau (`member_remove()`), la clé de groupe ChaCha20-Poly1305 n'est **pas renouvelée** — l'ancien membre qui possède une copie de la clé peut continuer à déchiffrer tous les futurs messages (`network.rs:271-279` : aucune action sur `group_key_b58` après suppression)
- Implémenter une rotation automatique de la clé de groupe à chaque exclusion de membre : générer une nouvelle `GroupKey`, la distribuer chiffrée aux membres restants via leurs paires de clés Ed25519, et invalider l'ancienne
- Loguer la rotation dans l'`audit_trail` avec le CID du membre exclu et l'horodatage


## Sécurité IPC Tauri — ACL des commandes — CRITIQUE

- Les 106 commandes Tauri sont exposées sans restrictions d'accès (`generate_handler!` dans `lib.rs:53-154`) — la CSP est explicitement désactivée (`"csp": null` dans `tauri.conf.json`) et aucun fichier de capabilities ne limite l'accès par WebView
- La commande `identity_show()` retourne `secret_b58` (clé privée Ed25519) en clair au frontend TypeScript — une vulnérabilité XSS dans les assets `dist/` permettrait d'exfiltrer la clé privée via IPC
- Activer le système de capabilities Tauri v2 : créer `src-tauri/capabilities/main.json` listant explicitement les commandes autorisées par fenêtre
- Supprimer `"csp": null` et définir une CSP stricte interdisant les scripts inline et les sources externes


## BDD corrompue — gestion d'erreur au démarrage — CRITIQUE

- Si `civium.db` est corrompu, l'erreur SQLite est capturée en `Err(_) => return` (`lib.rs:44`) sans aucune notification à l'UI — le Dashboard s'ouvre vide sans explication, l'utilisateur perd toutes ses données silencieusement
- Émettre un événement `civium://db-error` vers l'UI avec le message d'erreur SQLite, et afficher un écran d'erreur explicite avec les options : restaurer depuis un backup, ou réinitialiser (perte de données confirmée)
- Tenter automatiquement une restauration depuis le dernier backup dans `.backups/` avant de proposer la réinitialisation


## Watchdog du nœud P2P — CRITIQUE

- Si le thread P2P (`civium-core`) crashe, l'UI Tauri n'est **pas notifiée** — le Dashboard continue d'afficher l'indicateur "En ligne" alors que le nœud est mort (`lib.rs:47` : erreurs de démarrage silencieuses, `node.rs` : boucle sort sans émettre d'événement)
- Implémenter un watchdog : détecter la fin du task P2P (`spawn` + `JoinHandle`), émettre un événement `civium://node-crashed` vers l'UI, puis tenter un redémarrage automatique après délai exponentiel
- Afficher une bannière d'erreur rouge dans le Dashboard si le nœud P2P est inactif, avec un bouton "Redémarrer le nœud"


## Saturation DHT (DoS mémoire) — CRITIQUE

- Le store Kademlia utilise `MemoryStore` sans limite de capacité — un attaquant peut inonder le nœud avec des annonces DHT malveillantes et épuiser la RAM jusqu'au crash (`behaviour.rs:15-30` : `MemoryStore::new(peer_id)` sans paramètre de limite)
- Configurer `kad::store::MemoryStore` avec une capacité maximale (`max_records`, `max_provided_keys`) et rejeter les nouvelles entrées quand le seuil est atteint
- Combiner avec le rate limiting P2P (section suivante) pour limiter le nombre d'annonces acceptées par pair sur une fenêtre glissante


## Rate limiting P2P (protection DoS) — CRITIQUE

- Ajouter un compteur de requêtes par pair dans `civium-core/src/node/` : si un pair dépasse N requêtes `CiviumRequest` par seconde, ignorer les requêtes suivantes et éventuellement le déconnecter
- Le `HashMap<PeerId, PendingResponse>` dans `node.rs` croît sans limite de taille — ajouter un plafond et rejeter les requêtes dépassant le seuil
- Utiliser `libp2p::swarm::ConnectionLimits` pour limiter le nombre de connexions simultanées par pair et le nombre total de connexions entrantes
- Sans cette protection, un pair malveillant peut saturer le nœud avec 1 000 requêtes Sync par seconde → débordement mémoire et crash


## Migrations du schéma SQLite desktop — Priorité haute

- Le schéma SQLite local est appliqué via `CREATE TABLE IF NOT EXISTS` (un seul bloc `SCHEMA` dans `store.rs`) — si une mise à jour de l'app ajoute une colonne, elle n'est pas ajoutée aux bases existantes des utilisateurs
- Implémenter un système de migrations versionnées côté desktop, analogue au système PHP déjà en place (`website/src/migrations/`) : fichiers numérotés `001_initial.sql`, `002_add_column.sql`…, table `schema_migrations` SQLite, application automatique au démarrage de l'app
- Sans ce système, toute évolution du schéma (nouvelle table, nouvelle colonne) **casse silencieusement** les installations existantes


## NAT traversal — Circuit Relay — Priorité haute

- Deux nœuds derrière NAT sans IP publique ne peuvent pas se connecter directement — `libp2p-circuit-relay` et `libp2p-autonat` sont absents des features de `civium-core/Cargo.toml`
- Ajouter les features `circuit-relay` et `autonat` dans `civium-core` pour permettre le relay via un nœud tiers (ex. le nœud bootstrap Civium) quand la connexion directe échoue
- Documenter le workaround actuel (Cloudflare Tunnel via `external_addr`) dans la CONTRIBUTING.md en attendant l'implémentation native


## Validation des inputs utilisateur — Priorité haute

- Ajouter une validation côté Rust dans `civium-core` avant tout `Network::create()`, `submit_join_request()`, `message_send()` :
  - Nom de réseau et nom d'affichage : 1–64 caractères, pas de caractères de contrôle
  - Corps de message : longueur maximale configurable (suggestion : 64 Ko)
  - CID : longueur et format Base58 validés
- Actuellement seule une vérification `!value.trim().is_empty()` est faite côté React — un nom de 100 000 caractères passe sans erreur
- Retourner des erreurs `CiviumError::Validation` explicites avec le champ et la contrainte violée


## Support Linux — Priorité haute

- `tauri.conf.json` déclare `"targets": "all"` mais aucun build CI ni packaging Linux n'existe (pas de AppImage, `.deb`, `.rpm`)
- Ajouter une cible Linux dans le workflow CI Tauri (à créer) : build AppImage + `.deb` sur Ubuntu
- Tester le nœud P2P (TCP, QUIC, WebSocket, mDNS) sur Linux avant la release publique — libp2p supporte Linux mais non validé en CI
- Mentionner Linux comme plateforme supportée dans le README et la ROADMAP


## Code signing des builds — Priorité haute

> Sans signature, macOS affiche "développeur non vérifié" et Windows SmartScreen bloque l'installation — bloquant pour tout déploiement public.

- Obtenir un certificat Apple Developer ID et configurer la signature macOS dans `.github/workflows/` + `tauri.conf.json`
- Obtenir un certificat Authenticode (ex. Sectigo EV) et configurer la signature Windows
- Intégrer la signature dans le workflow CI Tauri (à créer — voir section CI/CD) : build signé à chaque push sur `master`
- Configurer la notarisation Apple (`xcrun notarytool`) pour macOS 10.15+ (obligation depuis Catalina)


## Deep links `civium://` — Priorité haute

- Déclarer le protocole `civium://` dans `desktop/civium-tauri/src-tauri/tauri.conf.json` (section `app.security.assetProtocol` ou `plugins.deep-link`) pour que l'OS puisse ouvrir l'app depuis un lien
- Implémenter un handler dans `src-tauri/src/` qui parse le deep link à l'ouverture de l'app (`civium://pair/<b58>` → déclenche le pairing, `civium://join/<cid>` → ouvre le modal de rejoindre)
- Gérer le cas où l'app est fermée : l'OS doit l'ouvrir, puis lui transmettre le lien (actuellement `civium://` est seulement utilisé en interne comme nom d'événement Tauri, pas comme protocole système)
- Sans ce handler, les liens d'invitation par email et les liens de pairing QR code ne fonctionnent pas depuis un navigateur ou un client mail


## Logs applicatifs desktop — Priorité haute

- Remplacer les `eprintln!` / `println!` dispersés dans `node.rs`, `root_connect.rs` et ailleurs par des appels `tracing` structurés (`tracing` est déjà en dépendance mais peu utilisé)
- Configurer un subscriber `tracing` avec rotation de fichiers (crate `tracing-appender`) : écriture dans `<data_dir>/civium.log` avec rotation quotidienne et rétention de 7 jours
- Ajouter des champs de contexte sur chaque span : `network_cid`, `peer_id`, `operation` — indispensable pour déboguer des problèmes de sync en production
- Exposer dans le Dashboard un bouton "Télécharger les logs" pour faciliter les rapports de bug


## Sessions web — durée et révocation — Priorité moyenne

- La session PHP après validation du magic link n'a pas de durée configurée explicitement — elle expire selon le `gc_maxlifetime` du serveur (défaut 24 min) ce qui peut déconnecter l'utilisateur de façon imprévisible pendant une longue session
- Configurer une durée de session explicite dans `AuthController.php` (ex. 30 jours avec cookie `remember_me`, ou 8h pour une session normale)
- Implémenter un logout global ("se déconnecter de tous les appareils") : stocker les sessions en BDD (table `sessions`) et les invalider toutes d'un coup à la demande


## Documentation du serveur MCP — Priorité moyenne

- Les 6 resources exposées par le serveur MCP (`civium://networks`, `civium://network/{cid}/members`, `messages`, `proposals`, `agenda`, `documents`) ne sont documentées que dans le code `mcp.rs`
- Créer un fichier `docs/mcp.md` : liste des resources, format JSON retourné, exemples de requêtes `resources/list` et `resources/read`, authentification Bearer token, limitations (lecture seule, CIL appliqué)
- Ajouter un lien vers cette doc dans le Dashboard (section MCP) pour que les utilisateurs sachent comment connecter un assistant IA à leur nœud


## Internationalisation des emails — Priorité basse

- Les emails envoyés par le serveur PHP (magic link, alertes fraude) sont uniquement en français codé en dur dans `MagicLink.php` et `Mailer.php`
- Créer un système de templates email multilingues (fr/en minimum) : détecter la langue préférée de l'utilisateur depuis le `Accept-Language` HTTP ou un champ `lang` en BDD, sélectionner le template correspondant
- Fournir les templates dans `website/src/templates/emails/fr/` et `en/`


## Retour visuel après copie (clipboard) — Priorité basse

- Les boutons "Copier" dans le Dashboard (`navigator.clipboard.writeText()`) ne donnent aucun retour visuel — l'utilisateur ne sait pas si la copie a réussi (pas de toast, pas de changement d'icône)
- Ajouter un feedback post-copie : icône ✓ pendant 2 s, ou mini-toast "Copié !" — applicable à tous les boutons copie : CID, secret, adresse P2P, token MCP, lien d'invitation


## Workflow GitHub Releases — Priorité haute

- Créer `.github/workflows/release.yml` déclenché sur tag `v*` : build Tauri pour Windows (`.exe`/`.msi`), macOS (`.dmg`), Linux (`.AppImage`/`.deb`), puis création automatique de la release GitHub avec les artefacts signés
- Intégrer la signature de code (voir section Code signing) dans ce workflow
- Les releases sont actuellement créées manuellement — bloquant pour distribuer l'app à des utilisateurs non-techniques


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


## Rotation du token MCP — Priorité moyenne

- Le token Bearer MCP est généré une fois au démarrage du nœud et ne change jamais — aucune expiration, aucun endpoint de rotation (`mcp.rs:52-66`)
- Ajouter une commande Tauri `mcp_rotate_token` qui régénère le token sans redémarrer le nœud et invalide l'ancien immédiatement
- Ajouter une date d'expiration configurable (ex. 30 jours) avec renouvellement automatique et notification dans le Dashboard


## Interface admin RCC — fonctionnalités manquantes — Priorité moyenne

- Ajouter une recherche et un filtrage des réseaux enregistrés dans la page admin (`/admin`) : actuellement liste brute sans WHERE dynamique ni pagination avancée
- Ajouter une page de statistiques globales : nombre total de réseaux, évolution dans le temps, répartition par tier white-label
- Ajouter des actions de modération sur un réseau : suspension temporaire, suppression, signalement comme malveillant (alimentation automatique du RRM Global)
- Exposer les logs d'erreur PHP récents dans l'interface admin (lecture du fichier `tmp/php_error.log`) pour faciliter le débogage en production


## Benchmarks de performance — Priorité moyenne

- Ajouter un répertoire `desktop/civium-core/benches/` avec des benchmarks `criterion` sur les opérations critiques :
  - Chiffrement/déchiffrement ChaCha20-Poly1305 (messages de 1 Ko, 10 Ko, 1 Mo)
  - Sérialisation/désérialisation CBOR d'un `CiviumRequest`
  - Merge CRDT d'une mailbox de 1 000 messages
  - Vérification de signature Ed25519
- Intégrer `cargo bench` dans la CI pour détecter les régressions de performance


## Onboarding — indicateur de progression — Priorité moyenne

- Ajouter une barre de progression ou des étapes numérotées ("2 / 5") dans `Onboarding.tsx` — actuellement les 6 étapes (welcome → identity → choice → create/join → done) se succèdent sans aucun indicateur visuel d'avancement
- Ajouter une option "J'ai déjà un compte" dès l'étape `welcome` permettant de saisir son `secret_b58` pour restaurer une identité existante (actuellement absent — voir section UX Desktop)


## Mémoire WASM — pagination des données — Priorité moyenne

- Les fonctions `wasm-bindgen` dans `civium-core/src/wasm.rs` retournent des blobs complets : `network_create` retourne tout `NetworkData` (membres inclus), `vote_compute` reçoit tous les votes en JSON — risque de saturation mémoire pour des réseaux avec des milliers de membres ou de votes
- Ajouter une pagination côté WASM : `members_list(offset, limit)`, `votes_list(proposal_id, offset, limit)` plutôt que des retours complets
- Afficher un avertissement dans le client web si un réseau dépasse un seuil (ex. 500 membres) et suggérer l'app desktop


## Nœud headless — fichier de configuration — Priorité moyenne

- Le CLI supporte le mode serveur headless via `civium node start` avec des flags, mais il n'y a pas de fichier de configuration persistant (`civium.toml` ou `.env`) — relancer le nœud après un redémarrage serveur nécessite de retaper tous les flags
- Ajouter la lecture d'un fichier `civium.toml` dans le répertoire de données (ou via `--config`) : `listen_tcp`, `listen_ws`, `external_addr`, `bootstrap_peers`, `announce`, `auto_accept_connections`
- Fournir un exemple `civium.toml.example` et un template de service systemd dans le dépôt (`deploy/civium.service`)


## Affichage des timestamps — Priorité basse

- Les timestamps sont bien stockés en UTC (secondes Unix) dans toute la base de données — ✅
- Côté CLI, les timestamps sont affichés en secondes Unix brutes — les convertir en date/heure locale lisible (`2026-05-17 14:32:10`) dans toutes les commandes `list`
- Côté Dashboard, vérifier que l'affichage utilise le fuseau local de l'utilisateur et non UTC (JavaScript `new Date(ts * 1000).toLocaleString()`)


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


## API REST documentée (OpenAPI) — Priorité moyenne

- Documenter les endpoints PHP existants (`/api/register`, `/api/networks`, `/api/status`, `/hub/*`) avec une spécification OpenAPI 3.0 (fichier `openapi.yaml` dans `website/`)
- Générer une page de documentation interactive (Swagger UI ou Redoc) accessible à `/api/docs`
- Le serveur MCP du nœud (`mcp.rs`) n'est pas une API REST standard — ajouter un endpoint REST léger (`/api/v1/`) sur le nœud Tauri pour les intégrateurs qui ne veulent pas implémenter MCP


## Tests de fuzzing — Priorité moyenne

- Créer des targets de fuzzing avec `cargo-fuzz` sur les fonctions critiques de `civium-core` :
  - Parsing des messages CBOR (`CiviumRequest` / `CiviumResponse`)
  - Vérification des signatures Ed25519 (`CiviumKeypair::verify`)
  - Parsing des adresses multiaddr
  - Déchiffrement ChaCha20-Poly1305 (input aléatoire → pas de panic)
- Intégrer le fuzzing dans la CI (GitHub Actions) avec un budget de temps limité (ex. 60 s par target)


## Rôles intermédiaires dans un réseau — Priorité moyenne

- Ajouter un rôle `Moderator` dans `civium-core/src/network/member.rs` : peut supprimer des messages et mettre en sourdine des membres, mais ne peut pas admettre/exclure des membres ni créer des propositions
- Ajouter un rôle `Observer` : voit les messages et les propositions, ne peut ni poster ni voter — utile pour inviter un auditeur externe ou un tuteur légal
- Exposer le changement de rôle dans le Dashboard (actuellement seul `Admin`/`Member` via `member_set_role`)


## Markdown dans les messages — Priorité moyenne

- Ajouter `react-markdown` dans `desktop/civium-tauri/package.json` et rendre le corps des messages en Markdown (gras, italique, liens, blocs de code, listes)
- Ajouter une toolbar de mise en forme basique dans le champ de saisie (boutons **G** *I* `` ` `` lien)
- S'assurer que les liens externes sont ouverts dans le navigateur système (`shell.open`) et non dans la WebView Tauri (prévention des redirections malveillantes)
- Sanitiser le Markdown avant rendu pour éviter toute injection HTML (`rehype-sanitize`)


## Reconnexion automatique P2P — Priorité moyenne

- Implémenter une boucle de reconnexion avec backoff exponentiel vers les pairs connus qui viennent de se déconnecter (actuellement libp2p utilise un timeout idle de 60 s sans logique de retry custom)
- Persister la liste des pairs de confiance connus entre redémarrages (actuellement `MemoryStore` Kademlia = perdu à l'arrêt) dans une table `known_peers` SQLite
- Afficher dans le Dashboard un indicateur "Reconnexion en cours..." quand le nœud tente de rejoindre un pair connu


## Audit trail immuable — Priorité moyenne

- La table `admin_actions` trace les actions admin mais reste une table SQLite modifiable — un admin avec accès direct à `civium.db` peut réécrire l'historique
- Ajouter une chaîne de hachage (hash chain) sur les `admin_actions` : chaque entrée inclut le hash de l'entrée précédente, signé par la clé Ed25519 de l'acteur — toute modification rompt la chaîne et est détectable
- Exposer la vérification de l'intégrité du journal dans le Dashboard : bouton "Vérifier l'audit trail" qui recompute la chaîne et signale toute rupture


## Nettoyage et maintenance de la BDD — Priorité moyenne

- Ajouter une commande Tauri `db_vacuum` : lance `PRAGMA vacuum` + `PRAGMA optimize` pour compacter `civium.db` après des suppressions importantes
- Ajouter une commande `db_purge_messages(network_cid, before_timestamp)` : supprime les messages plus anciens que N jours (rétention configurable par réseau)
- Afficher dans le Dashboard la taille actuelle de `civium.db` et la date du dernier compactage, avec un bouton "Optimiser la base de données"


## Plugin Tâches — Priorité moyenne

- Créer un modèle `Task` dans `civium-core/src/` : titre, description, assigné (`assigned_to_cid`), échéance, statut (À faire / En cours / Terminé), priorité
- Ajouter la table `tasks` dans `store.rs` avec les fonctions CRUD
- Ajouter les commandes Tauri `task_create`, `task_list`, `task_update`, `task_delete`
- Ajouter une section "Tâches" dans le Dashboard avec vue liste et vue Kanban (colonnes par statut)
- Lier une tâche à un événement agenda ou à un message (référence croisée par ID)


## Pagination dans le CLI — Priorité moyenne

- Ajouter `--limit` et `--offset` (ou `--page` / `--per-page`) sur toutes les commandes `list` du CLI : `msg list`, `member list`, `governance list`, `doc list`, `agenda list`, `activity list`
- Actuellement toutes les listes sont affichées intégralement sans limite — un réseau avec des milliers de messages génère une sortie illisible
- Ajouter aussi un flag `--json` pour obtenir la sortie en JSON plutôt qu'en texte formaté (utile pour scripter)


## Export iCal (calendrier externe) — Priorité moyenne

- Ajouter une sérialisation des `AgendaEvent` au format iCalendar (`.ics` / RFC 5545) dans `civium-core/src/agenda/`
- Ajouter une commande Tauri `agenda_export_ics(network_cid)` qui génère un fichier `.ics` téléchargeable
- Ajouter un bouton "Exporter vers calendrier" dans la section Agenda du Dashboard (ouvre le fichier `.ics` → Google Calendar / Apple Calendar / Thunderbird l'importent automatiquement)
- Ajouter une URL d'abonnement CalDAV ou webcal pour une synchronisation en continu (optionnel — backlog)


## Multi-compte / multi-identité — Priorité moyenne

- Actuellement la table `identity` impose `id = 1` — un seul compte par installation, aucun switch possible
- Permettre plusieurs profils sur le même nœud : modifier le schéma SQLite pour supporter N identités avec une notion d'identité "active"
- Ajouter dans le Dashboard un sélecteur de profil (icône utilisateur → "Changer de compte") et un bouton "Ajouter un compte"
- Utile pour les cas famille (plusieurs membres sur le même ordinateur) ou test/développement


## TTL et nettoyage des nœuds DHT morts — Priorité moyenne

- Le store Kademlia utilise `MemoryStore` avec le TTL par défaut libp2p (~24h) — les nœuds offline restent dans les tables de routage jusqu'à expiration naturelle
- Implémenter un ping périodique actif des pairs connus pour détecter rapidement les nœuds devenus injoignables et les retirer des tables de routage
- Persister les pairs de confiance connus entre redémarrages (actuellement `MemoryStore` = perdu à l'arrêt) pour accélérer la reconnexion au DHT


## Thème web — adaptation au système — Priorité basse

- Le client web (`app.html`) impose un thème sombre fixe (`background: #0f1117`) sans respecter `prefers-color-scheme`
- Remplacer les couleurs codées en dur par des variables CSS et ajouter une media query `@media (prefers-color-scheme: light)` pour adapter l'interface au thème OS de l'utilisateur


## Mode observateur dans un réseau — Priorité basse

- Ajouter un mode d'invitation "observateur" : la personne rejoint le réseau avec le rôle `Observer` — voit les messages et propositions, ne peut pas en créer (voir section Rôles intermédiaires)
- Utile pour : inviter un notaire, un auditeur, un membre fondateur qui ne participe plus activement, ou un parent surveillant le réseau de son enfant


## Complétion shell CLI — Priorité basse

- Ajouter `clap_complete` dans `civium-cli/Cargo.toml` et exposer une commande `civium completions <shell>` (bash, zsh, fish, powershell)
- L'aide `--help` par sous-commande est déjà excellente — la complétion est le complément naturel


## BIP39 — phrase mnémonique pour la clé — Priorité basse

- Ajouter le crate `bip39` dans `civium-core` pour dériver une phrase de 24 mots depuis la clé Ed25519
- Afficher la phrase mnémonique dans la section Identité du Dashboard en complément du `secret_b58` — plus facile à noter sur papier pour un utilisateur non-technique
- Permettre la restauration d'une identité depuis la phrase mnémonique dans l'onboarding


## Cache mémoire côté Tauri — Priorité basse

- Actuellement chaque commande Tauri ouvre une nouvelle connexion SQLite et relit la BDD — ajouter un `AppState` avec un cache en mémoire pour les données fréquemment lues (identité, liste des réseaux, membres actifs)
- Invalider le cache sur les événements `civium://sync-completed` et les mutations locales
- Commencer par les données les plus lues : identité, liste des réseaux, liste des membres du réseau sélectionné


## Télémétrie opt-in — Priorité basse

- Ajouter un mécanisme de collecte de métriques anonymisées opt-in (désactivé par défaut) : version du client, nombre de réseaux, nombre de membres, OS, erreurs fréquentes
- Envoyer périodiquement ces métriques agrégées au RCC (`POST /api/telemetry`) pour permettre de mesurer l'adoption et prioriser les bugs
- Afficher dans le Dashboard un panneau "Contribuer à l'amélioration de Civium" avec description des données collectées et toggle opt-in


## Notifications push web — Priorité basse

- Remplacer le polling HTTP toutes les 30 s dans le client web par l'API Web Push (Service Worker + `PushManager`) pour recevoir les notifications même onglet fermé
- Ajouter la table `web_push_subscriptions` dans les migrations PHP pour stocker les endpoints Push par utilisateur
- Envoyer une notification push lors d'un nouveau message, invitation, ou alerte fraude


## Révocation de clé publique (CID compromis) — Priorité haute

- Il n'existe aucun mécanisme pour annoncer qu'un CID est compromis — une clé Ed25519 volée reste valide indéfiniment dans tous les réseaux où le membre est inscrit (`identity/cid.rs` : CID immuable, aucun champ `revoked`)
- Implémenter un message de révocation signé par la clé compromise elle-même (ou par un admin réseau) : `RevocationRecord { cid_full, reason, revoked_at, signature }`
- Propager la révocation via P2P et DHT ; les nœuds qui reçoivent une révocation valide bloquent les messages signés par ce CID
- Lier à la rotation de la clé de groupe (section Forward secrecy) : révocation d'un membre → nouvelle clé groupe


## Rate limit sur l'envoi de messages — Priorité haute

- Un membre peut envoyer des milliers de messages par seconde sans aucune limite — il n'existe pas de rate limit par membre dans `civium-core/src/messaging/` ni dans le protocole de sync
- Ajouter un compteur de messages par membre par fenêtre glissante (ex. 60 messages/minute) dans `civium-core` ; dépasser le seuil produit une erreur `CiviumError::RateLimited`
- Appliquer le même rate limit côté réception P2P : ignorer les messages d'un pair qui dépasse le seuil et lui notifier le blocage temporaire


## Erreurs frontend web — handler global — Priorité haute

- Le client web (`app.html`) n'a aucun handler global d'erreur JavaScript — les crashes WASM et les promises rejetées tombent silencieusement dans la console, l'utilisateur voit une interface gelée sans explication
- Ajouter `window.addEventListener('error', ...)` et `window.addEventListener('unhandledrejection', ...)` pour capturer toutes les erreurs non traitées et les afficher à l'utilisateur (bandeau rouge + message)
- Ajouter des attributs `integrity` (SRI) sur les scripts chargés depuis CDN (Alpine.js, TweetNaCl.js, blake3) pour détecter toute altération


## `node_modules` dans le dépôt — Priorité haute

- Le dossier `node_modules` de `desktop/civium-tauri/` est tracké dans Git — il alourdit le dépôt et empêche `npm audit` de fonctionner correctement
- Vérifier et corriger le `.gitignore` de `desktop/civium-tauri/` pour exclure `node_modules/`
- Supprimer `node_modules` de l'historique Git si présent (via `git filter-repo` ou BFG Repo Cleaner)
- Intégrer `npm audit` dans le workflow CI PHP/JS pour détecter les vulnérabilités dans les dépendances


## Synchronisation des cercles de confiance — Priorité moyenne

- Les cercles de confiance sont définis une seule fois à l'admission et ne sont jamais modifiables ni synchronisés entre appareils — chaque client maintient sa propre copie locale sans merge (`network.rs` : aucune fonction `set_member_circle()`)
- Ajouter une commande `member_change_circle` (déjà listée dans la section Cercles de confiance) et propager les changements via CRDT (Last-Write-Wins sur le champ `circle` avec timestamp)
- Synchroniser les changements de cercle entre appareils jumelés au même compte


## Optimisation des syncs P2P — delta plutôt que full — Priorité moyenne

- À chaque sync P2P, `SyncData` retourne la liste complète des membres (`Vec<MemberRecord>`) sans diff — pour un réseau de 1 000 membres, cela représente ~350 Ko transférés intégralement à chaque sync, même si rien n'a changé
- Ajouter un mécanisme de sync incrémentale : le nœud initiateur envoie un hash ou un numéro de version de sa liste de membres ; le nœud distant ne retourne que les enregistrements modifiés depuis ce hash
- Réduire drastiquement la bande passante pour les réseaux actifs avec de nombreux membres


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