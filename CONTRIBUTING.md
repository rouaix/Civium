# Contributing to Civium

## Setup développement — Windows

Civium utilise SQLCipher (chiffrement de la base SQLite) qui nécessite **cmake** et **Perl** sur la machine de build.

```powershell
# Installe automatiquement toutes les dépendances (Rust, cmake, Strawberry Perl, Node)
.\scripts\setup-windows-dev.ps1
```

Puis redémarrer le terminal pour que les PATH soient pris en compte.

## Setup développement — Linux / macOS

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Linux — dépendances système
sudo apt install cmake perl libssl-dev pkg-config build-essential

# macOS
brew install cmake perl openssl@3
```

## Lancer l'app en développement

```bash
cd desktop/civium-tauri
npm install
npm run tauri dev
```

## Compiler uniquement civium-core

```bash
cd desktop
cargo build -p civium-core
```

## NAT traversal — état actuel et workaround

### Pourquoi deux nœuds derrière NAT ne peuvent pas se connecter directement

Sans IP publique ou port ouvert, deux nœuds en NAT symétrique ne peuvent pas établir de connexion TCP/QUIC directe. libp2p résout cela via le **Circuit Relay** (nœud tiers qui relaie le trafic) et **AutoNAT** (sonde le type de NAT).

### Ce qui est implémenté

- **AutoNAT** (`libp2p::autonat`) : activé dans `CiviumBehaviour`. Permet à un nœud de détecter s'il est derrière NAT et quel type.
- **mDNS** : découverte locale sur le réseau LAN (fonctionne sans internet).
- **Kademlia DHT** : bootstrapping P2P via les nœuds connus.

### Ce qui est différé

- **Circuit Relay** (`libp2p::relay`) : incompatible avec `with_websocket()` dans le builder libp2p 0.55 — les deux ne peuvent pas cohabiter dans la même chaîne de build. Voir le commentaire dans `civium-core/src/node/behaviour.rs`.

### Workaround actuel pour les nœuds derrière NAT stricte

Utiliser **Cloudflare Tunnel** pour exposer le nœud Civium avec une IP publique :

```bash
# Installer cloudflared
# https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/

# Exposer le port libp2p (défaut: 4001)
cloudflared tunnel --url tcp://localhost:4001

# Récupérer l'adresse multiaddr publique et la renseigner dans les paramètres
# "Adresse externe" du Dashboard → le nœud l'annoncera à ses pairs
```

Puis, dans le Dashboard → Paramètres du nœud → "Adresse externe", renseigner l'adresse `multiaddr` Cloudflare (ex: `/dns4/xxx.trycloudflare.com/tcp/443/tls/ws`).

### Feuille de route relay

Quand libp2p résoudra la compatibilité relay + WebSocket, ou quand nous migrerons vers libp2p 0.56+, activer le relay dans `civium-core/src/node/mod.rs` en ajoutant `.with_relay_client(...)` dans le SwarmBuilder.

## Migrations de schéma SQLite

Chaque changement de schéma dans `civium-tauri/src-tauri/src/store.rs` doit être ajouté comme une nouvelle constante `MIGRATION_NNN` et enregistré dans le tableau `MIGRATIONS`. Ne jamais modifier une migration déjà appliquée.

## Tests

```bash
cd desktop
cargo test
```

## Conventions de commit

```
feat(scope): description
fix(scope): description
chore(scope): description
docs(scope): description
```

Exemples : `feat(desktop): ajout multi-compte`, `fix(hub): correction signature agenda`.
