# Vision Civium

Ce document décrit l'ambition à long terme de Civium. Il est séparé de la spec technique (README.md) car il s'adresse autant aux futurs contributeurs, partenaires et utilisateurs qu'aux développeurs.

---

## Le problème que Civium résout

Les outils numériques actuels imposent un choix binaire :

**Option A — Les plateformes centralisées (WhatsApp, WeChat, Slack, Facebook Groups)**
Riches en fonctionnalités, mais vos données leur appartiennent. Votre communauté vit sur leur infrastructure, selon leurs règles, avec leur algorithme. Si la plateforme disparaît, change de politique ou est rachetée, votre communauté disparaît avec elle.

**Option B — Les outils auto-hébergés (Nextcloud, Mattermost, Discourse)**
Souverains, mais cloisonnés. Chaque outil est une île. Pas d'interopérabilité, pas de gouvernance collective, pas de connexion entre communautés.

**Civium est une troisième voie** : des communautés souveraines, interconnectées par choix, avec des outils aussi riches que les plateformes centralisées — sans en subir les contreparties.

---

## L'ambition : dépasser WeChat, mais autrement

WeChat est l'exemple le plus avancé de super-app au monde : messagerie, paiements, réseau social, services gouvernementaux, santé, transport, e-commerce — tout dans une seule application, utilisée par plus d'un milliard de personnes.

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
