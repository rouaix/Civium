# Vision Civium

Ce document décrit l'ambition à long terme de Civium. Il est séparé de la spec technique (README.md) car il s'adresse autant aux futurs contributeurs, partenaires et utilisateurs qu'aux développeurs.

---

## Objectif principal

> **Remplacer toutes les grandes applications centralisées — WhatsApp, Facebook, TikTok, YouTube, WeChat, Slack, Notion, Instagram et les autres — par un protocole unique, souverain, où chaque communauté garde le contrôle total de ses données, de ses règles et de ses connexions.**

Ce n'est pas une ambition partielle. Civium n'est pas un outil parmi d'autres, ni une niche pour les technophiles militants. C'est une infrastructure de remplacement — conçue pour être aussi riche, aussi accessible et plus puissante que ce qu'elle remplace, avec un avantage que les plateformes centralisées ne peuvent pas offrir : **le contrôle**.

**Le contrôle, c'est :**
- Vos données sur votre nœud — personne d'autre n'y a accès sans votre permission explicite
- Vos règles — chaque communauté définit sa gouvernance, ses cercles de confiance, ses connexions
- Votre identité — un CID cryptographique qui vous appartient, portable entre tous les réseaux
- Votre écosystème — vous choisissez vos plugins, vous ne subissez pas les fonctionnalités imposées
- Votre indépendance — si Civium disparaît demain, votre nœud continue de fonctionner

Les plateformes centralisées vous offrent des outils en échange de vos données, de votre attention et de votre dépendance. **Civium vous offre les mêmes outils sans cette contrepartie.**

---

## Le problème que Civium résout

Les outils numériques actuels imposent un choix binaire :

**Option A — Les plateformes centralisées (WhatsApp, WeChat, Slack, Facebook Groups)**
Riches en fonctionnalités, mais vos données leur appartiennent. Votre communauté vit sur leur infrastructure, selon leurs règles, avec leur algorithme. Si la plateforme disparaît, change de politique ou est rachetée, votre communauté disparaît avec elle.

**Option B — Les outils auto-hébergés (Nextcloud, Mattermost, Discourse)**
Souverains, mais cloisonnés. Chaque outil est une île. Pas d'interopérabilité, pas de gouvernance collective, pas de connexion entre communautés.

**Civium est la troisième voie** : toutes les fonctionnalités des plateformes centralisées, avec la souveraineté des outils auto-hébergés, et l'interopérabilité que ni l'un ni l'autre ne propose.

---

## L'ambition : remplacer ou se connecter aux grandes plateformes

Civium a vocation à devenir une **alternative souveraine** aux grandes plateformes centralisées — et, pour les communautés qui ne veulent pas rompre totalement avec elles, un **pont vers ces plateformes**.

### Deux modes de relation avec les plateformes existantes

```
Mode 1 — Remplacement
  La communauté quitte la plateforme et migre sur Civium.
  Ses données, sa gouvernance, ses interactions : 100 % souveraines.

Mode 2 — Pont (bridge)
  La communauté utilise Civium en interne et reste visible
  sur les plateformes externes pour toucher une audience plus large.
  Civium est la source de vérité. Les plateformes sont des miroirs.
```

### Civium face à chaque grande plateforme

#### Facebook / Meta

Facebook Groups, Pages, Events, Marketplace, Messenger — Civium couvre nativement tout cela :

| Fonctionnalité Facebook | Équivalent Civium |
|---|---|
| Groupes | Réseau Civium (avec gouvernance réelle) |
| Pages (organisations) | Profil annuaire public d'un réseau |
| Événements | Plugin Agenda |
| Marketplace | Plugin Marketplace + commission 1% |
| Messenger | Plugin Messagerie E2E |
| Fil d'actualité | Plugin Fil d'activité (sans algorithme) |
| Connexion entre groupes | Connexion inter-réseaux (APC) |

**Pont Facebook :** un plugin connecteur publie automatiquement le contenu public d'un réseau Civium (cercle 0) sur une Page Facebook — pour toucher les membres qui n'ont pas encore migré.

---

#### TikTok

TikTok est une machine à contenu court, algorithmique, conçue pour la dépendance. Civium propose une alternative sans algorithme de manipulation :

| Fonctionnalité TikTok | Équivalent Civium |
|---|---|
| Vidéos courtes | Plugin Vidéo (hébergé sur le nœud du créateur) |
| Fil de contenu | Fil d'activité curé par la communauté, pas par un algorithme |
| Abonnements | Connexion inter-réseaux (cercle 1) |
| Commentaires | Réactions et fils dans le plugin Fil d'activité |
| Monétisation créateur | Transactions directes membres → créateur (1% Civium) |
| Live streaming | Plugin Visioconférence / Streaming (RSC) |

**Pont TikTok :** un plugin publie automatiquement les vidéos publiques d'un réseau sur TikTok et YouTube simultanément. Le créateur reste propriétaire du contenu sur son nœud Civium.

