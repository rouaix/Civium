# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project status

Civium is currently in the **specification phase**. The repository contains the protocol design document (`README.md`) and a logo (`civium.png`). No implementation exists yet — this CLAUDE.md will evolve as code is added.

## What Civium is

Civium is a **sovereign network-of-networks protocol**. Any group (family, company, association, neighborhood) can create a private Civium network, govern it by its own rules, and connect it to other Civium networks. It is explicitly **not** a centralized social network — each node is sovereign, and inter-node connections are explicit and collectively governed.

## Core concepts

### Member identity
A member's primary identity is their **CID** — derived from their Ed25519 public key, globally unique by cryptographic guarantee (no central registry needed). The human-readable display name is chosen freely and is only unique within each network. Cross-network search uses CID; human search within a directory uses display name.

### Trust circles (Cercles de confiance)
Identity and access are revealed progressively based on the relationship level. Each member assigns each relationship to one of four circles:

| Circle | Name | Access granted |
|---|---|---|
| **0** | Annuaire | Name, existence in the network |
| **1** | Connaissance | Partial profile, basic messaging |
| **2** | Confiance | Full profile, content sharing, services |
| **3** | Intime | Full access, sensitive data, history |

Trust is **asymmetric by default**. The 0→1 transition on first interaction is the sole automatic exception (symmetric). All subsequent circles are asymmetric.

### Encryption model
Two distinct layers — do not conflate them:
- **Group key** (circles 0-2): shared among network members, enables CRDT merging after client-side decryption. Nobody outside the network can read. Admins can read.
- **Pair key / true E2E** (circle 3, private messages): only the recipient can decrypt. No CRDT — conflicts resolved by last-write-wins on reconnection.

### Network connections
Every connection is **contractualized** in an **Accord de Partage Civium (APC)** — a cryptographically signed document listing exactly what each side exposes and at what access level. The **CIL (Civium Integration Layer)** enforces the APC on every request, including MCP requests. **MCP = transport, APC = contract.**

### Plugins
Everything in Civium — including messaging, calendar, and governance — is a plugin. "Everything is a plugin" applies to the API surface and sandboxing (WASM), not to uninstallability. Two plugins are **system plugins** and cannot be removed:
- **Gouvernance** — required for any collective decision
- **CIL** — required for all data access and inter-network enforcement

