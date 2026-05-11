# CIVIUM
### Cadre d'Interconnexion et de Validation des Intentions et des Usages des Membres

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

La confiance est **asymétrique par défaut** : je peux te faire confiance au niveau 2 pendant que tu me fais confiance au niveau 1. Chacun est libre de placer l'autre dans le cercle qu'il juge approprié, indépendamment.

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
daniel_asso-velo  ──[cercle 2]──  sophie_asso-velo
         │                                │
         │  daniel rejoint equipe-design  │
         ▼                                │
daniel_equipe-design                      │
  └── reconnaît sophie_equipe-design  ────┘
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
[Vote]  ← durée configurable (ex : 24h, 7 jours)
     │
     ├── Quorum atteint ?
     │     ├── Non → Décision caduque ou prolongation
     │     └── Oui → Dépouillement
     │
     ├── Résultat
     │     ├── Adopté  → Application automatique ou manuelle
     │     └── Rejeté  → Archivé, nouvelle proposition possible après délai
     │
     └── [Audit] ← décision enregistrée, immuable, consultable
```

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
Proposant :    sophie_asso-velo
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

Le **protocole Civium** maintient un **annuaire racine décentralisé** — non contrôlé par une entité centrale — servant de point d'entrée minimal pour la découverte de réseaux et d'annuaires publics. Il fonctionne via le maillage P2P (DHT) et ne stocke que les CID et métadonnées minimales des réseaux qui choisissent d'y figurer.

```
Annuaire racine Civium (DHT)
├── Liste des annuaires publics
├── Liste des réseaux publics
└── Points d'entrée pour rejoindre le maillage P2P
```

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
| Réseau malveillant | Validation explicite avant toute connexion |
| Admin abusif | Garde-fou majoritaire, journal immuable |
| Fuite de métadonnées | Minimisation des données exposées par défaut |
| Attaque sur le nœud | Chiffrement au repos, clés locales |
| Déni de service | Architecture P2P sans point central d'attaque |

### Chiffrement

#### En transit
Toutes les communications entre nœuds sont chiffrées via le **Noise Protocol** (intégré dans libp2p), indépendamment du transport utilisé (TCP, QUIC, WebRTC).

```
Nœud A  ──[Noise Protocol / TLS 1.3]──  Nœud B
         chiffrement de bout en bout
         authentification mutuelle des nœuds
```

#### De bout en bout (E2E)
Les messages privés et les données sensibles sont chiffrés **au niveau applicatif**, en plus du chiffrement de transport. Seuls les destinataires autorisés peuvent déchiffrer.

```
Message de daniel → chiffré avec la clé publique de sophie
                  → seule sophie (clé privée) peut lire
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

Un membre peut **supprimer ses données** à tout moment :
- **Suppression partielle** : retrait de contenus spécifiques
- **Désinscription d'un réseau** : suppression du profil réseau, les contributions partagées suivent la politique du réseau
- **Suppression du compte** : effacement de la clé et des données locales — les données déjà partagées et répliquées chez d'autres membres suivent leurs propres politiques de rétention

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

### Divulgation responsable

Civium est un protocole ouvert. Les failles de sécurité peuvent être signalées via un processus de **divulgation responsable** défini dans la gouvernance du projet. Toute faille confirmée est communiquée à l'ensemble des opérateurs de nœuds avant publication.

---

## Cas d'usage

### Famille
La famille Martin crée un Réseau Civium privé. Ils y partagent un agenda, des photos, des documents (actes, contrats). Ils connectent leur réseau à celui de la famille Dupont (cousins) avec un accès partiel : agenda commun visible, mais documents privés inaccessibles.

### Association
L'association "Vélo Urbain" crée son réseau : annonces, événements, membres. Elle se connecte à l'annuaire des associations de sa ville, et établit une connexion totale avec l'association "Mobilité Douce" pour co-organiser des événements.

### Équipe professionnelle
Une agence de design crée un réseau pro interne. Elle partage son annuaire de compétences (mais pas ses projets) avec un réseau de freelances partenaires. Les nouvelles connexions sont validées par un collectif de 3 associés.

### Individu
Un membre individuel appartient à 3 réseaux : sa famille, son asso de quartier, et son réseau professionnel. Son profil s'adapte à chaque contexte (cercles de confiance). Il apparaît dans l'annuaire public Civium uniquement s'il le choisit.

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
│  Couche 3 — Fédération : ActivityPub             │
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