**Différence fondamentale :** sur TikTok, l'algorithme décide qui voit quoi. Sur Civium, chaque communauté définit ses propres règles de diffusion — ou n'en a pas.

---

#### YouTube

YouTube est la référence mondiale pour la vidéo. Civium ne cherche pas à remplacer YouTube pour la diffusion massive — mais à offrir un espace souverain pour les créateurs et communautés :

| Fonctionnalité YouTube | Équivalent Civium |
|---|---|
| Hébergement vidéo | Plugin Vidéo (nœud propre ou hébergeur tiers) |
| Chaînes | Réseau Civium d'un créateur ou d'une organisation |
| Abonnés | Membres du réseau (cercle 1) |
| Commentaires | Fil d'activité |
| Monétisation | Transactions directes, plugin Marketplace |
| Playlists | Collections dans le plugin Bibliothèque |
| Lives | Plugin Streaming (RSC) |

**Pont YouTube :** le plugin connecteur SaaS YouTube synchronise les vidéos publiques (cercle 0) vers YouTube. Le créateur publie une fois sur Civium, la vidéo apparaît sur YouTube automatiquement. Les revenus YouTube restent au créateur — Civium ne prend rien dessus.

**Cas d'usage clé :** une chaîne YouTube éducative migre sur Civium pour héberger ses cours privés (payants, cercle 2), garder YouTube pour la visibilité publique. Les abonnés payants sont dans son réseau Civium — leurs données ne sont pas chez Google.

---

#### WeChat

WeChat est l'exemple le plus avancé de super-app au monde : messagerie, paiements, réseau social, services gouvernementaux, santé, transport, e-commerce — tout centralisé chez Tencent, sous contrôle chinois, sans souveraineté des données.

Civium ne cherche pas à copier WeChat. Il cherche à rendre **possible ce que WeChat ne peut pas faire** :

| Dimension | WeChat | Civium |
|---|---|---|
| **Gouvernance** | Tencent décide | Chaque communauté décide |
| **Données** | Centralisées chez Tencent | Sur le nœud de chaque communauté |
| **Services** | Imposés par la plateforme | Choisis et gouvernés par chaque réseau |
| **Connexions** | Inexistantes entre groupes | Contractualisées (APC) |
| **Identité** | Liée à WeChat | Portable, cryptographique, universelle |
| **Juridiction** | Droit chinois | Droit local de chaque opérateur |
| **Algorithme** | Opaque, optimisé pour l'engagement | Absent — pas de recommandation |
| **Interopérabilité** | Fermée | Ouverte (ActivityPub, MCP, API) |

WeChat est une super-app. **Civium est un super-protocole** : chaque communauté y assemble sa propre super-app depuis un écosystème ouvert, sans dépendre d'une seule entreprise.

---

### Tableau de synthèse

| Plateforme | Civium peut remplacer | Civium peut s'y connecter |
|---|---|---|
| **Facebook Groups** | ✓ Complètement | ✓ Pont publication |
| **WhatsApp** | ✓ Complètement (E2E natif) | — |
| **WeChat** | ✓ Complètement + davantage | Pont publication |
| **TikTok** | ✓ Pour les communautés | ✓ Pont vidéo |
| **YouTube** | Partiellement (privé/payant) | ✓ Pont vidéo fort |
| **Slack / Teams** | ✓ Pour les organisations | ✓ Connecteur SaaS |
| **Notion / Confluence** | ✓ Plugin Documents + Wiki | ✓ Connecteur SaaS |
| **Eventbrite** | ✓ Plugin Agenda + billetterie | ✓ Connecteur SaaS |
| **Meetup** | ✓ Annuaire + événements | ✓ Pont publication |
| **Instagram** | Partiellement (galerie, communauté) | ✓ Pont publication |
| **LinkedIn** (groupes) | ✓ Pour les réseaux pro | ✓ Connecteur SaaS |

---

## Ce que Civium peut devenir

### Pour une famille

Un espace où coexistent : album photo partagé, agenda familial, coffre-fort de documents (actes, contrats, testaments), messagerie E2E, caisse commune, suivi médical partagé (ordonnances, carnet de santé), liste de courses collaborative — tout en local, sans Google Photos, sans WhatsApp, sans Dropbox.

### Pour une association

Gestion des membres, votes, agenda, communication interne, comptabilité, appels à projets, marketplace de services entre membres, connexion avec d'autres associations partenaires, interface publique pour les non-membres.

### Pour un quartier

Annuaire de voisinage, troc et dons, signalement de problèmes urbains, concertation citoyenne, événements locaux, covoiturage, bibliothèque partagée, connexion au réseau de la mairie.

### Pour une entreprise

Gestion de projets, documents partagés, facturation, RH, communication interne, connexion sécurisée avec des prestataires externes — chaque connexion contractualisée, chaque accès audité.

### Pour une institution (mairie, école, hôpital)

