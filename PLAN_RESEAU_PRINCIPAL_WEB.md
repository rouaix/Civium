# Plan — Réseau Principal Civium (web)

Réseau Civium hébergé côté serveur, accessible depuis le navigateur.  
PHP + MySQL côté serveur. Clé privée membre dans le navigateur (IndexedDB).  
Accès : admin uniquement + membres invités par l'admin.

---

## État d'avancement

| # | Module | État |
|---|--------|------|
| 1 | Schéma BDD (migrations) | ✅ Fait |
| 2 | Identité web (keypair navigateur, lien email↔CID) | ✅ Fait |
| 3 | Invitations (admin envoie, user accepte) | ✅ Fait (join page) |
| 4 | Interface membre — tableau de bord web | ✅ Fait |
| 5 | Messagerie réseau | ✅ Fait |
| 6 | Messagerie directe | ✅ Fait (API) |
| 7 | Gouvernance (propositions + votes) | ✅ Fait |
| 8 | Annuaire (lecture + publication) | ✅ Fait (API) |
| 9 | Fil d'activité + notifications | ✅ Fait |
| 10 | Interface admin — gestion du réseau | ⬜ À faire |

---

## 1. Schéma BDD

Nouvelles tables dans `website/src/migrations/005_reseau_principal.sql` :

```
web_members          — membres du réseau principal
web_invitations      — invitations envoyées par l'admin
web_messages         — messages réseau (public au réseau)
web_direct_messages  — messages directs entre membres
web_proposals        — propositions de gouvernance
web_votes            — votes sur les propositions
web_directory_entries — entrées d'annuaire publiées
web_activity         — fil d'activité
web_notifications    — notifications par membre
```

Champs clés `web_members` :
- `id` — UUID
- `email` — email (lié au magic link)
- `cid_short` / `cid_full` — identité Ed25519 du membre (déclarée depuis le navigateur)
- `display_name`
- `role` — `admin` | `member`
- `circle` — 0 à 3
- `status` — `active` | `suspended`
- `invited_by` — CID de l'admin
- `joined_at`

---

## 2. Identité web

Flux première connexion :
1. Magic link → session PHP créée
2. Page "Configurer mon identité" :
   - Option A : coller son `secret_b58` depuis l'appli desktop (même CID que desktop)
   - Option B : générer une nouvelle paire de clés dans le navigateur
3. La clé privée est chiffrée avec un PIN et stockée dans IndexedDB
4. Le CID public est envoyé au serveur → lié à l'email dans `web_members`

Connexions suivantes : magic link → saisie PIN → clé déchiffrée depuis IndexedDB.

---

## 3. Invitations

- Admin envoie une invitation par email depuis `/admin/network/members`
- Crée une ligne dans `web_invitations` (token, email, expires_at)
- L'email contient un lien `/civium/join?token=…`
- Le destinataire clique → magic link automatique + flux identité (étape 2)
- Une fois le CID déclaré → ajouté dans `web_members` avec `role=member`

---

## 4. Interface membre — tableau de bord web

URL : `/civium/network`  
Auth requise (magic link + PIN).

Sections :
- **Fil d'activité** — derniers événements du réseau
- **Messages** — messages réseau + messages directs
- **Gouvernance** — propositions en cours, voter
- **Membres** — liste des membres (selon cercle)
- **Annuaire** — parcourir les entrées publiées

---

## 5. Messagerie réseau

- `POST /civium/api/message` — envoyer un message
- `GET /civium/api/messages?since=…` — récupérer les messages (polling ou SSE)
- Stocké dans `web_messages` (author_cid, body, sent_at)
- Chiffrement : groupe (clé réseau partagée) — à définir en phase 2

---

## 6. Messagerie directe

- `POST /civium/api/dm` — envoyer un message direct
- Stocké dans `web_direct_messages` (from_cid, to_cid, body, sent_at)
- Chiffrement E2E à définir en phase 2

---

## 7. Gouvernance

- `POST /civium/api/proposal` — créer une proposition
- `POST /civium/api/vote` — voter
- `GET /civium/api/proposals` — liste + résultats
- Stocké dans `web_proposals` + `web_votes`
- Règles : quorum configurable, fenêtre de vote, options libres

---

## 8. Annuaire

- `GET /civium/api/directory` — liste des entrées publiques
- `POST /civium/api/directory` — publier une entrée (membres autorisés)
- Entrées : réseaux, membres, services
- Recherche par nom ou tags

---

## 9. Fil d'activité + notifications

- Chaque action (message, vote, invitation acceptée…) crée une ligne dans `web_activity`
- Les notifications sont générées pour les membres concernés dans `web_notifications`
- `GET /civium/api/notifications` — non lues
- `POST /civium/api/notifications/read` — marquer comme lue

---

## 10. Interface admin

URL : `/admin/network`  
Auth admin requise.

Actions :
- Inviter un membre (email)
- Voir la liste des membres (actifs, invitations en attente)
- Suspendre / réactiver un membre
- Gérer les propositions de gouvernance
- Configurer les paramètres du réseau (nom, règles de vote, quorum)

---

## Routes PHP à ajouter dans `config.ini`

```ini
; Réseau principal web — interface membre
GET /civium/network=NetworkController->dashboard
GET /civium/join=NetworkController->joinPage
POST /civium/api/message=NetworkController->postMessage
GET /civium/api/messages=NetworkController->getMessages
POST /civium/api/dm=NetworkController->postDm
POST /civium/api/proposal=NetworkController->createProposal
POST /civium/api/vote=NetworkController->castVote
GET /civium/api/proposals=NetworkController->getProposals
GET /civium/api/directory=NetworkController->getDirectory
POST /civium/api/directory=NetworkController->publishEntry
GET /civium/api/notifications=NetworkController->getNotifications
POST /civium/api/notifications/read=NetworkController->markRead
GET /civium/api/members=NetworkController->getMembers

; Admin réseau
GET /admin/network=AdminNetworkController->index
POST /admin/network/invite=AdminNetworkController->invite
POST /admin/network/member/suspend=AdminNetworkController->suspend
```

---

## Fichiers à créer

```
website/src/migrations/005_reseau_principal.sql
website/src/app/modules/civium/controllers/NetworkController.php
website/src/app/modules/civium/controllers/AdminNetworkController.php
website/src/app/modules/civium/views/network-dashboard.html
website/src/app/modules/civium/views/network-join.html
website/src/app/modules/civium/views/admin-network.html
```

---

## Ordre d'implémentation recommandé

1. Migration BDD (005)
2. Invitations + flux d'inscription (modules 3 + 2)
3. Interface membre de base + messagerie réseau (modules 4 + 5)
4. Gouvernance (module 7)
5. Annuaire (module 8)
6. Messagerie directe (module 6)
7. Fil d'activité + notifications (module 9)
8. Interface admin complète (module 10)
