# ROADMAP.md — Civium

Suivi du développement, phase par phase. Chaque tâche cochée = code mergé sur `master`.

**Spec de référence :** [README.md](README.md) | **Stack :** [STACK.md](STACK.md)

---

## Statut global

| Phase | Nom | Statut |
|---|---|---|
| — | Site web de présentation | 🔲 Non démarré |
| 0 | MVP | 🚧 En cours |
| 1 | Gouvernance & Annuaires | ⏳ En attente Phase 0 |
| 2 | Services & Intégrations | ⏳ En attente Phase 1 |
| 3 | Applications & Écosystème | ⏳ En attente Phase 2 |
| 4 | Maturité | ⏳ En attente Phase 3 |

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

## Phase 0 — MVP `~12 semaines`

> Valider que deux réseaux souverains peuvent se connecter sans serveur central.

### Semaines 1–2 — Protocole de base (`civium-core`)

- [x] Génération de paires de clés Ed25519
- [x] Dérivation du CID depuis la clé publique
- [x] Création et démarrage d'un nœud local
- [x] Transport libp2p TCP + QUIC
- [x] Découverte de pairs via DHT Kademlia
- [x] Chiffrement des connexions via Noise Protocol

### Semaines 3–4 — Identité et membres

- [x] Création de compte (CID membre + clé Ed25519)
- [x] Format identifiant réseau (`<cid_membre_court>@<cid_réseau_court>`)
- [x] Nom affiché par réseau (choix libre, unique dans le réseau)
- [x] Invitation d'un membre (lien ou CID)
- [x] Validation d'admission par l'admin
- [x] Cercles de confiance 0, 1 et 2

### Semaines 5–6 — Messagerie

- [x] Chiffrement E2E applicatif (clé de groupe — cercles 0-2)
- [x] Messages directs entre membres d'un même réseau
- [x] Fils de discussion dans le réseau
- [x] Synchronisation CRDT entre membres connectés
- [x] Queue locale + resync à la reconnexion (offline-first)

### Semaines 7–8 — Connexion inter-réseaux

- [x] Handshake : `CONNECT_REQUEST` / `CONNECT_RESPONSE`
- [x] États de connexion (Demandée → En validation → Active / Refusée / Bloquée)
- [x] Validation par l'admin (acceptation, refus simple, refus motivé, blocage)
- [x] Accord de Partage Civium (APC) signé cryptographiquement
- [x] Partage d'annuaire membres entre réseaux connectés
- [x] Révocation unilatérale d'une connexion

### Semaines 9–10 — Interface Desktop + CLI

- [ ] Application Tauri (interface React + Tailwind — base)
- [ ] Onboarding : création de compte + réseau en < 5 min
- [ ] CLI : commandes essentielles (`node start`, `network create/connect`, `member invite`)
- [ ] Adressage hybride : direct (IP/URL) + P2P (CID via DHT)
- [ ] Stockage local chiffré SQLCipher

### Semaines 11–12 — Stabilisation

- [ ] Tests de bout en bout (2-3 réseaux pilotes réels)
- [ ] Pairing multi-appareils (QR code + sous-clés dérivées)
- [ ] Mode hors-ligne : cache local + resync CRDT
- [ ] Documentation utilisateur v0.1
- [ ] Publication protocole v0.1

### Critères de succès MVP

- [ ] Deux réseaux indépendants peuvent se connecter sans serveur central
- [ ] Un réseau peut refuser ou bloquer une connexion
- [ ] Les messages sont chiffrés et illisibles hors des nœuds destinataires
- [ ] Le nœud fonctionne hors-ligne et se resynchronise à la reconnexion
- [ ] Un utilisateur non technique peut créer un réseau et inviter un membre en moins de 5 minutes
- [ ] Aucune donnée ne transite par un serveur Civium central

---

## Phase 1 — Gouvernance & Annuaires

- [ ] Votes collectifs et quorum configurable
- [ ] Garde-fou majoritaire (suspension automatique si majorité contre)
- [ ] Délégation de vote
- [ ] Annuaire de réseaux et de membres (réseau dédié)
- [ ] Fédération d'annuaires
- [ ] RRM — Registre des Réseaux Malveillants
- [ ] Profils enfants et contrôle parental

---

## Phase 2 — Services & Intégrations

- [ ] API plugin complète : manifeste, CIL, sandbox WASM, hooks de cycle de vie
- [ ] Plugins préinstallés : Agenda, Documents, Fil d'activité, Notifications
- [ ] Plugin Marketplace (transactions + commission 1 %)
- [ ] Connecteurs SaaS (Google Calendar, Stripe, Notion, Slack…)
- [ ] Webhooks entrants et sortants
- [ ] Serveur MCP (accès IA aux données du réseau)
- [ ] Registre de Services Civium (RSC) — catalogue + publication

---

## Phase 3 — Applications & Écosystème

- [ ] Application mobile iOS / Android (React Native ou Flutter + Rust FFI)
- [ ] Application web PWA (PHP Fat-Free + Alpine.js — nœud Scaleway)
- [ ] Cercle 3 (pair E2E) + récupération sociale
- [ ] Interopérabilité ActivityPub (Mastodon, PeerTube…)
- [ ] Notarisation (OpenTimestamps / Bitcoin) — axe monétisation
- [ ] Badge légal (vérification Sirene / JO) — axe monétisation

---

## Phase 4 — Maturité

- [ ] Programme de certification des plugins (niveaux Minimal / RSC / Certifié)
- [ ] Audit de sécurité externe
- [ ] SDK Civium (intégrateurs tiers)
- [ ] Documentation développeur complète
- [ ] White-label (licence par taille d'organisation)
- [ ] Gouvernance du projet Civium lui-même (association ou fondation)

---

## Décisions techniques en suspens

| Décision | Options | Échéance |
|---|---|---|
| Framework mobile | React Native vs Flutter | Avant Phase 3 |
| Hébergement nœuds bootstrap | À définir (`bootstrap.civium.net`) | Avant fin Phase 0 |

---

*Dernière mise à jour : 2026-05-14*
