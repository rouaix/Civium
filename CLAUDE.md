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
| Protocol core | Rust (`civium-core`) — shared across Desktop, Mobile, CLI |
| Desktop app | Tauri (Rust + WebView) |
| Mobile app | React Native or Flutter + Rust FFI |
| CLI | Rust (native binary) |
| Web app | PHP Fat-Free Framework + Alpine.js (proxy to a remote Civium node) |
| Transport | libp2p (DHT Kademlia, Noise Protocol, QUIC/TCP/WebRTC) |
| Data sync | CRDT (group-key scope) + last-write-wins (E2E scope) |
| Federation | ActivityPub (interop with Mastodon, PeerTube, etc.) |
| Plugin runtime | WASM sandbox |

## Repository structure

```
civium/
  desktop/          ← Rust workspace (app de bureau en cours de développement)
    Cargo.toml      ← workspace racine
    civium-core/    ← bibliothèque partagée (identité, P2P, gouvernance…)
    civium-cli/     ← outil en ligne de commande
    civium-tauri/   ← application Tauri (desktop GUI)
  website/          ← site web (PHP F3 + Alpine.js)
  README.md         ← spécification du protocole
  ROADMAP.md        ← plan de développement
```