### Couche 3 — ActivityPub (fédération)
Civium implémente ActivityPub pour **l'interopérabilité** avec l'écosystème décentralisé existant (Mastodon, PeerTube, Pixelfed, etc.). Un Réseau Civium peut ainsi interagir avec des acteurs extérieurs à Civium sans abandonner ses propres règles de gouvernance.

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
- Est versionnée : toute modification requiert une nouvelle validation
- Peut être révoqué unilatéralement à tout moment (retour à l'état Suspendu puis Révoqué)

#### Modification des permissions en cours de connexion

Un réseau peut à tout moment **restreindre ou élargir** les données qu'il expose, sans rompre la connexion :
- **Restriction** : prise d'effet immédiate, les données retirées deviennent inaccessibles
- **Élargissement** : requiert une re-validation par le réseau partenaire avant prise d'effet

---

### Refus et blocage

Un réseau peut **refuser** ou **bloquer** une demande de connexion entrants :

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

### Principe : plateforme ouverte et extensible

Civium n'est pas un ensemble de fonctionnalités figées. C'est une **plateforme de services** : chaque réseau choisit les services qu'il installe, et n'importe qui peut développer et publier un nouveau service à tout moment.

```
┌─────────────────────────────────────────────────────┐
│               Réseau Civium                         │
│                                                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────┐  │
│  │Messagerie│ │  Agenda  │ │Marketplace│ │ ... + │  │
│  └──────────┘ └──────────┘ └──────────┘ └───────┘  │
│                                                     │
│         ← services installés par le réseau →        │
└─────────────────────────────────────────────────────┘
                        ↕
         Registre de Services Civium (RSC)
         (catalogue public de services disponibles)
```

### Services natifs (inclus par défaut)

Ces services font partie du cœur de Civium et sont disponibles dès la création d'un réseau :

| Service | Description |
|---|---|
| **Messagerie** | Messages directs et fils de discussion, chiffrés de bout en bout |
| **Agenda** | Calendrier partagé, événements, invitations |
| **Annuaire** | Gestion des membres et de leurs profils |
| **Documents** | Coffre-fort de fichiers, partage contrôlé |
| **Fil d'activité** | Actualités et publications au sein du réseau |
| **Gouvernance** | Votes, sondages, décisions collectives |
| **Notifications** | Alertes configurables par membre |

### Services étendus (exemples)

Services additionnels qui peuvent être installés selon les besoins du réseau :

| Service | Usage type |
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

### Identifiant initial

À la création de son compte, chaque membre choisit librement son **identifiant initial** — son pseudo Civium, unique sur l'ensemble du protocole.

```
Identifiant initial :  daniel
```

Cet identifiant est ancré sur une clé cryptographique Ed25519 générée lors de la création du compte. La clé garantit l'unicité et la souveraineté : personne d'autre ne peut revendiquer cet identifiant, et le membre en reste propriétaire même s'il change de nœud ou d'hébergement.

```
Compte maître
├── identifiant : daniel
├── clé privée  : Ed25519 (conservée localement, jamais transmise)
└── clé publique : diffusée pour vérification
```

### Identifiant réseau

Lorsqu'un membre rejoint un réseau, son identifiant dans ce réseau est formé automatiquement :

```
identifiant_initial + "_" + nom_public_du_réseau

daniel_famille-martin
daniel_asso-velo
daniel_equipe-design
```

Cet identifiant réseau est :
- **Stable** : il ne change pas tant que le membre appartient au réseau
- **Lisible** : il identifie immédiatement le membre et son réseau d'appartenance
- **Vérifiable** : signé par la clé du membre, impossible à usurper

### Profil par réseau

Un membre peut avoir un **profil distinct dans chaque réseau** : nom affiché, photo, biographie, informations exposées. L'identité s'adapte au contexte.

```
daniel (compte maître)
│
├── daniel_famille-martin
│   └── profil : "Papa", photo de famille, agenda partagé
│
├── daniel_asso-velo
│   └── profil : "Daniel R.", bénévole, compétences logistique
│
└── daniel_equipe-design
    └── profil : "Dan", portfolio, compétences UI/UX, tarif journalier
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

**Nom affiché : nom réel ou pseudonyme**
Pour chaque réseau, le membre choisit comment il souhaite être identifié publiquement :

| Option | Exemple affiché | Identifiant réseau |
|---|---|---|
| **Nom réel** | Daniel Rouaix | `daniel_asso-velo` |
| **Pseudonyme** | Dan_R | `daniel_asso-velo` |
| **Nom de réseau personnalisé** | Le Vélociste | `daniel_asso-velo` |

L'identifiant réseau (`daniel_asso-velo`) reste interne au protocole. Ce qui est affiché publiquement est le **nom choisi par le membre** pour ce réseau — réel, pseudo, ou surnom.

### Recherche et découverte

Un membre peut être trouvé :
- Par son **identifiant réseau complet** (`daniel_asso-velo`) — recherche directe
- Par son **identifiant initial** (`daniel`) — si au moins un de ses profils est public
- Via l'**annuaire du réseau** — si la politique du réseau le permet

### Appartenance à plusieurs réseaux

Un membre peut appartenir à autant de réseaux que souhaité. Ses différents identifiants réseau ne sont **pas liés publiquement** entre eux par défaut : connaître `daniel_asso-velo` ne révèle pas l'existence de `daniel_famille-martin`.

```
Vue publique (annuaire)       Vue du membre (compte maître)
                              ┌─────────────────────────────┐
daniel_asso-velo  ✓ visible   │ daniel                      │
daniel_famille-??  ✗ masqué   │ ├── _famille-martin  [privé]│
daniel_equipe-??   ✗ masqué   │ ├── _asso-velo      [public]│
                              │ └── _equipe-design  [privé] │
                              └─────────────────────────────┘
```

Le compte maître et la liste complète des appartenances ne sont visibles **que par le membre lui-même**.

---

## Fonctionnalités transversales

| Fonctionnalité | Description |
|---|---|
| Identité multi-contexte | Un compte maître, un profil adapté par réseau et par cercle |
| Chiffrement de bout en bout | Toutes les communications et données sensibles |
| Permissions granulaires | Contrôle fin sur chaque donnée partagée entre réseaux |
| Gouvernance configurable | Admin seul, collectif, ou mixte — au choix de chaque réseau |
| Annuaires hiérarchisables | Public, semi-public, privé — fédérables entre eux |
| Interopérabilité | Compatible ActivityPub, ouvert sur d'autres protocoles décentralisés |
| Export total | Export de toutes les données à tout moment, formats ouverts |
| Hors-ligne | Fonctions de base accessibles sans connexion |

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

**Rôle :** nœud complet — l'application embarque le protocole Civium et peut héberger un réseau.

**Capacités :**
- Héberge un ou plusieurs Réseaux Civium localement
- Fonctionne entièrement hors-ligne
- Synchronise en P2P avec les autres nœuds
- Accès à toutes les fonctionnalités et tous les services
- Gestion des clés cryptographiques en local

**Stack technique :** [Tauri](https://tauri.app) (Rust + WebView)
- Exécutable léger (< 10 Mo vs 150 Mo+ pour Electron)
- Interface web (React / Vue / Svelte) pour l'UI
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
│  │  └── Stockage chiffré local  │  │
│  └──────────────────────────────┘  │
└────────────────────────────────────┘
```

### Application Mobile

**Rôle :** nœud léger — synchronise avec un nœud complet (desktop, NAS, VPS) ou directement en P2P.

**Capacités :**
- Accès à tous les réseaux dont le membre fait partie
- Fonctionne hors-ligne (données en cache local chiffré)
- Synchronise en P2P ou via le nœud maître du membre
- Notifications push
- Toutes les fonctionnalités membres (pas d'hébergement de réseau)

**Stack technique :** React Native ou Flutter
- Base de code partagée iOS / Android
- Module natif Rust pour le protocole (via FFI)
- Stockage local chiffré (SQLite + clé dérivée)

**Gestion de la batterie et de la connectivité :**
- Synchronisation différée hors Wi-Fi (configurable)
- Mode ultra-léger en arrière-plan (notifications uniquement)
- Reconnexion automatique P2P à la reprise de connexion

### Application Web

**Rôle :** client distant — se connecte à un nœud Civium existant (desktop, NAS, VPS, instance mutualisée).

**Capacités :**
- Accès aux réseaux via un nœud distant authentifié
- Fonctionnement partiel hors-ligne (PWA avec cache)
- Toutes les fonctionnalités membres
- Pas d'hébergement de réseau (limitation navigateur)
- WebRTC pour les communications P2P directes en session

**Stack technique :** PHP Fat-Free Framework + Alpine.js

```
Navigateur
  │
  ├── Pages & routing ──────→ PHP Fat-Free Framework
  │   Templates, sessions,     (hébergement Scaleway existant)
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
- **Alpine.js** : 2 Ko, s'écrit dans les templates PHP sans étape de compilation, gère toute la réactivité UI nécessaire
- **Vanilla JS** pour le Service Worker (PWA) et les connexions WebSocket/WebRTC — aucune dépendance supplémentaire
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

### Interface CLI

**Rôle :** nœud serveur — gestion headless d'un réseau Civium en production.

**Capacités :**
- Installation et gestion d'un nœud sur serveur (VPS, NAS, Raspberry Pi)
- Administration complète via ligne de commande
- Scripting et automatisation
- Intégration dans des pipelines DevOps
- Monitoring et journalisation

**Stack technique :** Rust (binaire natif)

```bash
# Exemples de commandes CLI Civium
civium node start                        # démarre le nœud
civium network create --name "mon-asso"  # crée un réseau
civium network connect --cid civium:...  # connecte à un réseau
civium member invite --email ...         # invite un membre
civium service install marketplace       # installe un service
civium audit log --last 7d              # journal des 7 derniers jours
civium backup export --encrypted        # sauvegarde chiffrée
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

### Base de code partagée

Le **cœur du protocole Civium** est un module Rust unique, partagé entre toutes les applications :

```
civium-core (Rust)
├── utilisé par  Desktop  (Tauri — natif)
├── utilisé par  Mobile   (React Native / Flutter — via FFI)
├── utilisé par  CLI      (binaire natif)
└── compilé en  WASM     (pour usage futur dans le navigateur)
```

Une seule implémentation du protocole — pas de divergence entre plateformes.

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
| Identité membre | Identifiant initial + identifiant réseau (`daniel_mon-reseau`) |
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
| Cercle 3 + cautionnement | Simplifié à 3 cercles pour le MVP |
| Récupération sociale | Phrase de récupération uniquement en MVP |
| Applications mobile / web | Desktop + CLI en priorité |
| Services avancés | Messagerie seule suffit à valider |
| Garde-fou majoritaire | Admin seul pour le MVP |

### Scénario de validation du MVP

```
Alice crée un Réseau Civium "asso-velo" sur son laptop
  └── installe le nœud (CLI ou Desktop)
  └── génère son identifiant : alice_asso-velo
  └── invite Bob → bob_asso-velo rejoint le réseau

Bob crée son propre Réseau Civium "quartier-sud"
  └── génère son identifiant : bob_quartier-sud

Alice demande une connexion entre "asso-velo" et "quartier-sud"
  └── Bob valide la demande
  └── Alice configure : annuaire membres visible, messages non partagés

Alice et Bob voient les membres de l'autre réseau
Alice envoie un message à bob_quartier-sud → chiffré E2E → Bob le reçoit

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
  ├── Création de compte (identifiant + clé)
  ├── Format identifiant réseau (id_nom-reseau)
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

- [ ] Deux réseaux indépendants peuvent se connecter sans serveur central
- [ ] Un réseau peut refuser ou bloquer une connexion
- [ ] Les messages sont chiffrés et illisibles hors des nœuds destinataires
- [ ] Le nœud fonctionne hors-ligne et se resynchronise à la reconnexion
- [ ] Un utilisateur non technique peut créer un réseau et inviter un membre en moins de 5 minutes
- [ ] Aucune donnée ne transite par un serveur Civium central

---

## Feuille de route

### Phase 0 — MVP (3 mois)
Voir section MVP ci-dessus.

### Phase 1 — Gouvernance & Annuaires
- [ ] Votes collectifs et quorum
- [ ] Garde-fou majoritaire
- [ ] Délégation de vote
- [ ] Annuaire de réseaux et de membres
- [ ] Fédération d'annuaires

### Phase 2 — Services & Intégrations
- [ ] Architecture plugin (manifest, CIL)
- [ ] Services natifs : agenda, documents, fil d'activité
- [ ] Connecteurs SaaS (Google Calendar, Stripe...)
- [ ] Webhooks et API externe
- [ ] Serveur MCP

### Phase 3 — Applications & Écosystème
- [ ] Application mobile (iOS / Android)
- [ ] Application web (PWA)
- [ ] Interopérabilité ActivityPub
- [ ] Registre de Services Civium (RSC)
- [ ] Cercle 3 + récupération sociale

### Phase 4 — Maturité
- [ ] Modèle économique et gouvernance du projet Civium
- [ ] Programme de certification des plugins
- [ ] Audit de sécurité externe
- [ ] Documentation développeur complète
- [ ] SDK Civium (pour intégrateurs tiers)

---

*Civium — Des réseaux souverains, connectés par choix.*
