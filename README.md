# CIVIUM
### Cadre d'Interconnexion et de Validation des Intentions et des Usages des Membres

> **v0.1 — Implémentation de référence disponible.**
> Le protocole décrit dans ce document est implémenté dans le workspace Rust [`desktop/`](desktop/).
> Phases 0 à 4 complètes. Voir [ROADMAP.md](ROADMAP.md) pour l'état détaillé de chaque composant.

---

## Table des matières

- [Vision](#vision)
- [Philosophie](#philosophie)
- [Concept central : le Réseau Civium](#concept-central--le-réseau-civium)
- [Architecture : Réseau de réseaux](#architecture--réseau-de-réseaux)
- [Cercles de confiance entre membres](#cercles-de-confiance-entre-membres)
- [Gouvernance des réseaux](#gouvernance-des-réseaux)
- [Annuaires Civium](#annuaires-civium)
- [Sécurité & vie privée](#sécurité--vie-privée)
  - [Chiffrement](#chiffrement)
  - [Gestion des clés](#gestion-des-clés)
  - [Authentification](#authentification)
  - [Vie privée](#vie-privée)
  - [Audit et journalisation](#audit-et-journalisation)
  - [Sécurité des plugins](#sécurité-des-plugins)
  - [Sécurité du réseau P2P](#sécurité-du-réseau-p2p)
  - [Protection contre les réseaux Civium malveillants](#protection-contre-les-réseaux-civium-malveillants)
  - [Coopération judiciaire et réseaux illicites](#coopération-judiciaire-et-réseaux-illicites)
  - [Processus de sécurité du projet](#processus-de-sécurité-du-projet)
- [Cas d'usage](#cas-dusage)
- [Architecture technique](#architecture-technique)
- [Pile protocolaire](#pile-protocolaire)
- [Cycle de vie d'une connexion inter-réseaux](#cycle-de-vie-dune-connexion-inter-réseaux)
- [Services Civium](#services-civium)
- [Identité des membres](#identité-des-membres)
  - [Profils enfants et contrôle parental](#profils-enfants-et-contrôle-parental)
- [Fonctionnalités transversales](#fonctionnalités-transversales)
- [Applications](#applications)
- [Ce que Civium n'est pas](#ce-que-civium-nest-pas)
- [MVP — Produit Minimum Viable](#mvp--produit-minimum-viable)
- [Feuille de route](#feuille-de-route)
- [Glossaire](#glossaire)

---

## Vision

Civium est un **protocole et une plateforme de mise en réseau souverain**. Il permet à tout groupe d'individus — famille, entreprise, association, quartier, école, communauté — de créer son propre réseau privé, de le gérer selon ses règles, et de le connecter librement à d'autres réseaux Civium.

Civium n'est pas un réseau social centralisé. C'est un **réseau de réseaux**, décentralisé, où chaque nœud est souverain et où les connexions entre nœuds sont explicites, choisies et gouvernées collectivement.

---

## Philosophie

### Souveraineté des données
Chaque réseau héberge ses données sur ses propres nœuds. Civium ne centralise rien. Les données partagées entre réseaux transitent directement de nœud à nœud, sans passer par un tiers.

### Cercles de confiance progressifs
L'identité et l'accès se révèlent progressivement selon la relation entre membres et entre réseaux :

- **Cercle 0 — Annuaire** : présence minimale, découverte possible
- **Cercle 1 — Connaissance** : interaction légère, profil partiel
- **Cercle 2 — Confiance** : identité vérifiée par des pairs, accès enrichi
- **Cercle 3 — Intime / Interne** : accès complet, partage profond

Chaque membre définit pour chaque relation le cercle auquel elle appartient. Chaque réseau définit pour chaque réseau connecté le niveau d'accès accordé.

### Validation par les pairs
Les connexions, les membres et les contenus peuvent être validés par la communauté elle-même, selon les règles de gouvernance du réseau. Pas d'algorithme opaque, pas de modération centralisée.

---

## Concept central : le Réseau Civium

Un **Réseau Civium** est l'unité de base du protocole. Il peut être instancié par :

- Un **individu** (nœud personnel)
- Une **famille** (espace privé partagé)
- Une **entreprise ou équipe** (espace professionnel)
- Une **association ou communauté** (espace collectif)
- Un **quartier, une école, une institution** (espace territorial ou thématique)

Chaque réseau est souverain : il possède ses membres, ses données, ses règles de fonctionnement et ses politiques de connexion avec l'extérieur.

---

## Architecture : Réseau de réseaux

```
[Réseau Famille Martin] ←→ [Réseau Asso Vélo Urbain]
          ↕                          ↕
[Réseau Entreprise X]  ←→ [Annuaire Civium Régional]
          ↕                          ↕
   [Nœud individuel]       [Réseau Quartier Sud]
```

### Trois niveaux d'existence

| Niveau | Description |
|---|---|
| **Nœud individuel** | Un membre, son profil, ses données personnelles |
| **Réseau Civium** | Un groupe de membres partageant un espace commun |
| **Annuaire Civium** | Un registre de réseaux et/ou de membres, public ou semi-public |

### Connexion entre réseaux

Les réseaux peuvent se connecter entre eux de deux manières :

- **Connexion totale** : les membres des deux réseaux peuvent interagir librement dans un espace partagé défini
- **Connexion partielle** : un réseau expose seulement certaines informations ou services à un autre réseau, avec des permissions granulaires (ex : un réseau professionnel partage son annuaire de compétences mais pas ses discussions internes)

Chaque connexion est **contractualisée** dans le protocole : les deux réseaux définissent explicitement ce qui est partagé, dans quelle direction, et sous quelles conditions.

---

## Cercles de confiance entre membres

### Principe

La confiance entre membres ne se décrète pas — elle se construit progressivement. Chaque membre gère ses relations via un système de **cercles de confiance** : plus un membre est proche, plus il accède à des informations et des interactions enrichies.

La confiance est **asymétrique par défaut** : je peux te faire confiance au niveau 2 pendant que tu me fais confiance au niveau 1. Chacun est libre de placer l'autre dans le cercle qu'il juge approprié, indépendamment. La première interaction (cercle 0 → 1) est l'unique exception : elle est symétrique automatiquement. Tous les cercles suivants sont asymétriques par défaut.

### Les quatre cercles

```
        ┌─────────────────────────────────────────┐
        │  Cercle 3 — Intime                      │
        │   ┌─────────────────────────────────┐   │
        │   │  Cercle 2 — Confiance           │   │
        │   │   ┌─────────────────────────┐   │   │
        │   │   │  Cercle 1 — Connaissance│   │   │
        │   │   │   ┌─────────────────┐   │   │   │
        │   │   │   │ Cercle 0        │   │   │   │
        │   │   │   │ Annuaire        │   │   │   │
        │   │   │   └─────────────────┘   │   │   │
        │   │   └─────────────────────────┘   │   │
        │   └─────────────────────────────────┘   │
        └─────────────────────────────────────────┘
```

| Cercle | Nom | Qui | Accès accordé |
|---|---|---|---|
| **0** | Annuaire | Tout membre du réseau | Nom affiché, existence dans le réseau |
| **1** | Connaissance | Membres avec qui on a interagi | Profil partiel, messagerie basique |
| **2** | Confiance | Membres explicitement validés | Profil complet, partage de contenu, services |
| **3** | Intime | Membres de confiance profonde | Accès total au profil, partage privé, données sensibles |

### Progression dans les cercles

#### Cercle 0 → Cercle 1 : première interaction

Le passage au cercle 1 est déclenché par une **interaction explicite** : envoi d'un message, participation à un même événement, invitation acceptée. Il est automatique ou manuel selon les préférences du membre.

```
daniel envoie un message à sophie
         │
         ▼
sophie passe en Cercle 1 pour daniel
daniel passe en Cercle 1 pour sophie  ← symétrique automatiquement
```

#### Cercle 1 → Cercle 2 : confiance explicite

Le passage au cercle 2 est un **acte délibéré** : un membre décide activement de faire confiance à un autre. Il peut être :

- **Unilatéral** : daniel place sophie en cercle 2 sans que sophie en soit informée (elle accède à plus de données de daniel, pas l'inverse)
- **Mutuel** : les deux membres se placent mutuellement en cercle 2 — débloque des fonctionnalités de partage bidirectionnel

```
daniel  ──[cercle 2]──▶  sophie   (daniel voit le profil complet de sophie)
daniel  ◀──[cercle 1]──  sophie   (sophie voit le profil partiel de daniel)
```

#### Cercle 2 → Cercle 3 : validation par les pairs (optionnel)

Le réseau peut exiger qu'un passage au cercle 3 soit **validé par d'autres membres** — une ou plusieurs personnes du réseau qui attestent de la relation. Ce mécanisme de **cautionnement** renforce la confiance collective.

```
daniel veut placer sophie en cercle 3
         │
         ▼
Demande de cautionnement envoyée à 2 membres du réseau
         │
    Marie ✓   Pierre ✓
         │
         ▼
sophie passe en cercle 3 pour daniel
```

Le cautionnement est optionnel — chaque réseau choisit s'il l'active et pour quel cercle.

### Ce que chaque cercle débloque

| Fonctionnalité | Cercle 0 | Cercle 1 | Cercle 2 | Cercle 3 |
|---|:---:|:---:|:---:|:---:|
| Nom affiché | ✓ | ✓ | ✓ | ✓ |
| Photo de profil | — | ✓ | ✓ | ✓ |
| Biographie courte | — | ✓ | ✓ | ✓ |
| Profil complet | — | — | ✓ | ✓ |
| Messagerie | — | basique | complète | complète |
| Partage de fichiers | — | — | ✓ | ✓ |
| Données sensibles | — | — | — | ✓ |
| Agenda personnel | — | — | partiel | complet |
| Contacts en commun | — | — | ✓ | ✓ |
| Historique partagé | — | — | — | ✓ |

Le contenu exact de chaque cercle est **configurable par le membre** — ce tableau définit les valeurs par défaut.

### Asymétrie et réciprocité

```
Vue de daniel :         Vue de sophie :
sophie → cercle 2       daniel → cercle 1

Ce que daniel voit      Ce que sophie voit
de sophie :             de daniel :
  profil complet          profil partiel
  agenda partiel          nom + photo
  messagerie complète     messagerie basique
```

La réciprocité complète (cercle 2 mutuel) débloque des fonctionnalités supplémentaires : agenda croisé, espaces de travail partagés, co-administration de contenus.

### Confiance inter-réseaux

Un membre peut **porter sa confiance d'un réseau à un autre**. Si daniel et sophie se font confiance au cercle 2 dans `asso-velo`, daniel peut choisir de reconnaître sophie au cercle 2 d'emblée dans `equipe-design`, sans repartir de zéro.

```
a3f9@b4e2 (daniel, asso-velo)  ──[cercle 2]──  c7d3@b4e2 (sophie, asso-velo)
         │                                              │
         │  daniel rejoint equipe-design                │
         ▼                                              │
a3f9@e91f (daniel, equipe-design)                      │
  └── reconnaît c7d3@e91f (sophie, equipe-design)  ────┘
      au cercle 2 directement (si sophie est aussi dans ce réseau)
```

Cette reconnaissance inter-réseaux est **toujours un choix explicite** du membre — la confiance ne se transfère jamais automatiquement.

### Révocation et exclusion

L'exclusion d'un membre est **graduée selon qui décide et le niveau de consensus atteint**.

#### Exclusion personnelle — par un membre

Un membre peut exclure un autre membre pour lui-même uniquement. L'exclusion n'est effective que pour celui qui la prononce — les autres membres du réseau ne sont pas affectés.

```
daniel exclut marc
  │
  ▼
Pour daniel : marc n'existe plus (invisible, aucune interaction possible)
Pour les autres membres : marc est toujours présent normalement
```

| Action | Effet |
|---|---|
| **Rétrogradation** | Passage à un cercle inférieur, accès réduits |
| **Retour au cercle 0** | Relation minimale, accès révoqués |
| **Blocage personnel** | Marc invisible pour daniel, aucune interaction possible |

#### Exclusion par l'admin

L'administrateur du réseau peut exclure totalement un membre du réseau. L'exclusion est immédiate et effective pour l'ensemble des membres.

```
Admin exclut marc
  │
  ▼
marc est retiré du réseau pour tous les membres
ses accès sont révoqués, ses contributions restent (selon politique du réseau)
```

#### Exclusion collective — partielle ou totale

Un groupe de membres peut initier une **procédure d'exclusion collective**. L'effet dépend du niveau de consensus atteint :

```
Procédure d'exclusion de marc
│
├── Minorité vote pour l'exclusion
│     └── Exclusion partielle : marc est exclu uniquement
│         pour les membres ayant voté
│
└── Majorité vote pour l'exclusion
      └── Exclusion totale : marc est exclu du réseau
          pour l'ensemble des membres
```

**Seuil de majorité** : défini par la gouvernance du réseau (ex : 50%+1, deux tiers, etc.).

**Pendant la procédure** : marc reste membre à part entière jusqu'au résultat du vote. Aucun accès n'est restreint pendant la délibération.

**Notification** : le membre concerné peut être notifié ou non de la procédure, selon la politique du réseau. Le résultat lui est toujours communiqué.

#### Synthèse

| Qui décide | Consensus | Effet de l'exclusion |
|---|---|---|
| Un membre | — | Exclusion personnelle — invisible uniquement pour ce membre |
| Un groupe | Minorité | Exclusion partielle — invisible pour les membres du groupe |
| Un groupe | Majorité | Exclusion totale du réseau |
| L'admin | — | Exclusion totale du réseau |

La révocation personnelle (cercle, blocage) est silencieuse par défaut. L'exclusion collective ou par l'admin est toujours notifiée au membre exclu.

---

## Gouvernance des réseaux

### Principe

Chaque réseau Civium définit librement son modèle de gouvernance. Il n'existe pas de modèle imposé — seulement un **cadre commun** de mécanismes disponibles que chaque réseau configure selon sa culture et ses besoins.

### Rôles

| Rôle | Description |
|---|---|
| **Fondateur** | Membre ayant créé le réseau. Dispose de tous les droits à l'origine, peut les déléguer ou les distribuer |
| **Admin** | Membre disposant des droits de gestion du réseau. Un réseau peut avoir plusieurs admins |
| **Modérateur** | Membre pouvant gérer les contenus et les membres, sans accès aux paramètres du réseau |
| **Membre** | Participant actif. Peut voter selon les règles définies |
| **Observateur** | Accès en lecture seule, sans droit de vote ni de contribution |
| **Collectif de gouvernance** | Groupe désigné de membres habilités à co-décider sur des sujets définis |

Les rôles sont **cumulables et configurables** — un réseau peut créer ses propres rôles avec des permissions sur mesure.

### Modèles de gouvernance

Chaque réseau choisit son modèle, applicable globalement ou par type de décision :

| Modèle | Fonctionnement |
|---|---|
| **Autocratique** | L'admin décide seul de tout |
| **Administratif** | L'admin décide, les membres peuvent proposer |
| **Représentatif** | Un collectif élu décide au nom des membres |
| **Participatif** | Les membres votent sur les décisions importantes |
| **Consensuel** | Les décisions requièrent l'absence d'opposition formelle |
| **Hybride** | Modèle différent selon le type de décision |

Le modèle hybride est le plus courant : décisions courantes par l'admin, décisions stratégiques par vote collectif.

### Types de décisions et gouvernance associée

| Décision | Exemple | Qui peut décider |
|---|---|---|
| **Opérationnelle** | Épingler un message, créer un événement | Membre, modérateur, admin |
| **Modération** | Supprimer un contenu, avertir un membre | Modérateur, admin |
| **Structurelle** | Installer un service, modifier les règles | Admin ou vote collectif |
| **Stratégique** | Connexion inter-réseaux, modification de gouvernance | Vote collectif ou admin |
| **Exclusion** | Exclure un membre | Voir modèle d'exclusion |
| **Dissolution** | Fermer le réseau | Fondateur ou supermajorité |

Chaque réseau configure librement quel niveau de décision requiert quel niveau de gouvernance.

### Cycle de vie d'une décision collective

```
[Proposition]
     │
     ▼
[Délibération]  ← période de débat, durée configurable
     │
     ▼
[Vote]  ← fenêtre configurable (ex : 24h, 7 jours)
     │       votes asynchrones — chaque membre vote quand il peut
     │       CRDT collecte les votes (dédupliqués par CID membre)
     │
     ├── Deadline atteinte ?
     │     ├── Non → vote encore ouvert
     │     └── Oui → dépouillement de tous les votes reçus
     │
     ├── Quorum atteint ?
     │     ├── Non → Décision caduque ou prolongation
     │     └── Oui → Résultat calculé
     │
     ├── Résultat
     │     ├── Adopté  → Application automatique ou manuelle
     │     └── Rejeté  → Archivé, nouvelle proposition possible après délai
     │
     └── [Audit] ← résultat signé cryptographiquement, inscrit dans le journal immuable
```

#### Votes hors-ligne

Le vote est **entièrement asynchrone** : un membre peut voter à n'importe quel moment pendant la fenêtre, en ligne ou hors-ligne. Un vote soumis hors-ligne est mis en queue locale et synchronisé via CRDT dès la reconnexion.

```
Membre hors-ligne pendant le vote :
  → vote soumis localement, mis en queue
  → reconnexion avant deadline : vote synchronisé et comptabilisé
  → reconnexion après deadline  : vote ignoré (fenêtre fermée)

Garantie anti-doublon : 1 vote par membre par CID — le CRDT
déduplique automatiquement en cas de synchronisation multiple.
```

Le résultat n'est calculé et signé qu'à la deadline — jamais en temps réel. Cela garantit que les votes de membres temporairement hors-ligne sont bien pris en compte, et que le journal immuable ne contient que des résultats définitifs.

### Mécanismes de vote

| Mécanisme | Description | Usage type |
|---|---|---|
| **Majorité simple** | 50%+1 des votants | Décisions courantes |
| **Supermajorité** | Seuil configurable (ex : 2/3, 3/4) | Décisions structurelles |
| **Consensus** | Absence d'opposition formelle | Décisions sensibles |
| **Veto** | Un ou plusieurs membres peuvent bloquer | Décisions à fort impact |
| **Vote pondéré** | Le poids du vote varie selon le rôle | Réseaux hiérarchiques |
| **Vote préférentiel** | Classement de plusieurs options | Choix entre alternatives |

### Quorum

Le **quorum** est le nombre minimum de membres devant participer pour qu'une décision soit valide. Il est configurable par réseau et par type de décision.

```
Exemple : réseau de 20 membres
  Quorum = 50% → 10 membres doivent voter
  Si 8 votent → décision caduque
  Si 10 votent et 6 pour / 4 contre → adopté (majorité simple)
```

Si le quorum n'est pas atteint à l'échéance :
- **Décision caduque** : la proposition est archivée
- **Prolongation automatique** : le délai de vote est étendu une fois
- **Décision par défaut** : l'admin tranche en cas d'impasse (selon configuration)

### Délégation de vote

Un membre peut **déléguer son vote** à un autre membre de confiance, pour une décision spécifique ou pour une durée définie.

```
daniel délègue son vote à sophie
  pour toutes les décisions du mois de juin
  │
  ▼
sophie vote avec 2 voix (la sienne + celle de daniel)
daniel peut révoquer la délégation à tout moment
```

La délégation est **révocable à tout moment** et **non transférable** (sophie ne peut pas re-déléguer le vote de daniel).

### Proposition et délibération

Tout membre (selon son rôle) peut soumettre une proposition. La phase de délibération permet le débat avant le vote.

```
Proposition :  "Installer le service Marketplace"
Proposant :    c7d3@b4e2 (sophie)
Délibération : 48h (commentaires, questions, amendements)
Vote :         72h
Quorum :       40% des membres
Seuil :        majorité simple
```

Un amendement à une proposition relance la phase de délibération.

### Garde-fou majoritaire

Lorsqu'un admin prend une décision unilatérale (sans vote collectif) et qu'une **majorité de membres exprime son désaccord**, une **alerte réseau** est déclenchée automatiquement.

```
Admin décide unilatéralement : "Connexion avec Réseau X acceptée"
         │
         ▼
Membres notifiés → peuvent exprimer leur accord ou désaccord
         │
         ├── Majorité en désaccord atteinte
         │     │
         │     ▼
         │   ALERTE RÉSEAU déclenchée
         │   ├── Tous les membres sont notifiés
         │   ├── La décision est suspendue automatiquement
         │   └── Une procédure de vote collectif est ouverte
         │
         └── Majorité non atteinte → décision maintenue
```

**Délai de contestation** : configurable par le réseau (ex : 24h, 48h après la décision de l'admin). Passé ce délai sans majorité de désaccord, la décision est définitivement appliquée.

**Effets de l'alerte :**

| Effet | Description |
|---|---|
| **Notification universelle** | Tous les membres du réseau reçoivent une alerte, quelle que soit leur activité |
| **Suspension automatique** | La décision de l'admin est mise en attente le temps du vote collectif |
| **Vote d'urgence** | Un vote collectif est ouvert avec un délai réduit |
| **Journal** | L'alerte et son contexte sont enregistrés dans le journal de gouvernance |

Ce mécanisme s'applique aux décisions **structurelles et stratégiques** (connexions inter-réseaux, installation de services, modification des règles). Les décisions opérationnelles courantes n'y sont pas soumises.

Le seuil de désaccord déclenchant l'alerte est configurable par le réseau (par défaut : majorité simple des membres actifs).

---

### Transparence et audit

Toutes les décisions collectives sont **enregistrées de manière immuable** dans le journal de gouvernance du réseau :

- Qui a proposé, quand
- Résultat du vote (avec ou sans anonymat selon configuration)
- Date d'application
- Historique des amendements

Le journal est accessible à tous les membres. Il peut être partiellement ouvert aux réseaux connectés (selon politique de partage).

### Vote anonyme ou nominatif

Chaque réseau configure le **type de scrutin** pour chaque catégorie de décision :

| Type | Le résultat montre | Usage |
|---|---|---|
| **Nominatif** | Qui a voté quoi | Décisions où la responsabilité est importante |
| **Anonyme** | Résultat global uniquement | Élections, sujets sensibles |
| **Semi-anonyme** | Qui a voté (pas comment) | Vérification du quorum sans dévoiler les choix |

---

## Annuaires Civium

### Définition

Un **Annuaire Civium** est un type spécialisé de Réseau Civium dont la fonction principale est de **référencer et rendre découvrable** d'autres réseaux, membres ou services. Il suit les mêmes règles de gouvernance, de connexion et de partage que tout réseau Civium, avec des fonctionnalités supplémentaires de catalogue et de recherche.

```
Annuaire Civium
├── est un Réseau Civium (même protocole, même gouvernance)
├── dispose d'un catalogue structuré et interrogeable
├── peut être connecté à d'autres annuaires (fédération)
└── permet la découverte sans révéler ce qui n'est pas autorisé
```

### Types d'annuaires

| Type | Référence | Exemple |
|---|---|---|
| **Annuaire de réseaux** | Des Réseaux Civium | Annuaire des associations d'une ville |
| **Annuaire de membres** | Des individus | Annuaire des professionnels d'un secteur |
| **Annuaire de services** | Des services/plugins Civium | Registre de Services Civium (RSC) |
| **Annuaire mixte** | Réseaux + membres + services | Annuaire territorial général |
| **Registre des Réseaux Malveillants (RRM)** | Réseaux au comportement malveillant avéré | RRM Global Civium, RRM thématiques |

Le RRM est un type d'annuaire spécialisé dont le fonctionnement détaillé est décrit dans la section [Protection contre les réseaux Civium malveillants](#protection-contre-les-réseaux-civium-malveillants).

### Qui peut créer un annuaire

N'importe quel membre ou réseau Civium peut créer un annuaire. Un annuaire est souverain — il définit ses propres règles d'entrée, de validation et de visibilité.

Exemples :
- Une fédération sportive crée l'annuaire de ses clubs membres
- Une ville crée l'annuaire de ses associations et services publics
- Un collectif professionnel crée l'annuaire de ses membres

### Visibilité d'un annuaire

| Niveau | Accès à l'annuaire | Accès aux fiches |
|---|---|---|
| **Public** | Tout le monde | Configurable par entrée |
| **Semi-public** | Sur demande validée | Configurable par entrée |
| **Privé** | Sur invitation uniquement | Membres de l'annuaire seulement |

### Fiche d'entrée dans un annuaire

Chaque entrée (réseau ou membre référencé) dispose d'une **fiche** dont le contenu est défini par l'annuaire et complété par le référencé :

```json
{
  "cid": "civium:a3f9c2...e71b",
  "type": "réseau | membre | service",
  "nom_public": "Association Vélo Urbain",
  "description": "Promotion du vélo en ville",
  "tags": ["mobilité", "association", "Bordeaux"],
  "contact": "configurable (public / sur demande / masqué)",
  "url": "https://civium.asso-velo.fr",
  "date_inscription": "2026-01-15",
  "validé_par": "Annuaire Associations Bordeaux"
}
```

Le référencé contrôle **ce qu'il expose dans sa fiche** — dans la limite de ce que l'annuaire exige comme champs obligatoires.

### Inscription dans un annuaire

```
Réseau ou membre souhaite s'inscrire
         │
         ▼
Soumission de la fiche (champs obligatoires + optionnels)
         │
         ▼
Validation par l'annuaire (admin ou vote collectif)
         │
         ├── Accepté  → fiche publiée, CID inscrit
         ├── Refusé   → notification + motif optionnel
         └── En attente → délai de traitement configurable
```

Un réseau ou membre peut :
- **Se désinscire** à tout moment → fiche retirée immédiatement
- **Mettre à jour** sa fiche à tout moment → la mise à jour peut nécessiter une re-validation selon la politique de l'annuaire
- **Être retiré** par l'annuaire (ex : inactivité, non-conformité)

### Recherche et découverte

Un annuaire est interrogeable par :
- **Nom** : recherche textuelle sur les noms publics
- **Tags** : filtrage par catégorie, secteur, localisation
- **Type** : réseau, membre, service
- **Proximité** : géolocalisation optionnelle
- **CID** : recherche directe par identifiant Civium

La recherche ne retourne que les fiches dont la visibilité est compatible avec le statut du chercheur (membre, connecté, public).

### Fédération d'annuaires

Des annuaires peuvent se connecter entre eux pour former un **réseau d'annuaires fédéré** — permettant une recherche unifiée sur plusieurs catalogues sans les fusionner.

```
[Annuaire Associations Bordeaux]
           ↕
[Annuaire Associations Gironde]  ←→  [Annuaire Associations Nouvelle-Aquitaine]
           ↕
[Annuaire National Associations]
```

Une recherche dans l'annuaire national peut remonter des résultats des annuaires régionaux et locaux — selon les permissions de fédération définies entre eux.

**Règles de fédération :**
- Chaque annuaire choisit avec quels autres annuaires il se fédère
- La fédération est soumise à validation (même modèle que la connexion inter-réseaux)
- Chaque annuaire contrôle quelles fiches sont propagées vers les annuaires fédérés
- Une fiche peut apparaître dans plusieurs annuaires fédérés sans duplication des données — seule la référence (CID) est partagée

### Annuaire racine Civium

Le **protocole Civium** maintient un **annuaire racine** — point d'entrée principal pour la découverte de réseaux. Il est hébergé sur le nœud officiel Civium et fédéré avec les nœuds bootstrap communautaires. Tout nouveau réseau Civium y est **enregistré automatiquement** à sa création.

```
Annuaire racine Civium
├── Enregistrement automatique de chaque nouveau réseau
├── Liste des annuaires publics
├── Liste des réseaux (visibilité selon politique du réseau)
└── Points d'entrée pour rejoindre le maillage P2P
```

L'enregistrement automatique inscrit le CID du réseau et ses métadonnées minimales (nom public, type, date de création). La **visibilité** dans l'annuaire reste contrôlée par la politique du réseau — un réseau privé est enregistré (son CID est connu du maillage) mais ses informations ne sont pas accessibles publiquement.

```
Réseau créé → enregistrement automatique dans l'annuaire racine
                    │
                    ├── Réseau public    → fiche visible par tous
                    ├── Réseau semi-ouvert → fiche visible sur demande
                    └── Réseau privé     → CID enregistré, fiche masquée
```

### Nœuds bootstrap

Tout nouveau nœud Civium est distribué avec une **liste de nœuds bootstrap officiels** permettant de rejoindre le maillage P2P au premier démarrage. Une fois connecté, le nœud découvre d'autres pairs via le DHT et n'a plus besoin des bootstrap.

```
Nœuds bootstrap officiels (maintenus par l'équipe Civium) :
  bootstrap.civium.net:7771
  bootstrap2.civium.net:7771

Nœuds bootstrap communautaires (opt-in, liste publique) :
  tout réseau Civium peut s'y ajouter volontairement

Configuration manuelle possible :
  civium.toml → bootstrap_nodes = ["ip:port", ...]
```

Les nœuds bootstrap n'ont aucun pouvoir sur le réseau — ils servent uniquement de point d'entrée initial dans le DHT. Leur défaillance n'affecte pas les nœuds déjà connectés.

---

## Sécurité & vie privée

### Principes fondateurs

Civium est conçu selon les principes du **Privacy by Design** : la protection des données n'est pas une option ajoutée après coup, elle est structurelle. Aucune fonctionnalité ne peut être implémentée en contournant le modèle de sécurité.

```
Pas de sécurité optionnelle.
Pas de données accessibles sans permission explicite.
Pas de confiance implicite entre réseaux ou membres.
```

### Modèle de menace

Civium protège contre les menaces suivantes :

| Menace | Protection |
|---|---|
| Écoute des communications | Chiffrement de bout en bout (E2E) |
| Usurpation d'identité | Clés cryptographiques Ed25519, signatures |
| Accès non autorisé aux données | Permissions granulaires, validation obligatoire |
| Réseau malveillant avant connexion | Validation explicite, handshake cryptographique |
| Réseau connecté devenu malveillant | Zero trust continu, enforcement APC, suspension immédiate |
| Scraping via connexion légitime | Rate limiting inter-réseaux, détection d'anomalies |
| Injection de contenu malveillant | Contenu entrant sandboxé, validation stricte |
| Réseau honeypot | Vérification croisée des annuaires, signalement collectif |
| Usurpation d'identité de réseau | CID ancré cryptographiquement, impossible à forger |
| Admin abusif | Garde-fou majoritaire, journal immuable |
| Fuite de métadonnées | Minimisation des données exposées par défaut |
| Attaque sur le nœud | Chiffrement au repos, clés locales |
| Déni de service | Architecture P2P sans point central d'attaque |
| Plugin malveillant | Sandbox WASM, capabilities minimales, signature pinned |
| Attaque supply chain | Builds reproductibles, hash des manifestes, RSC signé |
| Attaque Sybil (DHT) | Preuve d'identité cryptographique, admission contrôlée |
| Eclipse attack (DHT) | Diversification des pairs, détection d'anomalies de routage |
| Compromission de clé | Révocation signée diffusée, re-keying, notification aux pairs |
| Escalade inter-plugins | Isolation mémoire totale, communication uniquement via CIL |

### Chiffrement

#### En transit
Toutes les communications entre nœuds sont chiffrées via le **Noise Protocol** (intégré dans libp2p), indépendamment du transport utilisé (TCP, QUIC, WebRTC).

```
Nœud A  ──[Noise Protocol / TLS 1.3]──  Nœud B
         chiffrement de bout en bout
         authentification mutuelle des nœuds
```

#### De bout en bout (E2E)
Le contenu sensible est chiffré **au niveau applicatif**, en plus du chiffrement de transport. Seuls les destinataires autorisés peuvent déchiffrer.

```
Message privé de daniel → chiffré avec la clé publique de sophie
                        → seule sophie (clé privée) peut lire
                        → ni le nœud relais, ni l'admin ne peuvent lire
```

La synchronisation des données entre nœuds repose sur les CRDT, qui opèrent après déchiffrement côté client. Les membres d'un réseau partagent une **clé de groupe** permettant la fusion des états CRDT. L'E2E strict (clé de paire) s'applique aux données de cercle 3 et aux messages privés — ces données ne participent pas aux CRDT et sont résolues à la reconnexion par last-write-wins.

```
Données de réseau (cercles 0-2) :
  → clé de groupe partagée entre membres
  → CRDT : fusion automatique hors-ligne
  → personne hors du réseau ne peut lire

Données sensibles (cercle 3, messages privés) :
  → clé de paire (vrai E2E)
  → pas de CRDT — last-write-wins à la reconnexion
  → ni le nœud relais, ni l'admin ne peuvent lire
```

#### Au repos
Les données stockées sur le nœud sont chiffrées avec une **clé dérivée du mot de passe maître** du membre ou du réseau. Un accès physique au nœud ne suffit pas à lire les données.

### Gestion des clés

Chaque membre et chaque réseau possède une paire de clés **Ed25519** :

```
Clé privée  → stockée localement, jamais transmise, jamais centralisée
Clé publique → diffusée dans le réseau pour authentification et chiffrement
```

#### Sauvegarde et récupération

La perte de la clé privée entraîne la perte d'accès au compte. Civium propose plusieurs mécanismes de sauvegarde :

| Mécanisme | Description |
|---|---|
| **Phrase de récupération** | Suite de mots (BIP-39) générée à la création du compte |
| **Sauvegarde chiffrée** | Export de la clé privée chiffrée, stocké par le membre |
| **Récupération sociale** | M membres de confiance (cercle 3) peuvent co-signer une récupération |

La récupération sociale est le mécanisme recommandé : N membres détiennent chacun un fragment de clé (schéma de Shamir), et M fragments suffisent à la reconstruction.

#### Rotation des clés

Un membre peut **renouveler sa paire de clés** sans perdre son historique ni ses relations. La nouvelle clé publique est signée par l'ancienne, garantissant la continuité de l'identité.

### Authentification

| Méthode | Description |
|---|---|
| **Mot de passe + clé locale** | Authentification standard, clé dérivée du mot de passe |
| **Passkey / FIDO2** | Authentification biométrique ou matérielle sans mot de passe |
| **Double facteur (2FA)** | TOTP (Google Authenticator, etc.) en complément |
| **Clé matérielle** | YubiKey ou équivalent pour les usages sensibles |

### Vie privée

#### Minimisation des données

Civium ne collecte que ce qui est strictement nécessaire au fonctionnement du protocole. Aucune donnée comportementale, aucune analytics, aucun profilage.

```
Ce que Civium ne fait pas :
✗ Analyse des comportements utilisateurs
✗ Publicité ciblée
✗ Vente ou partage de données à des tiers
✗ Conservation de métadonnées de navigation
```

#### Cloisonnement des sphères

Les données d'un réseau ne sont **jamais accessibles à un autre réseau** sans permission explicite. Un réseau famille et un réseau professionnel du même membre sont totalement étanches l'un à l'autre par défaut.

#### Droit à l'effacement

Un membre peut **supprimer ses données** à tout moment. L'effacement est **logique** : un événement CRDT `deleted` est diffusé à tous les nœuds, qui masquent la donnée et ne la servent plus. Elle peut subsister physiquement dans des sauvegardes locales des autres membres, mais elle est inaccessible. Ce modèle est conforme au RGPD — la CNIL reconnaît l'effacement effectif dès lors que la donnée n'est plus accessible.

- **Suppression partielle** : retrait de contenus spécifiques — l'événement `deleted` est propagé immédiatement
- **Désinscription d'un réseau** : suppression du profil réseau et des données personnelles ; les contributions collectives (posts, votes, wiki) suivent la politique de rétention du réseau, communiquée à l'inscription
- **Suppression du compte** : révocation de la clé, effacement des données locales et propagation `deleted` sur toutes les données personnelles dans tous les réseaux

#### Pseudonymat et anonymat

Civium supporte plusieurs niveaux d'exposition de l'identité :

```
Anonyme    → participation sans identifiant traçable (lecture seule)
Pseudonyme → identifiant stable mais non lié à l'identité réelle
Vérifié    → identité attestée par des pairs (cercle 2+)
Réel       → nom réel exposé volontairement
```

### Audit et journalisation

#### Journal local
Chaque nœud maintient un **journal local** des accès et des événements de sécurité :
- Connexions entrantes et sortantes
- Accès aux données par des services et intégrations
- Décisions de gouvernance
- Modifications de permissions

Le journal local appartient au réseau — il n'est jamais transmis automatiquement à l'extérieur.

#### Audit des intégrations
Tout accès d'un service (plugin, API, MCP, SaaS) aux données du réseau est **enregistré et consultable** par l'admin et les membres selon leur rôle. Un membre peut voir quels services ont accédé à ses données personnelles.

#### Immuabilité des décisions
Les décisions de gouvernance (votes, exclusions, connexions) sont enregistrées dans un **journal immuable** signé cryptographiquement — impossible à modifier rétrospectivement.

### Sécurité des plugins

Puisque tout service Civium — y compris les fonctions de base — est un plugin, la sécurité du système de plugins est critique. Une faille dans un plugin peut compromettre l'ensemble du réseau qui l'a installé.

#### Isolation mémoire totale

Chaque plugin s'exécute dans son propre sandbox WASM. Les plugins ne partagent aucun espace mémoire entre eux ni avec le cœur Civium.

```
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Plugin A     │  │ Plugin B     │  │ Plugin C     │
│ (sandbox)    │  │ (sandbox)    │  │ (sandbox)    │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                 │
       └─────────────────┴─────────────────┘
                         │
                    Civium CIL
               (seul point de contact)
```

Les plugins ne peuvent pas se parler directement. Toute communication inter-plugins passe par le CIL, qui applique les permissions et l'audit.

#### Limites de ressources

Chaque plugin est soumis à des quotas d'exécution configurables par le réseau :

| Ressource | Limite par défaut | Configurable |
|---|---|---|
| CPU | temps d'exécution max par appel | oui |
| Mémoire | heap WASM plafonné | oui |
| Stockage | quota de données locales | oui |
| Réseau | accès uniquement via CIL | non (structurel) |
| Fréquence | rate limit sur les appels API | oui |

Un plugin qui dépasse ses quotas est suspendu et l'admin est alerté.

#### Intégrité et supply chain

Un plugin n'est pas identifié seulement par son numéro de version — il est **ancré à un hash cryptographique** de son code :

```json
{
  "id": "com.example.mon-plugin",
  "version": "1.2.0",
  "hash": "sha256:e3b0c44298fc1c149afb...",
  "signature": "Ed25519:<sig de l'éditeur>"
}
```

- Le hash change à chaque modification, même mineure
- La signature est vérifiée par rapport à la clé publique de l'éditeur enregistrée au RSC
- Les **builds reproductibles** sont exigés pour les plugins certifiés : n'importe qui peut recompiler le source et obtenir le même hash
- Une mise à jour de plugin = un nouveau hash = une nouvelle validation governance

#### Principe de moindre privilège

Un plugin ne peut déclarer que des capabilities existantes dans le vocabulaire Civium. Il ne peut pas demander de permissions non définies. Le CIL refuse toute tentative d'accès hors manifeste, même depuis le code WASM.

```
Plugin tente d'accéder à messages.prive
  │
  ▼
CIL vérifie le manifeste
  │
  ├── capability déclarée ? non
  │
  ▼
Accès bloqué — événement de sécurité journalisé
```

#### Validation governance des mises à jour

L'installation d'une mise à jour de plugin suit le même processus que l'installation initiale. Un réseau ne peut pas se faire pousser silencieusement une mise à jour. Chaque mise à jour est :
- Soumise à la gouvernance du réseau (admin ou vote selon configuration)
- Journalisée avec l'ancien et le nouveau hash
- Révocable : retour à la version précédente possible à tout moment

---

### Sécurité du réseau P2P

L'architecture P2P introduit des menaces spécifiques absentes des architectures client-serveur.

#### Attaque Sybil

Une attaque Sybil consiste à créer un grand nombre de fausses identités pour influencer le DHT ou le routage. Civium s'en protège par :

- **Identité ancrée cryptographiquement** : chaque CID est dérivé d'une clé Ed25519 — générer des identités en masse est coûteux
- **Admission contrôlée** : rejoindre un réseau exige une invitation ou une validation — un attaquant ne peut pas auto-déclarer sa présence
- **Réputation des pairs** : les nœuds maintenus depuis longtemps sont pondérés plus fortement dans le routage DHT

#### Eclipse attack

Une eclipse attack consiste à encercler un nœud de pairs malveillants pour lui couper l'accès au réseau honnête. Civium s'en protège par :

- **Diversification géographique et topologique** des pairs connus (table de routage Kademlia)
- **Connexions sortantes prioritaires** : un nœud maintient des connexions vers des pairs qu'il a lui-même choisis, pas seulement vers ceux qui se sont connectés à lui
- **Détection d'anomalies** : si un nœud ne reçoit plus de messages de pairs connus depuis X minutes, une alerte est déclenchée et des pairs de secours sont contactés

#### Routage et confidentialité des métadonnées

Le DHT révèle par construction qui cherche quoi. Civium limite ce risque :

- Les requêtes DHT utilisent des **identifiants temporaires** distincts du CID du membre
- La résolution d'un CID vers une adresse peut passer par des **nœuds relais de confiance** (cercle 2+) pour masquer l'origine de la requête
- Les métadonnées de connexion (qui est connecté à qui) ne sont pas stockées en clair dans le DHT

---

### Protection contre les réseaux Civium malveillants

Un réseau Civium connecté est un tiers — pas un allié. Même après validation de la connexion, le principe **zero trust** s'applique en continu : chaque requête est vérifiée, chaque accès est contrôlé, aucune confiance implicite n'est accordée.

#### Zero trust continu

La validation initiale (handshake, APC) établit les droits. Elle ne les garantit pas dans le temps. À chaque requête d'un réseau connecté, le CIL vérifie :

```
Requête du Réseau B
        │
        ▼
L'APC en vigueur autorise-t-il cette ressource ?    → non → bloqué + journalisé
        │ oui
        ▼
La ressource demandée correspond-elle exactement ?   → non → bloqué + journalisé
        │ oui
        ▼
Le volume de requêtes est-il dans les limites ?      → non → rate limiting déclenché
        │ oui
        ▼
Accès accordé — journalisé
```

L'APC est re-vérifié cryptographiquement à intervalles réguliers. Si la signature ne correspond plus, la connexion est suspendue automatiquement.

#### Enforcement strict de l'APC

L'Accord de Partage Civium est le seul contrat entre deux réseaux. Tout ce qui n'y est pas explicitement listé est interdit, même si la donnée semble anodine.

```
APC : annuaire membres (lecture) + événements publics (lecture)

Réseau B tente d'accéder à :
  ├── annuaire membres     → ✓ autorisé
  ├── événements publics   → ✓ autorisé
  ├── liste des documents  → ✗ bloqué (hors APC)
  ├── profils complets     → ✗ bloqué (hors APC)
  └── métadonnées réseau   → ✗ bloqué (non déclaré)
```

#### Rate limiting inter-réseaux

Un réseau connecté ne peut pas émettre un volume illimité de requêtes. Des limites configurables s'appliquent par fenêtre temporelle :

| Limite | Valeur par défaut | Effet si dépassé |
|---|---|---|
| Requêtes / minute | 60 | Throttling progressif |
| Volume de données / heure | configurable | Suspension temporaire |
| Connexions simultanées | 10 | Refus des nouvelles connexions |
| Requêtes hors APC | 0 toléré | Alerte immédiate + journalisation |

Un pic de requêtes hors APC — même d'un seul accès — déclenche une alerte : c'est le signal d'un comportement de scraping ou d'exploration.

#### Contenu entrant : jamais exécuté, toujours validé

Tout contenu reçu d'un réseau connecté — texte, fichier, événement, fiche annuaire — est traité comme une entrée non fiable :

- **Pas d'exécution de code** : un plugin partagé par un réseau connecté s'exécute dans un sandbox encore plus restrictif que les plugins locaux
- **Validation de schéma stricte** : chaque objet reçu est validé contre le schéma attendu — tout surplus est ignoré, pas injecté
- **Assainissement du contenu riche** : le HTML/Markdown reçu est assaini avant affichage (pas de scripts, pas d'iframes, pas de ressources externes)
- **Fichiers mis en quarantaine** : les fichiers reçus via une connexion inter-réseaux sont stockés séparément et ne s'exécutent jamais automatiquement

#### Détection de comportement malveillant

Le CIL surveille en continu les patterns anormaux d'un réseau connecté :

| Signal | Interprétation | Réaction automatique |
|---|---|---|
| Requêtes hors APC répétées | Tentative d'escalade ou de scraping | Alerte admin + compteur d'incidents |
| Volume de requêtes > 10× la normale | Scraping ou DoS ciblé | Rate limiting renforcé |
| Requêtes sur des ressources supprimées | Sondage de l'état interne | Journalisation silencieuse |
| Accès depuis une clé différente de l'APC | Usurpation ou compromission | Suspension immédiate |
| Patterns d'énumération | Cartographie des membres | Throttling + alerte |

Les incidents sont comptabilisés par réseau. Au-delà d'un seuil configurable, la connexion est automatiquement suspendue et l'admin est notifié.

#### Suspension et révocation d'une connexion

Un admin peut suspendre ou révoquer une connexion à tout moment, sans délai ni justification :

```
Suspension  → accès coupé immédiatement, APC en veille, reconnexion possible
Révocation  → connexion fermée définitivement, CID du réseau ajouté en liste noire
```

La révocation est irréversible depuis l'interface standard. La réouverture d'une connexion avec un réseau révoqué nécessite une validation collective (vote), pas une décision admin seule.

#### Signalement collectif

Un réseau victime d'un comportement malveillant peut **signaler le réseau fautif** aux annuaires auxquels il appartient. Le signalement contient :

- Le CID du réseau signalé
- Le type de comportement observé (avec preuves issues du journal)
- L'horodatage et la signature du réseau signalant

Les annuaires agrègent les signalements. Un réseau cumulant plusieurs signalements vérifiés peut être :
- Marqué "signalé" dans le catalogue (visible par les réseaux qui envisagent une connexion)
- Suspendu de l'annuaire par vote de gouvernance de l'annuaire
- Exclu du maillage P2P par consensus des nœuds

Ce mécanisme de réputation distribuée ne repose sur aucune autorité centrale — c'est la communauté des réseaux qui décide.

#### Registre des Réseaux Malveillants (RRM)

Le **Registre des Réseaux Malveillants** est un type spécialisé d'Annuaire Civium dont la fonction est de centraliser, vérifier et diffuser les signalements de réseaux au comportement malveillant avéré.

```
Annuaire Civium standard   →  référence des réseaux dignes de confiance
Registre des Réseaux Malveillants (RRM)  →  référence des réseaux à risque
```

Comme tout annuaire, le RRM est **décentralisé** : plusieurs RRM peuvent coexister, gouvernés par des communautés différentes. Un réseau choisit à quels RRM il fait confiance.

##### États d'un réseau dans le RRM

```
[Signalé]  →  [En instruction]  →  [Confirmé]  →  [Exclu]
    │                │                   │
    │          Preuves insuffisantes   Appel accepté
    │                │                   │
    └────────────────┴───────────────[Classé sans suite]
```

| État | Signification | Effet par défaut |
|---|---|---|
| **Signalé** | Signalement(s) reçu(s), en dessous du seuil | Avertissement affiché lors d'une demande de connexion |
| **En instruction** | Seuil de signalements atteint, vérification en cours | Avertissement renforcé, connexion soumise à validation collective |
| **Confirmé** | Comportement malveillant vérifié par le RRM | Connexion bloquée par défaut (override possible par vote) |
| **Exclu** | Récidive ou gravité maximale | Connexion impossible, CID banni du maillage P2P |

##### Seuil et vérification

Un signalement seul ne suffit pas à lister un réseau. Le RRM exige :

- **Nombre minimum de réseaux signalants indépendants** (configurable, défaut : 3) — des réseaux liés entre eux (même fondateur, même infrastructure) comptent pour un seul
- **Preuves signées** : chaque signalement doit inclure des extraits de journal signés cryptographiquement par le réseau signalant — pas de signalement sur parole
- **Ancienneté des signalants** : un réseau créé très récemment a un poids réduit (protection contre les attaques de faux témoins coordonnées)
- **Diversité géographique / topologique** des signalants (optionnel, pour les RRM à haute exigence)

##### Droit de réponse et appel

Le réseau mis en cause est **notifié dès le premier signalement**. Il peut :

```
Notification envoyée au réseau signalé
        │
        ├── Contester : soumettre une réponse et des contre-preuves
        │     └── Le RRM instruit les deux versions avant décision
        │
        ├── Corriger : démontrer que le comportement a cessé
        │     └── Peut mener à un retrait du listing (sous surveillance)
        │
        └── Ne pas répondre
              └── L'instruction continue sans la version du signalé
```

Un réseau confirmé peut faire appel une fois. L'appel est instruit par un collège de membres du RRM distincts de ceux ayant instruit le dossier initial.

##### Intégration dans le workflow de connexion

À chaque demande de connexion entrante, le nœud consulte automatiquement les RRM auxquels il est abonné :

```
Réseau B envoie CONNECT_REQUEST
        │
        ▼
Consultation des RRM abonnés
        │
        ├── CID non listé         → handshake normal
        ├── CID signalé           → avertissement affiché à l'admin
        ├── CID en instruction    → avertissement renforcé + validation collective requise
        ├── CID confirmé          → connexion bloquée (override par vote collectif)
        └── CID exclu             → connexion refusée, réponse silencieuse
```

La consultation est asynchrone et mise en cache — elle n'introduit pas de délai perceptible dans le handshake.

##### Fédération des RRM

Les RRM peuvent se fédérer entre eux pour partager leurs listes sans les fusionner :

```
[RRM Communautés locales]  ←→  [RRM Associations]  ←→  [RRM Global Civium]
```

Le **RRM Global Civium** est lui-même un Réseau Civium — il n'est pas contrôlé par l'équipe Civium et ne constitue pas une autorité centrale. Il est gouverné par sa propre communauté selon les mêmes mécanismes que tout réseau Civium. Son CID est public et connu, ce qui permet à tout réseau de s'y abonner ou de le remplacer par un autre RRM de référence. L'équipe Civium n'y dispose d'aucun pouvoir particulier.

Un réseau peut s'abonner à plusieurs RRM avec des niveaux de confiance distincts :

```json
"rrm_subscriptions": [
  { "cid": "civium:rrm-global...", "trust": "block_confirmed" },
  { "cid": "civium:rrm-local...",  "trust": "warn_only" }
]
```

`block_confirmed` : bloque automatiquement les réseaux confirmés par ce RRM.
`warn_only` : affiche seulement un avertissement, quelle que soit la gravité.

##### Protection contre l'abus du RRM lui-même

Le RRM est un outil de protection — il ne doit pas devenir un outil de censure ou de discrimination.

| Risque | Protection |
|---|---|
| Signalements coordonnés pour exclure un réseau légitime | Seuil d'indépendance des signalants, pondération par ancienneté |
| RRM partisan ou corrompu | Chaque réseau choisit librement ses RRM — aucun n'est imposé |
| Listing permanent sans recours | Appel obligatoire instruit, listing réexaminé périodiquement |
| Fuite de la liste aux réseaux signalés | Les CID listés sont publics — la liste n'est pas secrète |
| Un RRM unique dominant | Fédération encouragée, diversité des RRM maintenue |

---

#### Plugins reçus d'un réseau connecté

Quand un réseau expose un plugin à un réseau connecté, ce plugin est traité avec le niveau de méfiance maximal :

```
Plugin local (installé par le réseau)
  └── sandbox WASM standard
  └── capabilities déclarées dans le manifeste

Plugin reçu d'un réseau connecté
  └── sandbox WASM renforcé (ressources réduites de 50%)
  └── capabilities limitées aux données partagées dans l'APC
  └── pas d'accès aux données locales hors APC
  └── re-validation governance requise à chaque mise à jour
```

Un réseau peut choisir de refuser tout plugin provenant de réseaux connectés — c'est la configuration par défaut pour les réseaux sensibles.

---

### Révocation et compromission de clé

#### Procédure de révocation

Si une clé privée est compromise, le membre doit agir immédiatement :

```
1. Générer une nouvelle paire de clés Ed25519
2. Émettre un message de révocation signé par l'ancienne clé
   └── horodaté, non rejouable (nonce unique)
3. Diffuser la révocation dans tous les réseaux du membre
4. Signer la nouvelle clé publique avec l'ancienne (chaîne de confiance)
5. Les pairs reçoivent la révocation et mettent à jour leur table de confiance
```

Passé un délai de grâce configurable (par défaut 24h), tout message signé par l'ancienne clé est rejeté — y compris les messages antérieurs à la révocation présentés avec un horodatage modifié.

#### Révocation sans accès à l'ancienne clé

Si la clé ancienne est irrémédiablement perdue ou volée et que le membre ne peut plus signer avec elle, la **récupération sociale** (schéma de Shamir) permet à M membres de cercle 3 de co-signer une révocation d'urgence sans la clé ancienne. Cette révocation est traitée comme prioritaire par le réseau.

#### Période de grâce et window d'attaque

Entre la compromission et la révocation effective, un attaquant peut se faire passer pour la victime. Civium réduit cette fenêtre par :
- Alertes de connexion anormale (nouveau device, nouvelle IP, heure inhabituelle)
- Confirmation 2FA exigée pour les actions critiques (connexion inter-réseaux, modification de gouvernance)
- Journalisation de toutes les actions signées, consultable après coup pour audit

---

### Coopération judiciaire et réseaux illicites

Civium est conçu pour protéger la vie privée des membres légitimes — pas pour offrir l'impunité à des activités illégales. Cette section définit le cadre de coopération avec les autorités judiciaires, en maintenant le Privacy by Design pour tous les utilisateurs non visés.

#### Principes

```
Ce que Civium peut faire :
  ✓ Fournir des métadonnées sur réquisition judiciaire
  ✓ Tracer les CID des réseaux illicites signalés
  ✓ Coopérer avec les forces de l'ordre via le RRM-LEA

Ce que Civium ne peut pas faire (par conception) :
  ✗ Déchiffrer les messages E2E (clé de paire — personne ne la détient)
  ✗ Accéder aux données de cercle 3
  ✗ Surveiller des réseaux sans réquisition judiciaire
```

Le contenu des communications reste inaccessible même à l'équipe Civium. Pour accéder au contenu, les autorités doivent procéder à la **saisie des appareils** — procédure judiciaire standard, indépendante du protocole.

#### Métadonnées conservées par les opérateurs de nœuds

Les opérateurs de nœuds Civium (dont le nœud officiel Civium) conservent des **journaux de connexion** pendant une durée légalement définie (par défaut : 12 mois, selon la législation applicable) :

| Métadonnée | Description | Conservée |
|---|---|---|
| CID source | Identifiant du nœud émetteur | Oui |
| CID destination | Identifiant du nœud destinataire | Oui |
| Horodatage | Date et heure de connexion | Oui |
| Adresse IP | IP du nœud au moment de la connexion | Oui |
| Volume | Quantité de données échangées | Oui |
| Contenu | Corps des messages, fichiers | **Non** — E2E, non accessible |

Ces journaux sont chiffrés au repos et ne sont accessibles que sur **réquisition judiciaire** (commission rogatoire, ordonnance de tribunal).

#### RRM-LEA — Registre pour les Forces de l'Ordre

Le **RRM-LEA** (Law Enforcement Authorities) est un type spécialisé de RRM dédié à la coopération judiciaire. Il est distinct du RRM communautaire et réservé aux forces de l'ordre accréditées (police nationale, Europol, INTERPOL, etc.).

```
Réseau victime ou témoin
        │
        ▼
Signalement au RRM-LEA
  ├── CID du réseau illicite
  ├── Nature de l'activité illégale
  └── Preuves signées (extraits de journal, captures)
        │
        ▼
RRM-LEA instruit le dossier
  ├── Vérifie l'indépendance des signalants
  ├── Valide les preuves cryptographiques
  └── Transmet aux autorités accréditées
        │
        ▼
Autorités judiciaires
  ├── Réquisition aux opérateurs de nœuds → métadonnées
  └── Saisie des appareils → contenu (procédure standard)
```

**Accès au RRM-LEA :**
- Les forces de l'ordre s'authentifient avec un CID institutionnel vérifié
- Les signalements sont pseudonymisés : le CID de la victime n'est pas exposé aux autorités sans son consentement explicite
- Les autorités reçoivent uniquement les CID des réseaux signalés et les preuves jointes

#### Traçabilité des CID

Chaque CID est cryptographiquement lié à une clé publique Ed25519. Un CID seul ne révèle pas l'identité réelle d'un membre — mais il permet de :

- **Relier des activités** : un même CID utilisé dans plusieurs réseaux illicites crée un graphe de relations exploitable
- **Identifier le nœud d'hébergement** : le DHT résout CID → adresse IP → opérateur de nœud → réquisition judiciaire
- **Dater les activités** : les journaux de connexion horodatés permettent de reconstituer une chronologie

```
CID illicite : civium:x7f2...
        │
        ▼
DHT : résolution CID → IP 185.220.xx.xx
        │
        ▼
Réquisition à l'hébergeur → identité de l'abonné
        │
        ▼
Réquisition au nœud Civium → journaux de connexion
```

L'identité réelle n'est accessible que via les opérateurs de nœuds et les hébergeurs — pas via le protocole Civium lui-même.

#### Obligations des opérateurs de nœuds

Tout opérateur d'un nœud Civium public (accessible depuis internet) est soumis aux obligations légales de son pays :

- **Conservation des journaux** pendant la durée légale applicable
- **Réponse aux réquisitions judiciaires** dans les délais légaux
- **Non-divulgation** de l'existence d'une réquisition en cours (si soumis à une obligation de confidentialité judiciaire)

Le nœud officiel Civium est opéré sous juridiction française et respecte le droit européen (RGPD, directive e-Privacy, Convention de Budapest sur la cybercriminalité).

---

### Processus de sécurité du projet

#### Audit externe

Avant chaque version majeure du protocole, un **audit de sécurité externe indépendant** est conduit. Le rapport est publié intégralement, qu'il soit favorable ou non.

#### Bug bounty

Un programme de **bug bounty** sera lancé avant la v1 publique. Les détails (montants, périmètre, plateforme) seront publiés dans la gouvernance du projet. Les catégories de criticité retenues :

| Criticité | Exemple |
|---|---|
| Critique | RCE, contournement du sandbox WASM, compromission de clé |
| Haute | Fuite de données inter-réseaux, bypass de permissions APC/CIL |
| Moyenne | Déni de service, fuite de métadonnées |
| Faible | Comportements inattendus sans impact de sécurité direct |

#### Divulgation responsable

Civium est un protocole ouvert. Les failles de sécurité peuvent être signalées via un processus de **divulgation responsable** défini dans la gouvernance du projet. Toute faille confirmée est communiquée à l'ensemble des opérateurs de nœuds avant publication.

---

## Cas d'usage

### Famille

La famille Martin veut un espace privé commun sans dépendre de Google Photos ou WhatsApp.

1. L'un des parents crée un Réseau Civium `famille-martin` (réseau privé, fermé)
2. Il invite les membres de la famille — chacun reçoit un lien de pairing
3. Le réseau dispose d'emblée : agenda partagé, coffre-fort de documents (actes, contrats, photos), messagerie chiffrée E2E
4. Les cousins Dupont ont leur propre réseau. Les deux familles établissent une **connexion partielle** : agenda commun visible des deux côtés, documents privés inaccessibles
5. Grand-mère, peu à l'aise avec le numérique, accède via l'app web depuis son navigateur

```
famille-martin  ──[agenda: lecture/écriture]──  famille-dupont
                ──[documents: aucun]──
```

### Association

L'association "Vélo Urbain" veut gérer ses membres, ses événements et sa communication sans dépendre d'un outil tiers.

1. Le président crée le réseau `asso-velo` (réseau semi-ouvert)
2. Le réseau s'enregistre automatiquement dans l'annuaire racine Civium et dans l'annuaire des associations de la ville
3. Les membres rejoignent le réseau : chacun choisit son nom affiché et sa visibilité
4. L'association installe les plugins Agenda, Fil d'activité et Marketplace (vente de matériel)
5. Une connexion totale est établie avec l'association "Mobilité Douce" pour co-organiser des événements — les membres des deux assos voient un agenda fusionné
6. Une décision de connexion avec un réseau commercial est soumise au vote collectif des membres (gouvernance participative)

### Équipe professionnelle

Une agence de design veut collaborer avec des freelances partenaires sans exposer ses projets internes.

1. Les 3 associés créent le réseau `agence-design` (réseau fermé, gouvernance collective)
2. Ils installent les plugins Gestion de projet, Documents et Facturation
3. Ils créent un réseau séparé `freelances-partenaires` avec une **connexion partielle** vers `agence-design` :
   - Annuaire de compétences : accessible aux freelances
   - Projets en cours : inaccessibles
   - Missions disponibles : accessibles en lecture
4. Toute nouvelle connexion inter-réseaux requiert la validation des 3 associés (quorum 3/3)
5. Un freelance compromettant ses accès est exclu du réseau partenaires sans affecter le réseau interne

```
agence-design  ──[compétences: lecture]──  freelances-partenaires
               ──[missions: lecture]──
               ──[projets: aucun]──
```

### Individu

Sophie appartient à 3 réseaux différents et veut que ses identités restent cloisonnées.

1. Sophie a un compte Civium (CID unique). Elle rejoint 3 réseaux :
   - `famille-martin` → nom affiché : "Sophie", profil familial
   - `asso-velo` → nom affiché : "Sophie R.", bénévole logistique
   - `reseau-pro` → nom affiché : "S. Renard", portfolio et tarif journalier
2. Ses 3 identités réseau ne sont pas liées publiquement — connaître son identité dans l'asso ne révèle pas son appartenance au réseau famille
3. Dans `asso-velo`, elle place daniel au cercle 2 (confiance) — il voit son profil complet. Elle le place au cercle 1 dans `reseau-pro` — il ne voit que son nom
4. Elle apparaît dans l'annuaire public Civium uniquement pour `asso-velo`, selon son choix

---

## Architecture technique

### Modèle hybride : Local-first + Adressage direct + P2P

Chaque Réseau Civium est **doublement adressable** : il peut être joint via une adresse directe (IP ou URL) ET via le maillage P2P. Les deux modes sont combinables et complémentaires.

```
        Connexion directe (IP / URL)
┌──────────────────────────────────────────┐
│                                          │
[Réseau A]                            [Réseau B]
https://civium.asso-velo.fr           192.168.1.10:7771
│                                          │
└──────────── Maillage P2P Civium ─────────┘
              (découverte + routage DHT)
```

| Mode | Comment | Quand l'utiliser |
|---|---|---|
| **Direct URL** | URL publique (`https://civium.monasso.fr`) | Serveur, VPS, haute disponibilité |
| **Direct IP** | Adresse IP + port (`192.168.1.10:7771`) | NAS, serveur local, réseau interne |
| **P2P (DHT)** | Découverte via le maillage, sans adresse fixe | Mobile, IP dynamique, anonymat renforcé |
| **Hybride** | URL/IP + maillage P2P simultanément | Mode recommandé — résilience maximale |

### Identifiant Civium (CID)

Chaque réseau possède un **identifiant Civium unique** (CID) — une clé cryptographique Ed25519 indépendante de son adresse réseau. Stable même si l'IP ou l'URL change. La résolution CID → adresse est assurée par le maillage P2P (DHT Kademlia).

```
CID : civium:a3f9c2...e71b   →  résout vers  →  https://civium.monasso.fr
                                               ou  192.168.1.10:7771
                                               ou  route P2P via maillage
```

---

## Pile protocolaire

Civium repose sur une **pile en quatre couches**, chaque couche utilisant le meilleur protocole existant :

```
┌─────────────────────────────────────────────────┐
│  Couche 4 — Protocole Civium (CP)               │
│  Connexions inter-réseaux, permissions,          │
│  gouvernance, annuaires, cercles de confiance    │
├─────────────────────────────────────────────────┤
│  Couche 3 — Fédération : ActivityPub  (roadmap) │
│  Interopérabilité avec l'écosystème              │
│  décentralisé (Mastodon, PeerTube, etc.)         │
├─────────────────────────────────────────────────┤
│  Couche 2 — Sync & données : CRDT               │
│  Synchronisation locale et hors-ligne,           │
│  résolution de conflits sans serveur central     │
├─────────────────────────────────────────────────┤
│  Couche 1 — Transport : libp2p                   │
│  Découverte DHT (Kademlia), NAT traversal,       │
│  chiffrement (Noise Protocol), QUIC/TCP/WebRTC   │
└─────────────────────────────────────────────────┘
```

### Couche 1 — libp2p (transport)
**Pourquoi libp2p** : protocole de transport P2P mature, utilisé en production par IPFS, Ethereum et Filecoin. Il gère nativement :
- La découverte de pairs via DHT Kademlia (résolution CID → adresse)
- Le NAT traversal et le hole punching (joindre un NAS derrière une box)
- Le chiffrement de toutes les connexions via le Noise Protocol
- Les transports TCP, QUIC et WebRTC (navigateurs et mobiles)
- L'identité des nœuds via des clés Ed25519 — qui deviennent les CID Civium

### Couche 2 — CRDT (sync & données)
Les données de chaque réseau sont stockées localement et synchronisées via des **CRDT** (Conflict-free Replicated Data Types). Cela garantit :
- Un fonctionnement **hors-ligne** complet
- Une synchronisation **sans serveur central**
- Une **résolution automatique des conflits** lors de la reconnexion

Les CRDT opèrent sur les données après déchiffrement côté client, via la clé de groupe du réseau. Les données E2E strictes (cercle 3, messages privés) n'utilisent pas de CRDT — leur synchronisation repose sur last-write-wins à la reconnexion. Voir la section Chiffrement pour le détail.

### Couche 3 — ActivityPub (fédération) *(roadmap)*
L'interopérabilité avec le Fediverse (Mastodon, PeerTube, Pixelfed, etc.) via ActivityPub est prévue mais hors scope de la v1. Elle sera spécifiée dans une version ultérieure, une fois la pile de base stable. Le mapping prévu : contenu cercle 0 → public ActivityPub, cercle 1 → followers, cercles 2-3 non fédérés.

### Couche 4 — Protocole Civium (CP)
La couche applicative propre à Civium. Elle définit :
- La création et la gestion des Réseaux Civium
- Le handshake et le cycle de vie des connexions inter-réseaux
- Le modèle de permissions et les cercles de confiance
- Les messages de gouvernance (votes, validations, révocations)
- Le protocole des annuaires

---

## Cycle de vie d'une connexion inter-réseaux

### Règle fondamentale

> **Aucun accès n'est possible entre deux réseaux tant que la validation n'est pas complète et acceptée des deux côtés.**

Pendant toute la phase de validation, les réseaux sont visibles l'un de l'autre uniquement via les informations qu'ils ont explicitement rendues publiques dans l'annuaire (nom, description, type). Aucune donnée interne, aucun membre, aucun contenu n'est accessible.

### États possibles

```
[Aucune]  →  [Demandée]  →  [En validation]  →  [Validée]  →  [Active]
                 │                  │                               │
                 │            [Refusée]                       [Suspendue]
                 │            [Bloquée]                       [Révoquée]
                 │
          ╔══════════════════════════════╗
          ║  ACCÈS : AUCUN               ║  ← dans tous les états sauf [Active]
          ╚══════════════════════════════╝
```

### Déroulement du handshake

```
Réseau A                                              Réseau B
   │                                                      │
   │── CONNECT_REQUEST (CID_A, type, permissions) ───────▶│
   │                                                      │
   │        ╔═══════════════════════════════╗             │
   │        ║  ÉTAT : EN VALIDATION         ║             │
   │        ║  Aucun accès des deux côtés   ║             │
   │        ╚═══════════════════════════════╝             │
   │                                                      │  Admin examine
   │                                                      │  ou vote collectif
   │◀── CONNECT_RESPONSE (accepté / refusé / bloqué) ────│
   │                                                      │
   │  Si accepté :                                        │
   │── PERMISSION_SYNC (accord signé cryptographiquement)▶│
   │◀── PERMISSION_ACK ───────────────────────────────────│
   │                                                      │
   │  ════════ Connexion active — accès selon permissions ════════
```

### Validation selon le modèle de gouvernance

| Modèle | Qui valide | Délai |
|---|---|---|
| **Admin seul** | L'administrateur du réseau accepte ou refuse | Immédiat dès décision |
| **Collectif** | Un quorum de membres désignés doit voter | Selon le délai de vote configuré |
| **Mixte** | L'admin propose, les membres confirment (ou inversement) | Deux étapes séquentielles |

Tant que le quorum n'est pas atteint ou que le délai de vote n'est pas écoulé, la connexion reste en état **En validation** — aucun accès accordé.

### Politique de partage de données

Chaque réseau définit souverainement **ce qu'il expose** aux réseaux avec lesquels il est connecté. Ce paramétrage est indépendant pour chaque connexion : un réseau peut partager plus avec un partenaire de confiance et moins avec un autre.

#### Catégories de données partageables

| Catégorie | Exemples | Granularité possible |
|---|---|---|
| **Annuaire membres** | Noms, profils, compétences | Tout / partiel / aucun |
| **Agenda & événements** | Événements publics, disponibilités | Lecture seule / écriture partagée |
| **Contenus & publications** | Articles, annonces, ressources | Tout / par tag / aucun |
| **Fichiers & documents** | Documents partagés, médias | Par dossier / par fichier / aucun |
| **Services** | Offres, missions, demandes | Lecture / participation |
| **Flux d'activité** | Notifications d'événements du réseau | Activé / désactivé |
| **Métadonnées réseau** | Nombre de membres, date de création | Public / privé |

#### Niveaux d'accès

Pour chaque catégorie, le réseau choisit le niveau accordé au réseau connecté :

```
Aucun  →  Lecture  →  Lecture + Commentaire  →  Participation  →  Co-administration
  │            │                  │                    │                    │
Défaut     Voir sans          Voir + réagir        Contribuer           Gérer ensemble
           interagir                               du contenu
```

#### Asymétrie des droits

La politique de partage est **asymétrique** par défaut : Réseau A peut exposer son agenda à Réseau B sans que B expose le sien en retour. Chaque réseau définit sa propre politique indépendamment.

```
Réseau A  ──[agenda: lecture]──▶  Réseau B
Réseau A  ◀──[annuaire: aucun]──  Réseau B
```

#### Accord de partage signé

Lors de la validation, les politiques des deux réseaux sont formalisées dans un **Accord de Partage Civium (APC)** — document signé cryptographiquement par les deux parties. Cet accord :
- Liste explicitement chaque catégorie exposée et le niveau d'accès accordé
- Est versionné : toute modification requiert une nouvelle validation
- Peut être révoqué unilatéralement à tout moment (retour à l'état Suspendu puis Révoqué)

#### Modification des permissions en cours de connexion

Un réseau peut à tout moment **restreindre ou élargir** les données qu'il expose, sans rompre la connexion :
- **Restriction** : prise d'effet immédiate, les données retirées deviennent inaccessibles
- **Élargissement** : requiert une re-validation par le réseau partenaire avant prise d'effet

---

### Refus et blocage

Un réseau peut **refuser** ou **bloquer** une demande de connexion entrante :

| Action | Effet |
|---|---|
| **Refus simple** | La demande est rejetée, le réseau demandeur peut en soumettre une nouvelle |
| **Refus motivé** | Un message optionnel explique le refus (ex : "hors périmètre de notre réseau") |
| **Blocage** | Le CID du réseau demandeur est inscrit en liste noire — aucune nouvelle demande possible |
| **Révocation** | Une connexion active est coupée unilatéralement ; les données partagées sont désynchronisées |

Le refus ou le blocage peut être décidé par l'admin seul ou soumis à validation collective, selon le modèle de gouvernance du réseau.

### Hébergement d'un nœud

| Support | Adressage | Disponibilité |
|---|---|---|
| VPS / serveur dédié | URL publique | Haute — recommandé pour les réseaux communautaires |
| NAS / Raspberry Pi | IP locale + P2P | Bonne — idéal pour les familles et équipes |
| Appareil mobile | P2P uniquement | Variable — nœud léger, sync asynchrone |
| Instance mutualisée | URL publique déléguée | Haute — pour les membres sans infrastructure |

### Local-first

Quelle que soit la méthode de connexion, les données restent sur le nœud du réseau. L'application fonctionne hors-ligne ; la synchronisation est asynchrone et chiffrée de bout en bout.

---

## Services Civium

### Principe : tout est plugin

Civium ne distingue pas de fonctionnalités "natives" séparées du système de plugins. **Tout service — y compris la messagerie, l'agenda ou la gouvernance — est un plugin**, soumis aux mêmes règles, aux mêmes permissions et au même cycle de vie que n'importe quel plugin tiers.

```
┌─────────────────────────────────────────────────────┐
│               Réseau Civium                         │
│                                                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────┐  │
│  │Messagerie│ │  Agenda  │ │Marketplace│ │ ... + │  │
│  └──────────┘ └──────────┘ └──────────┘ └───────┘  │
│                                                     │
│    ← tous des plugins, préinstallés ou ajoutés →    │
└─────────────────────────────────────────────────────┘
                        ↕
         Registre de Services Civium (RSC)
         (catalogue public de services disponibles)
```

Cette décision architecturale a trois conséquences directes :

- **API prouvée dès le départ** : les plugins officiels utilisent exactement la même API que les plugins tiers — si l'API est insuffisante, Civium lui-même en souffre en premier
- **Aucun privilège caché** : un développeur tiers peut remplacer, forker ou améliorer n'importe quel service officiel avec les mêmes capacités
- **Désinstallation possible** : un réseau peut retirer un plugin préinstallé dont il n'a pas besoin. Exception : les plugins **système** (Gouvernance et CIL) ne peuvent pas être désinstallés — ils sont requis par le protocole. Le principe "tout est plugin" s'entend au sens de l'API et du sandboxing, pas de la désinstallabilité.

### Plugins préinstallés

Ces plugins sont installés par défaut à la création d'un réseau :

| Plugin | Type | Peut être retiré |
|---|---|---|
| **Gouvernance** | Système | Non |
| **CIL** | Système | Non |
| **Messagerie** | Standard | Oui |
| **Agenda** | Standard | Oui |
| **Annuaire** | Standard | Oui |
| **Documents** | Standard | Oui |
| **Fil d'activité** | Standard | Oui |
| **Notifications** | Standard | Oui |

### Plugins additionnels (exemples)

Plugins installables selon les besoins du réseau, via le RSC ou en import direct :

| Plugin | Usage type |
|---|---|
| **Marketplace** | Annonces, offres, échanges de biens et services entre membres |
| **Visioconférence** | Appels et réunions P2P au sein du réseau |
| **Gestion de projet** | Tâches, jalons, tableau kanban collaboratif |
| **Carte & géolocalisation** | Cartographie des membres, lieux et événements |
| **Petites annonces** | Offres d'emploi, covoiturage, troc, don |
| **Formation** | Cours, ressources pédagogiques, suivi de progression |
| **Facturation** | Devis, factures, suivi des paiements entre membres |
| **Wiki** | Base de connaissances collaborative du réseau |
| **Sondages avancés** | Consultations, budgets participatifs, délibérations |
| **Bibliothèque** | Gestion et prêt de ressources physiques ou numériques |

Cette liste n'est pas limitative — elle illustre les possibilités.

### Architecture des services : modèle d'intégration universel

Civium ne se limite pas aux plugins embarqués. Il supporte **plusieurs types d'intégration**, tous traités avec le même modèle de permissions et de gouvernance.

```
┌──────────────────────────────────────────────────────────────┐
│                      Réseau Civium                           │
│                                                              │
│              Civium Integration Layer (CIL)                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  permissions · gouvernance · audit · chiffrement     │   │
│  └──────────────────────────────────────────────────────┘   │
│       ↕             ↕              ↕              ↕          │
│  ┌────────┐   ┌──────────┐  ┌──────────┐  ┌──────────┐     │
│  │Plugin  │   │  API     │  │  SaaS    │  │ Webhook  │     │
│  │natif   │   │externe   │  │connector │  │/ Event   │     │
│  └────────┘   └──────────┘  └──────────┘  └──────────┘     │
└──────────────────────────────────────────────────────────────┘
```

Tous les types d'intégration passent par la même couche — le **Civium Integration Layer (CIL)** — qui applique les règles de permissions, de gouvernance et d'audit de manière uniforme.

### Types d'intégration

#### Plugin natif
Module embarqué directement dans le nœud Civium. Fonctionne hors-ligne, accède aux données locales, dispose d'une interface utilisateur intégrée.

```
manifest.civium.json        ← identité, version, permissions requises
├── backend/                ← logique métier (ex : Rust, Go, WASM)
├── ui/                     ← interface intégrée à Civium
└── civium-api/             ← accès aux données, membres, événements
```

Cas d'usage : marketplace interne, gestion de projet, wiki, facturation.

#### API externe (REST / GraphQL / gRPC)
Connexion à un service tiers exposant une API standard. Civium agit comme client : il envoie ou récupère des données selon les permissions accordées.

```
Réseau Civium  →  [CIL]  →  https://api.service-tiers.com
                              (REST / GraphQL / gRPC)
```

Le réseau définit :
- Quelles données Civium peut envoyer à l'API (et lesquelles sont interdites)
- Quelles données l'API peut renvoyer dans Civium
- La fréquence et les conditions de synchronisation

Cas d'usage : intégration d'un ERP, d'un CRM, d'un service de paiement, d'un outil RH.

#### Connecteur SaaS
Intégration pré-configurée avec des services SaaS tiers connus. Le connecteur encapsule l'authentification OAuth et la logique de synchronisation.

```
Réseau Civium  →  [CIL]  →  Connecteur SaaS  →  Google Calendar
                                               →  Stripe
                                               →  Notion
                                               →  Slack / Matrix
                                               →  GitHub / GitLab
                                               →  Airtable
                                               →  ...
```

Le connecteur est un plugin spécialisé. Il déclare explicitement quelles données transitent entre Civium et le SaaS. **Les données restent dans Civium comme source de vérité** — le SaaS est un miroir ou un outil complémentaire, jamais propriétaire des données.

Cas d'usage : synchroniser l'agenda Civium avec Google Calendar, envoyer des notifications vers Slack, déclencher des paiements via Stripe.

#### Webhook & événements
Civium peut émettre ou recevoir des **événements** (webhooks) pour s'intégrer dans des flux automatisés ou des architectures microservices.

```
Événement Civium  →  [CIL]  →  POST https://mon-service.fr/webhook
                                (payload JSON signé)

POST https://civium.monreseau.fr/hook  →  [CIL]  →  Traitement interne
(événement entrant)
```

Cas d'usage : déclencher une action dans un outil tiers lors d'un événement Civium (nouveau membre, nouveau document, vote validé), ou recevoir des données externes et les injecter dans le réseau.

#### Microservice hébergé
Un service externe développé et hébergé indépendamment, qui communique avec Civium via l'API Civium. Le microservice peut avoir sa propre infrastructure, sa propre base de données, et expose des fonctionnalités à un ou plusieurs réseaux Civium.

```
Réseau Civium  ←→  [CIL]  ←→  Microservice (URL / CID Civium)
                               hébergé par l'éditeur ou en auto-hébergé
```

Cas d'usage : service de visioconférence, moteur de recherche, IA spécialisée, service de signature électronique.

#### Serveur MCP (Model Context Protocol)
Un plugin Civium peut exposer un **serveur MCP**, permettant à des agents IA (Claude, GPT, Mistral, etc.) d'accéder aux données du réseau de manière structurée et contrôlée. MCP peut également servir de **canal de partage de données entre réseaux Civium**.

**MCP = transport. APC = contrat.** MCP ne définit pas les droits d'accès — c'est l'APC en vigueur entre les deux réseaux qui les définit. Toute requête MCP inter-réseaux est vérifiée par le CIL exactement comme n'importe quelle requête : contrôle de l'APC, rate limiting, audit. Un réseau connecté ne peut accéder via MCP qu'aux ressources explicitement listées dans son APC.

```
                    ┌─────────────────────────────┐
Agent IA (Claude)   │       Réseau Civium          │
        │           │                              │
        │──MCP─────▶│  Serveur MCP Civium          │
        │           │  ┌────────────────────────┐  │
        │◀──────────│  │ Ressources exposées :  │  │
   réponses        │  │  · agenda              │  │
   contextuelles   │  │  · annuaire membres    │  │
                   │  │  · documents autorisés │  │
                   │  │  · fil d'activité      │  │
                   │  └────────────────────────┘  │
                   └─────────────────────────────┘
```

**Deux usages du MCP dans Civium :**

**1. Accès IA aux données du réseau**
Un agent IA peut interroger le réseau Civium via MCP pour répondre à des questions, générer des résumés, analyser des données ou automatiser des tâches — sans jamais accéder à plus que ce que les permissions MCP autorisent.

```
"Quels événements sont prévus cette semaine ?"
"Qui dans le réseau a des compétences en design ?"
"Résume les décisions du dernier vote collectif."
```

**2. Partage de données entre réseaux Civium via MCP**
Un réseau peut exposer un serveur MCP à un autre réseau connecté. Le réseau distant interroge les données exposées via le protocole MCP — standardisé, versionné, auditable.

```
Réseau A  ──[MCP]──▶  Réseau B
          expose :        consomme :
          annuaire        recherche de membres
          événements      affichage dans son agenda
          catalogue       marketplace fédérée
```

**Règles de gouvernance MCP :**
- Le serveur MCP d'un réseau est **désactivé par défaut**
- L'activation est soumise à la gouvernance du réseau (admin ou vote)
- Chaque ressource exposée via MCP suit le même modèle de permissions que les autres intégrations
- Tout accès MCP est **audité et journalisé** — le réseau sait qui a interrogé quoi et quand
- Un accès MCP peut être révoqué à tout moment, pour un agent ou un réseau spécifique

Cas d'usage : assistant IA de réseau, recherche fédérée entre réseaux, automatisation de flux de données, agrégation de contenus entre communautés.

### Manifeste d'intégration universel

Tout type d'intégration déclare un **manifeste** (`manifest.civium.json`) standardisé :

```json
{
  "id": "com.example.mon-service",
  "name": "Mon Service",
  "version": "1.2.0",
  "type": "plugin | api | saas | webhook | microservice",
  "author": "CID ou URL de l'éditeur",
  "permissions": {
    "read":  ["membres.annuaire", "agenda.evenements"],
    "write": ["agenda.evenements"],
    "forbidden": ["documents.prive", "messages"]
  },
  "data_residency": "local | remote | hybrid",
  "offline_capable": true,
  "expose_to_connected_networks": false
}
```

### Normalisation des plugins : normer le contrat, pas l'implémentation

#### Principe fondateur

La norme Civium s'applique à **ce que le plugin déclare et comment il communique**, pas à ce qu'il fait en interne. Cette séparation permet une grande diversité d'usages tout en garantissant sécurité, gouvernance et auditabilité.

| Normé par Civium | Libre pour le développeur |
|---|---|
| Permissions déclarées (capabilities) | Langage et technologie interne |
| API surface d'accès aux données | Logique métier |
| Hooks de cycle de vie | Interface utilisateur (dans un slot défini) |
| Manifeste lisible par la gouvernance | Hébergement (local, distant, hybride) |

Le CIL n'a besoin de connaître que le manifeste, les hooks et l'API surface. Tout le reste est opaque et libre.

#### Modèle de capabilities atomiques

Plutôt qu'une liste de fonctionnalités figées, Civium définit des **capabilities atomiques** combinables. Un plugin déclare exactement ce dont il a besoin — ni plus :

```json
"permissions": {
  "read":     ["membres.annuaire", "agenda.evenements"],
  "write":    ["agenda.evenements"],
  "listen":   ["events.nouveau_membre", "votes.resultat"],
  "emit":     ["notifications.membre"],
  "forbidden": ["messages", "documents.prive"]
}
```

Un plugin qui écoute des événements déclare uniquement `listen`. Un plugin de facturation déclare `read:membres` + `write:documents`. Chaque réseau peut bloquer certaines capabilities dans sa gouvernance. La norme est le **vocabulaire**, pas le catalogue des plugins possibles.

#### Sandbox d'exécution (WASM)

Les plugins natifs sont compilés en **WebAssembly** et s'exécutent dans un bac à sable : ils ne peuvent accéder à rien sauf via l'API Civium injectée selon les capabilities accordées.

```
Plugin WASM
  │
  └── ne voit que civium_api.*  (capabilities accordées)
  └── ne peut pas faire de syscall direct
  └── ne peut pas accéder au réseau sauf via le CIL
```

Le langage et l'architecture interne du plugin sont totalement libres — la contrainte est la frontière, pas l'intérieur.

#### Contrat de cycle de vie

Les **points d'ancrage** sont normés, pas le comportement :

| Hook | Déclencheur |
|---|---|
| `on_install()` | Migration de schéma, état initial |
| `on_event(e)` | Réaction à un événement Civium |
| `on_request(r)` | Réponse à une action utilisateur |
| `on_sync()` | Réconciliation CRDT lors de reconnexion |
| `on_uninstall()` | Nettoyage des données |

Un plugin qui n'implémente pas `on_sync()` ne peut pas être installé sur un réseau qui exige le fonctionnement hors-ligne. La compatibilité est déclarative et vérifiable.

#### Niveaux de compliance

Plutôt qu'une norme unique, trois niveaux permettent d'accueillir les plugins expérimentaux comme les plugins distribués à grande échelle :

| Niveau | Exigences | Débloque |
|---|---|---|
| **Minimal** | Manifeste valide + capabilities déclarées | Installation locale, import direct |
| **Compatible RSC** | + sandbox WASM + hooks de cycle de vie | Publication dans le Registre |
| **Certifié** | + audit de permissions + tests fournis | Badge de confiance, réseaux sensibles |

La diversité vit au niveau minimal. La confiance se construit vers le haut.

---

### Registre de Services Civium (RSC)

Le **RSC** est un annuaire décentralisé des services disponibles. Il fonctionne comme un catalogue ouvert :

- **Publier** : tout développeur peut soumettre un service au RSC
- **Découvrir** : un réseau parcourt le catalogue et installe les services qui lui conviennent
- **Vérifier** : chaque service est signé par son auteur — le réseau sait exactement ce qu'il installe
- **Auto-héberger** : un réseau peut installer un service sans passer par le RSC (import direct)

### Installation et partage d'un service

```
1. L'admin (ou le collectif) sélectionne un service dans le RSC
2. Le manifeste est examiné : permissions requises, éditeur, version
3. Validation selon la gouvernance du réseau (admin seul ou vote collectif)
4. Le service est installé et activé pour les membres du réseau
5. L'admin choisit si ce service est exposé aux réseaux connectés
   └── Si oui : les permissions de partage sont définies dans l'APC
```

### Services partagés entre réseaux

Quand un réseau expose un service à un réseau connecté, il définit le niveau d'accès accordé — exactement comme pour les données. Exemples :

| Réseau A expose | Réseau B peut |
|---|---|
| Marketplace (lecture) | Consulter les annonces, ne pas en publier |
| Agenda (participation) | Voir les événements et s'y inscrire |
| Formation (lecture) | Accéder aux cours publiés |
| Wiki (écriture) | Contribuer au contenu |

---

## Identité des membres

### Identifiant Civium (CID membre)

L'identité principale d'un membre est son **CID** — dérivé de sa clé publique Ed25519 générée à la création du compte. Le CID est l'unique identifiant garanti globalement unique sur le protocole, car son unicité est cryptographique, pas dépendante d'un registre central.

```
CID membre :  civium:a3f9c2...e71b   (dérivé de la clé publique Ed25519)
```

```
Compte maître
├── CID        : civium:a3f9c2...e71b  (identité globale, immuable)
├── clé privée : Ed25519 (conservée sur le nœud principal, jamais transmise)
└── clé publique : diffusée pour vérification et résolution CID
```

### Nœud principal et multi-appareils

Le compte maître réside sur un **nœud principal** — le premier nœud Civium sur lequel le compte a été créé. Ce nœud peut être un VPS, un NAS, ou une instance hébergée (comme le nœud web Civium). Le CID reste stable quelle que soit l'adresse du nœud : si l'URL ou l'IP change, le DHT met à jour la résolution CID → adresse automatiquement.

#### Modèle de clés : maître + sous-clés par appareil

La clé privée maître ne quitte jamais le nœud principal. Chaque appareil supplémentaire (mobile, second desktop, navigateur) reçoit une **sous-clé dérivée**, signée par la clé maître et révocable individuellement.

```
Nœud principal
├── clé privée maître  (jamais exportée)
├── sous-clé appareil A  [mobile daniel]    ← révocable
├── sous-clé appareil B  [desktop bureau]   ← révocable
└── sous-clé appareil C  [laptop perso]     ← révocable
```

La compromission d'un appareil ne compromet pas l'identité complète — seule la sous-clé de cet appareil est révoquée.

#### Pairing d'un nouvel appareil

```
Nœud principal :
  1. Génère un code de pairing à usage unique (TTL : 5 min)
  2. Affiche un QR code

Nouvel appareil :
  3. Scanne le QR code
  4. Reçoit la sous-clé dérivée, chiffrée en transit
  5. La sous-clé est signée par la clé maître → vérifiable par tous
  6. Stockée localement sur l'appareil, chiffrée au repos
```

#### Comportement hors-ligne

Quand le nœud principal est inaccessible, les appareils appairés passent en **mode lecture seule** depuis leur cache local chiffré. Les actions en attente (messages envoyés, modifications) sont stockées dans une queue locale et synchronisées via CRDT à la reconnexion.

```
Nœud principal hors-ligne :
  → Lecture  : cache local chiffré (données récentes disponibles)
  → Écriture : mise en queue locale
  → Reconnexion : sync CRDT automatique, résolution des conflits
```

### Nom affiché

À la création du compte, le membre choisit un **nom affiché** — son pseudo lisible. Ce nom est unique au sein de chaque réseau qu'il rejoint, mais pas garanti globalement : deux membres dans deux réseaux différents peuvent avoir le même pseudo. La recherche inter-réseaux se fait par CID ; la recherche humaine dans un annuaire se fait par nom affiché.

```
Nom affiché :  daniel   (unique dans chaque réseau, pas globalement)
CID          :  civium:a3f9c2...e71b   (unique globalement)
```

### Identifiant réseau

Lorsqu'un membre rejoint un réseau, son identifiant dans ce réseau est formé à partir de son CID membre et du CID du réseau :

```
<cid_membre_court>@<cid_réseau_court>

a3f9e71b@b4e2f91a
```

Cet identifiant est :
- **Stable** : basé sur des CID cryptographiques, il ne change jamais — ni si le membre change de nœud, ni si le réseau se renomme
- **Non ambigu** : deux réseaux portant le même nom public n'ont pas le même CID
- **Vérifiable** : signé par la clé du membre, impossible à usurper

Le nom public du réseau est modifiable librement sans aucun impact sur les identifiants de ses membres.

### Nom affiché par réseau

L'identifiant réseau (`a3f9e71b@b4e2f91a`) est l'identifiant **protocolaire** — utilisé par le système. Ce que les autres membres voient est le **nom affiché**, choisi librement par le membre pour chaque réseau :

| Identifiant protocolaire | Nom affiché choisi | Contexte |
|---|---|---|
| `a3f9e71b@b4e2f91a` | Daniel R. | Dans l'asso vélo |
| `a3f9e71b@c7d3a02e` | Papa | Dans le réseau famille |
| `a3f9e71b@e91f4b17` | Le Vélociste | Dans la marketplace |

Le membre peut choisir son nom réel, un pseudonyme, ou un surnom — différent dans chaque réseau. L'identifiant protocolaire (`a3f9e71b@b4e2f91a`) reste interne au protocole et n'est jamais affiché à l'utilisateur final.

### Profil par réseau

Un membre peut avoir un **profil distinct dans chaque réseau** : nom affiché, photo, biographie, informations exposées. L'identité s'adapte au contexte.

```
Compte maître (CID : a3f9e71b...)
│
├── @b4e2f91a  [asso-velo]
│   └── nom affiché : "Daniel R.", bénévole, compétences logistique
│
├── @c7d3a02e  [famille-martin]
│   └── nom affiché : "Papa", photo de famille, agenda partagé
│
└── @e91f4b17  [equipe-design]
    └── nom affiché : "Dan", portfolio, compétences UI/UX, tarif journalier
```

### Visibilité : double contrôle

La visibilité d'un identifiant réseau est soumise à **deux niveaux de contrôle indépendants** :

**Niveau 1 — Politique du réseau**
Le réseau définit la visibilité par défaut de ses membres vers l'extérieur :

| Politique réseau | Effet |
|---|---|
| **Ouvert** | Les membres et leurs identifiants réseau sont visibles dans l'annuaire public |
| **Semi-ouvert** | Seul le nombre de membres est visible ; les identifiants sont masqués |
| **Fermé** | Aucune information sur les membres n'est visible de l'extérieur |

**Niveau 2 — Choix du membre**
Indépendamment de la politique du réseau, chaque membre définit **sa propre visibilité**. Il peut se rendre plus privé **ou plus public** que la politique par défaut du réseau :

```
Politique réseau : Semi-ouvert (identifiants masqués par défaut)
        │
        ▼
Membre daniel choisit :
  ├── Plus public   → s'affiche dans l'annuaire public malgré la politique semi-ouverte
  ├── Par défaut    → suit la politique du réseau
  └── Plus privé    → masqué même si le réseau est ouvert
```

Les deux niveaux sont **indépendants** — la politique du réseau définit le comportement par défaut, le membre peut s'en écarter librement dans les deux sens.

### Recherche et découverte

Un membre peut être trouvé :
- Par son **CID membre** (`civium:a3f9e71b...`) — recherche directe inter-réseaux
- Par son **nom affiché** — dans l'annuaire d'un réseau spécifique, si la politique le permet
- Via l'**annuaire du réseau** — si la politique du réseau le permet

### Appartenance à plusieurs réseaux

Un membre peut appartenir à autant de réseaux que souhaité. Ses différentes appartenances ne sont **pas liées publiquement** entre elles par défaut : connaître l'identifiant d'un membre dans un réseau ne révèle pas ses autres appartenances.

```
Vue publique (annuaire)            Vue du membre (compte maître)
                                   ┌──────────────────────────────────┐
a3f9e71b@b4e2f91a  ✓ visible       │ CID : a3f9e71b...                │
a3f9e71b@c7d3a02e  ✗ masqué        │ ├── @b4e2f91a [asso-velo, public]│
a3f9e71b@e91f4b17  ✗ masqué        │ ├── @c7d3a02e [famille,   privé] │
                                   │ └── @e91f4b17 [design,    privé] │
                                   └──────────────────────────────────┘
```

Le compte maître et la liste complète des appartenances ne sont visibles **que par le membre lui-même**.

---

### Profils enfants et contrôle parental

Dans un réseau famille, les parents (admins) peuvent créer des **profils enfants** — comptes membres avec un ensemble de restrictions configurables selon l'âge.

#### Création d'un profil enfant

Un profil enfant est un compte Civium standard avec un CID propre, placé sous supervision parentale dès sa création :

```
Parent (admin du réseau famille)
  └── crée un profil enfant
       ├── âge déclaré → restrictions par défaut selon tranche d'âge
       ├── compte lié au parent superviseur
       └── sous-clé dérivée de la clé maître du parent (révocable)
```

La sous-clé dérivée permet au parent de révoquer l'accès de l'enfant sans affecter son propre compte. Quand l'enfant atteint la majorité, le compte peut être émancipé — il devient un CID indépendant, non lié au parent.

#### Restrictions configurables

Les parents configurent les restrictions depuis le tableau de bord parental :

| Restriction | Description | Configurable |
|---|---|---|
| **Plugins autorisés** | Liste blanche des plugins accessibles à l'enfant | Par plugin |
| **Plages horaires** | Accès limité à certaines heures de la journée | Par jour / heure |
| **Connexions externes** | Toute demande de connexion hors du réseau famille requiert approbation parentale | Oui / Non |
| **Contacts directs** | Messages privés limités aux membres approuvés | Liste blanche |
| **Contenu adulte** | Masquage automatique du contenu marqué 18+ | Activé par défaut |
| **Visibilité annuaire** | L'enfant n'apparaît pas dans les annuaires publics | Activé par défaut |

#### Tranches d'âge et restrictions par défaut

| Âge | Restrictions par défaut |
|---|---|
| **< 13 ans** | Accès limité aux plugins famille uniquement, zéro connexion externe, contacts uniquement dans le réseau famille, pas d'annuaire |
| **13–15 ans** | Connexions externes soumises à approbation parentale, contenu adulte masqué, annuaire masqué par défaut |
| **16–17 ans** | Connexions externes autorisées avec notification parentale, contenu adulte masqué, annuaire optionnel |
| **18 ans+** | Émancipation possible — le compte devient indépendant si le membre le souhaite |

Les restrictions par défaut sont modifiables par le parent à tout moment, dans les deux sens.

#### Approbation parentale

Quand une action d'un enfant requiert validation :

```
Enfant demande une connexion avec un réseau extérieur
        │
        ▼
Notification envoyée au(x) parent(s) superviseur(s)
        │
        ├── Parent approuve → connexion établie
        ├── Parent refuse   → connexion bloquée, enfant notifié
        └── Pas de réponse après X heures → refus automatique (configurable)
```

Les deux parents peuvent être désignés superviseurs — l'approbation d'un seul suffit (configurable : un ou les deux).

#### Tableau de bord parental

Le plugin **Contrôle parental** (préinstallé dans tout réseau de type famille) fournit aux admins-parents :

- **Activité récente** : dernières connexions, plugins utilisés, heures d'accès — sans accès au contenu des messages (E2E préservé)
- **Demandes en attente** : connexions et contacts à approuver
- **Alertes** : tentatives d'accès à du contenu bloqué, connexions depuis un nouvel appareil
- **Gestion des restrictions** : modification en temps réel des plages horaires et plugins autorisés

Le contenu des messages privés (cercle 3, E2E) reste inaccessible aux parents — le contrôle parental porte sur les **métadonnées et les accès**, pas sur le contenu chiffré.

#### Émancipation du compte

À 18 ans (ou à la décision des parents avant), le profil enfant peut être **émancipé** :

```
1. Parent ou enfant (18 ans+) déclenche l'émancipation
2. Un nouveau CID indépendant est généré pour l'enfant
3. L'historique et les relations sont migrés vers le nouveau CID
4. Le lien de supervision est rompu — le compte n'est plus révocable par le parent
5. L'enfant choisit de rester dans le réseau famille ou d'en partir
```

---

## Fonctionnalités transversales

| Fonctionnalité | Description |
|---|---|
| Identité multi-contexte | Un compte maître, un profil adapté par réseau et par cercle |
| Chiffrement de bout en bout | Toutes les communications et données sensibles |
| Permissions granulaires | Contrôle fin sur chaque donnée partagée entre réseaux |
| Gouvernance configurable | Admin seul, collectif, ou mixte — au choix de chaque réseau |
| Annuaires hiérarchisables | Public, semi-public, privé — fédérables entre eux |
| Interopérabilité | ActivityPub prévu (roadmap) — interop Mastodon, PeerTube, etc. |
| Export total | Export de toutes les données à tout moment (JSON, CSV, SQLite, ZIP) |
| Hors-ligne | Fonctions de base accessibles sans connexion |
| Contrôle parental | Profils enfants avec restrictions par âge, approbation parentale, tableau de bord |

---

## Applications

### Vue d'ensemble

Civium est accessible via **quatre types d'applications**, chacune adaptée à un contexte d'usage et à un niveau de capacité nœud différent.

```
┌─────────────────────────────────────────────────────────────┐
│                    Protocole Civium (CIL)                    │
├──────────────┬──────────────┬──────────────┬────────────────┤
│   Desktop    │    Mobile    │     Web      │      CLI       │
│  Nœud complet│  Nœud léger  │  Nœud distant│  Nœud serveur  │
│  Windows     │  iOS         │  Navigateur  │  Linux/macOS   │
│  macOS       │  Android     │  PWA         │  Windows       │
│  Linux       │              │              │                │
└──────────────┴──────────────┴──────────────┴────────────────┘
```

### Application Desktop

**Rôle :** nœud complet — héberge un ou plusieurs réseaux Civium localement, fonctionne entièrement hors-ligne, synchronise en P2P.

### Application Mobile

**Rôle :** nœud léger — accès à tous les réseaux du membre, hors-ligne (cache local chiffré), synchronise via P2P ou via le nœud maître, notifications push.

### Application Web

**Rôle :** client distant — se connecte à un nœud Civium existant via navigateur. PWA avec cache partiel hors-ligne. WebRTC pour les communications P2P directes en session.

### Interface CLI

**Rôle :** nœud serveur — gestion headless d'un réseau Civium en production (VPS, NAS, Raspberry Pi). Scripting et automatisation.

```bash
civium node start                        # démarre le nœud
civium network create --name "mon-asso"  # crée un réseau
civium network connect --cid civium:...  # connecte à un réseau
civium member invite --email ...         # invite un membre
civium service install marketplace       # installe un service
civium audit log --last 7d               # journal des 7 derniers jours
civium backup export --encrypted         # sauvegarde chiffrée
```

### Comparatif des applications

| Capacité | Desktop | Mobile | Web | CLI |
|---|:---:|:---:|:---:|:---:|
| Hébergement de réseau | ✓ | — | — | ✓ |
| Nœud P2P complet | ✓ | partiel | — | ✓ |
| Hors-ligne complet | ✓ | ✓ | partiel | ✓ |
| Interface graphique | ✓ | ✓ | ✓ | — |
| Notifications push | ✓ | ✓ | ✓ | — |
| Administration réseau | ✓ | partiel | partiel | ✓ |
| Scripting / automatisation | — | — | — | ✓ |
| Installation sans app store | ✓ | — | ✓ (PWA) | ✓ |

Pour les détails de stack technique (Tauri, PHP F3, React Native vs Flutter, Rust FFI, infrastructure Scaleway), voir [STACK.md](STACK.md).

---

## Ce que Civium n'est pas

- Pas une plateforme centralisée détenant vos données
- Pas un réseau publicitaire
- Pas un algorithme de recommandation opaque
- Pas un silo fermé : chaque réseau reste maître de ses connexions

---

## MVP — Produit Minimum Viable

### Objectif du MVP

Valider les deux hypothèses fondamentales de Civium :
1. Le modèle de connexion inter-réseaux est compréhensible et utilisable par de vraies personnes
2. La souveraineté des données est réelle et tangible — tout tourne en local, rien ne passe par un tiers

### Périmètre

**Inclus**

| Fonctionnalité | Détail |
|---|---|
| Création d'un Réseau Civium | Nœud local, CLI + Desktop (macOS/Linux/Windows) |
| Identité membre | Identifiant réseau (`a3f9e71b@b4e2f91a`) + nom affiché choisi librement |
| Invitation de membres | Par lien ou CID, validation par l'admin |
| Cercles de confiance | Cercles 0, 1 et 2 uniquement |
| Messagerie chiffrée | E2E entre membres d'un même réseau |
| Connexion inter-réseaux | Demande → validation → acceptation ou refus |
| Partage de données basique | Annuaire membres et fil d'activité entre réseaux connectés |
| Permissions de partage | Par catégorie, configurable par réseau |
| Exclusion d'un membre | Par l'admin uniquement (exclusion totale) |
| Adressage hybride | Direct (IP/URL) + P2P (CID via DHT) |

**Exclu du MVP**

| Fonctionnalité | Raison |
|---|---|
| Gouvernance par vote | Complexité, l'admin seul suffit pour valider le concept |
| Plugins / API / SaaS / MCP | Après validation du protocole de base |
| Annuaires | Phase suivante |
| Fédération ActivityPub | Phase suivante |
| Cercle 3 (pair E2E) | Simplifié à 3 cercles pour le MVP |
| Récupération sociale | Phrase de récupération uniquement en MVP |
| Applications mobile / web | Desktop + CLI en priorité |
| Services avancés | Messagerie seule suffit à valider |
| Garde-fou majoritaire | Admin seul pour le MVP |

### Scénario de validation du MVP

```
Alice crée un Réseau Civium "asso-velo" sur son laptop
  └── installe le nœud (CLI ou Desktop)
  └── CID Alice : civium:a3f9...  CID réseau : civium:b4e2...
  └── identifiant réseau : a3f9e71b@b4e2f91a — nom affiché : "Alice"
  └── invite Bob → identifiant réseau : c7d3a02e@b4e2f91a — nom affiché : "Bob"

Bob crée son propre Réseau Civium "quartier-sud"
  └── CID réseau : civium:e91f...
  └── identifiant réseau de Bob : c7d3a02e@e91f4b17

Alice demande une connexion entre "asso-velo" et "quartier-sud"
  └── Bob valide la demande
  └── Alice configure : annuaire membres visible, messages non partagés

Alice et Bob voient les membres de l'autre réseau
Alice envoie un message à c7d3a02e@e91f4b17 (Bob dans quartier-sud) → chiffré E2E → Bob le reçoit

Bob refuse une deuxième demande de connexion d'un réseau inconnu
  └── le réseau demandeur est bloqué

✓ Souveraineté : aucune donnée n'a quitté les laptops d'Alice et Bob
✓ Interconnexion : deux réseaux distincts ont collaboré avec des permissions explicites
```

### Stack technique MVP

| Composant | Technologie | Justification |
|---|---|---|
| Protocole core | Rust | Performance, sécurité mémoire, compilation cross-platform |
| Transport P2P | libp2p (Rust) | DHT, NAT traversal, chiffrement Noise intégré |
| Sync données | CRDT (automerge-rs) | Offline-first, pas de serveur central |
| Stockage | SQLite chiffré (SQLCipher) | Léger, embarqué, chiffrement au repos |
| Desktop | Tauri (Rust + WebView) | Léger, cross-platform, UI web |
| Interface MVP | React + Tailwind | Développement rapide, composants réutilisables |
| CLI | Clap (Rust) | Robuste, documentation auto-générée |

### Plan de développement MVP

```
Semaine 1-2 — Protocole de base
  ├── Génération de clés Ed25519 + CID
  ├── Création d'un nœud local
  ├── Transport libp2p (TCP + QUIC)
  └── Découverte P2P via DHT Kademlia

Semaine 3-4 — Identité et membres
  ├── Création de compte (CID + clé Ed25519)
  ├── Format identifiant réseau (CID_membre_court@CID_réseau_court)
  ├── Invitation et admission de membres
  └── Cercles de confiance (0, 1, 2)

Semaine 5-6 — Messagerie
  ├── Messages directs E2E (chiffrement applicatif)
  ├── Fils de discussion dans le réseau
  └── Synchronisation CRDT entre membres

Semaine 7-8 — Connexion inter-réseaux
  ├── Handshake de connexion (CONNECT_REQUEST / RESPONSE)
  ├── Validation par l'admin
  ├── Refus et blocage
  ├── Accord de partage (APC) signé
  └── Partage d'annuaire entre réseaux connectés

Semaine 9-10 — Interface Desktop + CLI
  ├── Application Tauri (interface de base)
  ├── CLI Civium (commandes essentielles)
  ├── Onboarding (création de compte + réseau)
  └── Tests de bout en bout

Semaine 11-12 — Stabilisation et test terrain
  ├── Tests avec de vrais utilisateurs (2-3 réseaux pilotes)
  ├── Corrections et ajustements
  ├── Documentation utilisateur
  └── Publication du protocole v0.1
```

### Critères de succès du MVP

- [x] Deux réseaux indépendants peuvent se connecter sans serveur central — P2P libp2p (TCP/QUIC/WebSocket), APC signé
- [x] Un réseau peut refuser ou bloquer une connexion — `JoinRejected`, blacklist DHT
- [x] Les messages sont chiffrés et illisibles hors des nœuds destinataires — clé de groupe partagée, Noise via libp2p
- [x] Le nœud fonctionne hors-ligne et se resynchronise à la reconnexion — `outbox_queue` SQLite, sync-on-reconnect
- [ ] Un utilisateur non technique peut créer un réseau et inviter un membre en moins de 5 minutes — à tester avec de vrais utilisateurs
- [x] Aucune donnée ne transite par un serveur Civium central — par construction (P2P + SQLite local)

---

## Feuille de route

### Phase 0 — MVP ✅
Voir section MVP ci-dessus. Critères principaux atteints — voir [ROADMAP.md](ROADMAP.md) pour le détail.

### Phase 1 — Gouvernance & Annuaires ✅
- [x] Votes collectifs et quorum — propositions, options, résultats
- [x] Garde-fou majoritaire — suspension automatique + vote déclenché
- [x] Délégation de vote
- [x] Annuaire de réseaux et de membres (Registre Central Civium — RCC)
- [x] Fédération d'annuaires
- [ ] Profils enfants et contrôle parental

### Phase 2 — Services & Intégrations ✅
- [x] API plugin complète (manifest, CIL, sandbox WASM, hooks)
- [x] Plugins préinstallés : agenda, documents, fil d'activité, messagerie, notifications
- [x] Serveur MCP (JSON-RPC 2.0, Bearer token, CIL appliqué)
- [ ] Connecteurs SaaS (Google Calendar, Stripe...) — hors scope actuel
- [ ] Webhooks et API externe — hors scope actuel

### Phase 3 — Applications & Écosystème ✅
- [x] Application mobile — FFI Rust (`civium-ffi`, uniffi-rs), React Native
- [x] Application web — client WASM + PHP F3 + Alpine.js, magic link
- [x] Interopérabilité ActivityPub — publication et réception de posts, followers
- [x] Registre de Services Civium (RSC) — annuaire des services certifiés
- [x] Pairing multi-appareils (ChaCha20-Poly1305 + BLAKE3, deep link `civium://pair/<b58>`)
- [x] Mode hors-ligne avec resynchronisation

### Phase 4 — Maturité ✅
- [x] Programme de certification des plugins (Uncertified / Minimal / RSC / Certifié)
- [x] Transport WebSocket desktop (connexion clients web → nœuds desktop)
- [x] Alertes fraude RCC (vérification Ed25519 + diffusion P2P)
- [x] SDK Civium pour intégrateurs tiers (`civium-sdk`)
- [ ] Audit de sécurité externe
- [ ] Modèle économique et gouvernance du projet Civium

---

## Glossaire

| Terme | Définition |
|---|---|
| **APC** | Accord de Partage Civium — contrat signé cryptographiquement entre deux réseaux, listant exactement ce qui est partagé et à quel niveau d'accès |
| **CID** | Civium Identifier — identifiant unique dérivé d'une clé publique Ed25519. Stable et global pour les membres et les réseaux, indépendant de l'adresse réseau |
| **CIL** | Civium Integration Layer — couche système qui intercepte et contrôle tous les accès aux données (plugins, API, MCP, inter-réseaux). Plugin système non retirable |
| **CP** | Protocole Civium — couche applicative propre à Civium (couche 4 de la pile) |
| **CRDT** | Conflict-free Replicated Data Type — structure de données permettant la synchronisation sans serveur central et la résolution automatique de conflits |
| **DHT** | Distributed Hash Table — table de hachage distribuée utilisée pour la découverte de pairs et la résolution CID → adresse, sans serveur central |
| **E2E** | Chiffrement de bout en bout — seuls les destinataires désignés peuvent déchiffrer ; ni les nœuds relais ni les admins ne peuvent lire |
| **Gouvernance** | Plugin système gérant les votes, décisions collectives et règles de fonctionnement d'un réseau. Non retirable |
| **Nœud** | Instance du protocole Civium hébergeant un ou plusieurs réseaux. Peut être un desktop, un serveur, un NAS, un Raspberry Pi |
| **Nœud principal** | Nœud d'un membre où réside sa clé privée maître. Les autres appareils reçoivent des sous-clés dérivées |
| **RRM** | Registre des Réseaux Malveillants — type spécialisé d'annuaire listant les réseaux au comportement malveillant avéré |
| **RRM-LEA** | RRM Law Enforcement Authorities — registre spécialisé pour la coopération judiciaire, accessible aux forces de l'ordre accréditées |
| **RSC** | Registre de Services Civium — catalogue décentralisé des plugins et services disponibles |
| **Sous-clé** | Clé dérivée de la clé maître, attribuée à un appareil spécifique. Révocable individuellement sans affecter l'identité principale |
| **WASM** | WebAssembly — format d'exécution sandboxé utilisé pour les plugins Civium |

---

*Civium — Des réseaux souverains, connectés par choix.*