Démarches administratives, services aux citoyens, communication officielle, vote participatif, gestion de dossiers — sur une infrastructure souveraine, conforme RGPD, sans dépendre d'AWS ou de Google.

### Pour un pays ou une région

Un réseau de réseaux à l'échelle territoriale : chaque commune, chaque association, chaque entreprise locale connectée selon des règles négociées — une infrastructure numérique souveraine de niveau national.

---

## Les couches de la vision

```
┌─────────────────────────────────────────────────────────────────┐
│  Niveau 3 — Civilisation numérique souveraine                   │
│  Réseaux de réseaux à l'échelle d'un territoire ou d'un pays   │
├─────────────────────────────────────────────────────────────────┤
│  Niveau 2 — Super-app souveraine par communauté                 │
│  Chaque réseau assemble ses services depuis le RSC              │
│  (messagerie, paiements, santé, gouvernance, e-commerce...)     │
├─────────────────────────────────────────────────────────────────┤
│  Niveau 1 — Protocole de base (MVP → v1)                        │
│  Identité, cercles de confiance, connexions inter-réseaux,      │
│  gouvernance, plugins, sécurité                                 │
└─────────────────────────────────────────────────────────────────┘
```

Civium construit de bas en haut. Le niveau 1 est la fondation. Les niveaux 2 et 3 émergent quand l'écosystème de plugins grandit.

---

## Les services que Civium peut héberger à terme

Tout service aujourd'hui centralisé peut devenir un plugin Civium souverain :

### Communication
- Messagerie instantanée E2E
- Visioconférence P2P
- Forums et fils de discussion
- Newsletters internes
- Notifications push

### Organisation
- Agenda partagé
- Gestion de projets et tâches
- Documents collaboratifs
- Wiki communautaire
- Sondages et votes

### Finance
- Paiements entre membres (marketplace)
- Caisse commune / trésorerie associative
- Facturation et devis
- Gestion des cotisations
- Comptabilité simplifiée

### Vie quotidienne
- Troc et dons entre membres
- Covoiturage interne
- Bibliothèque de ressources (prêt)
- Petites annonces
- Réservation de ressources partagées (salles, matériel)

### Santé et bien-être *(données ultra-sensibles — cercle 3)*
- Carnet de santé partagé en famille
- Suivi de traitement
- Agenda médical
- Partage d'ordonnances avec un médecin de confiance

### Services civiques
- Concertation citoyenne et budgets participatifs
- Signalement et suivi de problèmes urbains
- Démarches administratives (connexion institutions)
- Vote électronique sécurisé

### Commerce et économie locale
- Marketplace de produits et services
- Annuaires professionnels
- Programmes de fidélité entre réseaux
- Monnaie locale ou système de points

### Intelligence artificielle
- Agent IA de réseau (accès via MCP aux données du réseau)
- Résumé automatique de votes et décisions
- Recherche sémantique dans les documents
- Modération assistée

---

## Ce que Civium ne sera jamais

Même à son niveau d'ambition maximal, Civium ne sera jamais :

- **Un réseau social à algorithme** — pas de fil d'actualité optimisé pour l'engagement, pas de publicité ciblée
- **Une plateforme propriétaire** — le protocole est ouvert, l'écosystème n'appartient à personne
- **Un service centralisé** — Civium n'héberge pas vos données, il fournit le protocole pour que vous les hébergiez vous-même
- **Un outil de surveillance** — pas d'analytics comportementales, pas de profilage, pas de vente de données
- **Un monopole** — si Civium échoue ou déçoit, n'importe qui peut forker le protocole

---

## Horizon temporel

| Horizon | Ambition |
|---|---|
| **Court terme (1–2 ans)** | Protocole de base stable, premiers réseaux pilotes, écosystème de plugins naissant |
| **Moyen terme (3–5 ans)** | Super-app souveraine accessible à toute communauté, RSC riche, connexions inter-réseaux courantes |
| **Long terme (5–10 ans)** | Infrastructure numérique de référence pour les communautés souveraines — alternative crédible aux plateformes centralisées à l'échelle européenne |
| **Très long terme** | Protocole de couche civique : des villes, des régions, des États s'appuient sur Civium pour leurs services numériques souverains |

---

## Pourquoi maintenant

Les conditions sont réunies :

- **Technologie** : CRDT, libp2p, WASM, Ed25519, MCP — les briques existent et sont matures
- **Réglementation** : RGPD, DSA, DMA — l'Europe pousse vers la souveraineté numérique
- **Sentiment** : la méfiance envers les GAFAM est à son comble — les alternatives souveraines trouvent leur public
- **Financement** : NLnet, NGI, DINUM — les fonds publics cherchent exactement ce que Civium construit

Le moment pour construire un protocole de cette ambition n'a jamais été aussi favorable.

---

*Civium — Des réseaux souverains, connectés par choix. Une infrastructure pour les communautés humaines.*