All other pre-installed plugins (Messagerie, Agenda, Annuaire, Documents, Fil d'activité, Notifications) can be disabled.

### Governance
Each network defines its own model freely: autocratic, administrative, representative, participatory, consensual, hybrid. Key mechanisms: configurable voting, quorum, vote delegation, anonymous/nominative ballots, immutable audit log.

**Majoritarian safeguard**: when an admin takes a unilateral structural/strategic decision and a majority of members disagrees within a configurable window, the decision is suspended and a collective vote is triggered automatically.

### Directories (Annuaires) and RRM
A directory is a specialized Civium network whose function is to catalog and make discoverable other networks, members, or services. Directories can federate.

The **Registre des Réseaux Malveillants (RRM)** is a specialized directory type for listing networks with proven malicious behavior. Multiple RRMs can coexist; each network chooses which RRMs to trust. The **RRM Global Civium** is itself an ordinary Civium network — not controlled by the Civium team, governed by its own community.

### Three-level architecture
```
Individual node  →  Civium network  →  Civium directory
(member profile)    (group space)       (registry of networks/members/services)
```

## Planned tech stack

| Layer | Technology |
|---|---|
| Protocol core | Rust (`civium-core`) — shared across Desktop, Mobile, CLI, Web |
| Desktop app | Tauri (Rust + WebView) |
| Mobile app | React Native or Flutter + Rust FFI |
| CLI | Rust (native binary) |
| Web app | `civium-core` compilé en **WebAssembly** — tourne dans le navigateur, P2P direct via WebSocket + WebRTC |
| Web shell | PHP Fat-Free Framework + Alpine.js — sert les fichiers WASM, gère auth (magic link) et le Registre Central |
| Transport desktop | libp2p : TCP + QUIC + **WebSocket** (le WebSocket permet aux clients web de se connecter) |
| Transport web | libp2p : WebSocket + WebRTC (pas de TCP/QUIC dans un navigateur) |
| Data sync | CRDT (group-key scope) + last-write-wins (E2E scope) |
| Federation | ActivityPub (interop avec Mastodon, PeerTube…) |
| Plugin runtime | WASM sandbox |
| Central registry | PHP + MySQL — Registre Central Civium (RCC) |

## Registre Central Civium (RCC)

URL de base (codée en dur dans tous les clients) : **`https://www.rouaix.com/civium`**

### Rôle

Le RCC n'est **pas** une autorité. Il est un registre légal et un canal d'alerte :

- Tout réseau Civium **doit** s'enregistrer (conformité légale)
- Le RCC stocke les informations minimales requises pour répondre aux autorités
- Il peut diffuser des **alertes fraude** à tous les réseaux enregistrés (email + push P2P)
- Il n'a aucun pouvoir de refus, suspension ou modification d'un réseau

### Données stockées par réseau

| Champ | Description |
|---|---|
| `network_cid` | CID du réseau (clé primaire) |
| `network_name` | Nom public du réseau |
| `admin_cid` | CID du membre fondateur |
| `admin_pubkey` | Clé publique Ed25519 du fondateur (pour vérifier les signatures) |
| `admin_email` | Email de contact **obligatoire** |
| `ip_address` | IP au moment de l'enregistrement |
| `registered_at` | Horodatage UTC |
| `signature` | Signature Ed25519 de l'ensemble des champs — prouve que l'expéditeur contrôle le réseau |

### Flux d'enregistrement

```
App desktop : création réseau
  → HTTP POST https://www.rouaix.com/civium/api/register
    Body (JSON) : { network_cid, network_name, admin_cid, admin_pubkey,
                    admin_email, ip, registered_at, signature }
  → PHP vérifie la signature, stocke en MySQL, répond 201
  → Si échec réseau → retry exponentiel en arrière-plan :
      5 s → 30 s → 5 min → 30 min → 1 h → toutes les heures
  → Statut d'enregistrement visible dans le Dashboard Tauri
```

### Alertes fraude

Le RCC peut émettre une alerte avec : type, description, réseaux concernés (CIDs), date.

Diffusion :
1. **Email** → tous les `admin_email` enregistrés
2. **Push P2P** → message signé par la clé RCC, diffusé via DHT à tous les nœuds actifs

Chaque nœud Civium affiche l'alerte si elle est signée par la clé publique RCC connue (codée en dur).

## Client web Civium

### Architecture

```
Navigateur
  └── civium-core.wasm  ← Rust compilé en WASM (wasm-pack)
        ├── Identité Ed25519 de l'utilisateur (déchiffrée localement)
        ├── P2P libp2p : WebSocket + WebRTC (DHT, Noise, CRDT)
        ├── Toutes les fonctionnalités : messagerie, gouvernance,
        │   agenda, annuaire, plugins, notifications…
        └── Sync CRDT natif avec les nœuds desktop / mobile

PHP F3 (https://www.rouaix.com/civium)
  ├── Sert les fichiers HTML / JS / WASM
  ├── API magic link (génération token, validation, session)
  ├── API RCC (register, alert)
  └── MySQL ← tokens magic link + registre réseaux
```

PHP ne voit **jamais** la clé privée de l'utilisateur. Toute la cryptographie se passe dans le navigateur.

### Connexion des nœuds desktop aux clients web

Les nœuds desktop (Tauri / CLI) **doivent écouter en WebSocket** en plus de TCP/QUIC, pour que les clients web puissent s'y connecter via libp2p. Le transport WebSocket est activé par défaut dans `NodeConfig`.

### Authentification web (magic link + clé locale)

```
1. Utilisateur saisit son email sur le client web
2. PHP envoie un lien à usage unique (token SHA-256, expire 15 min)
3. Clic sur le lien → session PHP créée (preuve de possession de l'email)
4. Première connexion sur cet appareil :
   → L'utilisateur saisit son secret_b58
     (affiché dans l'app desktop : Identité → "Clé secrète")
   → Le WASM chiffre la clé avec un PIN saisi par l'utilisateur
   → Clé chiffrée stockée dans IndexedDB du navigateur (jamais envoyée au serveur)
5. Connexions suivantes : magic link → saisie PIN → clé déchiffrée depuis IndexedDB
```

Lien email ↔ CID stocké sur le RCC (déclaratif, pour les alertes push).

## Repository structure

```
civium/
  desktop/                  ← Rust workspace
    Cargo.toml              ← workspace racine
    civium-core/            ← bibliothèque partagée (identité, P2P, gouvernance…)
    civium-cli/             ← outil en ligne de commande
    civium-tauri/           ← application Tauri (desktop GUI)
  website/                  ← PHP F3 + Alpine.js
    src/
      controllers/
        ApiController.php   ← /api/register, /api/alert (RCC)
        AuthController.php  ← magic link (generate, validate, session)
        AppController.php   ← sert le client web WASM
      models/
        Network.php         ← registre RCC (MySQL)
        MagicLink.php       ← tokens temporaires
        Alert.php           ← alertes fraude
      www/
        civium/             ← point d'entrée client web
        wasm/               ← civium-core.wasm + bindings JS (wasm-pack output)
  README.md                 ← spécification du protocole
  ROADMAP.md                ← plan de développement
  CLAUDE.md                 ← ce fichier

## Décisions architecturales figées

- L'URL du RCC est `https://www.rouaix.com/civium` — codée en dur dans `civium-core`. Toute modification nécessite une mise à jour du code et un nouveau build.
- L'enregistrement RCC est **obligatoire** et **non contournable** depuis les apps officielles.
- La clé privée ne quitte jamais le périphérique de l'utilisateur (desktop, mobile, ou IndexedDB navigateur). Le serveur PHP ne la voit jamais.
- Les nœuds desktop exposent WebSocket en plus de TCP/QUIC pour permettre la connexion des clients web.
```
