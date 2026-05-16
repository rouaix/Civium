# Stack technique Civium

Ce document détaille les choix d'implémentation de Civium. Il est séparé du README (spec du protocole) car ces choix sont susceptibles d'évoluer.

---

## Cœur du protocole — `civium-core`

Le protocole Civium est implémenté en **Rust**, dans un module unique partagé entre toutes les applications.

```
civium-core (Rust)
├── utilisé par  Desktop  (Tauri — natif)
├── utilisé par  Mobile   (React Native — via FFI, civium-ffi)
├── utilisé par  CLI      (binaire natif, civium-cli)
├── utilisé par  SDK      (intégrateurs tiers, civium-sdk)
└── compilé en  WASM     (client web navigateur)
```

Une seule implémentation du protocole — pas de divergence entre plateformes.

---

## Application Desktop

**Stack :** [Tauri](https://tauri.app) (Rust + WebView)

- Exécutable léger (< 10 Mo vs 150 Mo+ pour Electron)
- Interface web **React + Tailwind** pour l'UI (MVP)
- Cœur du protocole en Rust — performance et sécurité mémoire
- Disponible Windows, macOS, Linux depuis une seule base de code

```
┌────────────────────────────────────┐
│  Application Desktop (Tauri)       │
│  ┌──────────────────────────────┐  │
│  │  Interface (WebView)         │  │
│  ├──────────────────────────────┤  │
│  │  Civium Core (Rust)          │  │
│  │  ├── libp2p (transport P2P)  │  │
│  │  ├── CRDT (sync données)     │  │
│  │  ├── Protocole Civium (CP)   │  │
│  │  └── SQLCipher (stockage chiffré) │  │
│  └──────────────────────────────┘  │
└────────────────────────────────────┘
```

---

## Application Mobile

**Stack :** React Native *(décision arrêtée — Phase 4)*

- Base de code partagée iOS / Android
- Module natif Rust pour le protocole (via FFI — `civium-ffi` avec uniffi-rs)
- Stockage local chiffré (SQLCipher — SQLite + clé dérivée)

**Gestion batterie et connectivité :**
- Synchronisation différée hors Wi-Fi (configurable)
- Mode ultra-léger en arrière-plan (notifications uniquement)
- Reconnexion automatique P2P à la reprise de connexion

---

## Application Web

**Stack :** PHP Fat-Free Framework + Alpine.js

Hébergé sur Scaleway (infrastructure existante).

```
Navigateur
  │
  ├── Pages & routing ──────→ PHP Fat-Free Framework
  │   Templates, sessions,     (hébergement Scaleway)
  │   authentification,
  │   proxy API → nœud Civium
  │
  ├── UI dynamique ──────────→ Alpine.js (2 Ko)
  │   Réactivité dans les       s'intègre dans les templates F3
  │   templates PHP,            sans build step
  │   sans SPA complète
  │
  └── Temps-réel ────────────→ Connexion directe navigateur
      WebSocket, WebRTC         ↕ nœud Civium
                                (bypass PHP — F3 fournit
                                 uniquement le token signé)
```

**Pourquoi cette stack :**
- **PHP F3** : framework existant, zéro changement d'infrastructure sur Scaleway
- **Alpine.js** : 2 Ko, s'écrit dans les templates PHP sans étape de compilation
- **Vanilla JS** pour le Service Worker (PWA) et WebSocket/WebRTC
- **Scaleway bas de gamme** : PHP + nginx, empreinte mémoire minimale

**Séparation des responsabilités :**

| Couche | Technologie | Rôle |
|---|---|---|
| Routing & pages | PHP Fat-Free | Rendu templates, sessions, auth |
| API bridge | PHP Fat-Free | Proxy REST vers le nœud Civium, validation tokens |
| UI réactive | Alpine.js | Composants dynamiques dans les templates |
| Temps-réel | Vanilla JS | WebSocket et WebRTC directs vers le nœud |
| Hors-ligne | Service Worker | Cache PWA, fonctionnement sans connexion |

**Flux d'authentification WebSocket :**
```
1. Navigateur → PHP F3 : demande de token signé
2. PHP F3 → Nœud Civium : vérifie la session membre
3. Nœud Civium → PHP F3 : token WebSocket signé (TTL court)
4. PHP F3 → Navigateur : retourne le token
5. Navigateur → Nœud Civium : connexion WebSocket avec token
   (PHP n'est plus dans la boucle)
```

---

## Interface CLI

**Stack :** Rust (binaire natif)

```bash
civium node start                        # démarre le nœud
civium network create --name "mon-asso"  # crée un réseau
civium network connect --cid civium:...  # connecte à un réseau
civium member invite --email ...         # invite un membre
civium service install marketplace       # installe un service
civium audit log --last 7d               # journal des 7 derniers jours
civium backup export --encrypted         # sauvegarde chiffrée
```

---

## Infrastructure

| Composant | Hébergement | Notes |
|---|---|---|
| Nœud web Civium (v1) | Scaleway | Premier nœud bootstrap + annuaire racine |
| Application web | Scaleway | PHP + nginx, bas de gamme |
| Nœuds bootstrap officiels | À définir | `bootstrap.civium.net`, `bootstrap2.civium.net` |
