# Sécurité Civium

Ce document couvre les exigences de sécurité opérationnelles de Civium. Il complète la spec protocolaire (README.md) en se concentrant sur les vecteurs d'attaque liés à la monétisation, la conformité et les procédures de réponse aux incidents.

---

## Table des matières

- [Périmètre et priorités](#périmètre-et-priorités)
- [Sécurité des flux financiers](#sécurité-des-flux-financiers)
- [Intégrité du service de notarisation](#intégrité-du-service-de-notarisation)
- [Sécurité du badge organisation légale](#sécurité-du-badge-organisation-légale)
- [Protection de la licence white-label](#protection-de-la-licence-white-label)
- [Sécurité du RSC face aux plugins malveillants](#sécurité-du-rsc-face-aux-plugins-malveillants)
- [Conformité PCI DSS](#conformité-pci-dss)
- [Pen testing et audits](#pen-testing-et-audits)
- [Gestion des incidents](#gestion-des-incidents)
- [Divulgation responsable](#divulgation-responsable)

---

## Périmètre et priorités

La monétisation de Civium introduit des surfaces d'attaque absentes d'un protocole purement gratuit. Par ordre de criticité :

| Surface | Risque principal | Criticité |
|---|---|---|
| Flux de paiement (transactions 1%) | Contournement, rejeu, injection | Critique |
| Service de notarisation | Falsification du registre immuable | Critique |
| Badge légal | Usurpation d'identité légale | Haute |
| RSC (plugins commerciaux) | Plugin malveillant exfiltrant des données de paiement | Haute |
| Licence white-label | Copie / redistribution non autorisée | Moyenne |
| API pay-per-use | Abus de quota, scraping | Moyenne |

Les surfaces marquées **Critique** bloquent tout lancement de service monétisé si elles ne sont pas couvertes.

---

## Sécurité des flux financiers

### Principe fondateur

Le CIL (Civium Integration Layer) est le **seul point par lequel un flux financier peut transiter**. Aucun plugin ne peut déclencher ou recevoir un paiement sans passer par le CIL. Ce n'est pas une règle — c'est une contrainte architecturale.

```
Plugin commerce
  └── déclare capability : commerce.transaction
  └── appelle civium_api.transaction.create(montant, vendeur, acheteur)
       └── CIL intercepte
            ├── vérifie la capability déclarée dans le manifeste
            ├── calcule la commission (1%)
            ├── soumet au provider de paiement (Stripe Connect / Mollie)
            └── journalise la transaction avec signature Ed25519
```

Un plugin qui tenterait d'accéder directement à un provider de paiement sans passer par le CIL verrait sa requête réseau bloquée par le sandbox WASM.

### Protection contre le contournement de commission

| Vecteur | Protection |
|---|---|
| Plugin qui appelle Stripe directement | Bloqué par le sandbox WASM — pas d'accès réseau hors CIL |
| Plugin qui déclare `commerce.transaction` mais redirige le paiement | Hash du plugin vérifié à chaque exécution — toute modification casse la signature |
| Faux montant transmis au CIL | Le CIL recalcule le montant depuis la commande signée — pas de confiance sur la valeur déclarée par le plugin |
| Rejeu d'une transaction | Chaque transaction a un `nonce` unique + horodatage — le CIL rejette les doublons |

### Non-répudiation des transactions

Chaque transaction génère un **reçu signé** :

```json
{
  "transaction_id": "tx_a3f9e71b...",
  "timestamp": "2026-05-12T14:32:00Z",
  "montant_brut": 10.00,
  "commission_civium": 0.50,
  "montant_net": 8.91,
  "frais_paiement": 0.59,
  "cid_vendeur": "civium:a3f9...",
  "cid_acheteur": "civium:b4e2...",
  "cid_reseau": "civium:c7d3...",
  "plugin_id": "com.example.marketplace",
  "plugin_hash": "sha256:e3b0c4...",
  "signature_cil": "Ed25519:<sig>"
}
```

Ce reçu est stocké localement sur le nœud du réseau et ne peut pas être modifié rétrospectivement (CRDT `deleted` interdit sur les reçus de transaction pendant la durée légale de conservation).

### Reçus de transaction et droit à l'effacement RGPD

Les reçus de transaction contiennent des données personnelles (CID vendeur/acheteur) soumises au droit à l'effacement (Article 17 RGPD). Cependant, la loi comptable française impose une conservation des pièces justificatives pendant **10 ans**. Ces deux obligations coexistent via la règle suivante :

```
Pendant la durée légale (10 ans) :
  → Reçu conservé intégralement — obligation comptable prime sur le droit à l'effacement
  → Le membre peut demander la pseudonymisation de ses données dans le reçu
       └── CID remplacé par un identifiant opaque non réversible
       └── Le montant et la date restent (nécessaires à la comptabilité)

Après la durée légale :
  → CRDT `deleted` autorisé — le reçu est effaçable à la demande
```

Cette règle est communiquée aux membres lors de l'activation du module de paiement.

### Anti-fraude

| Signal | Détection | Réaction |
|---|---|---|
| Volume de transactions anormalement élevé en peu de temps | Rate limiting par réseau et par plugin | Suspension temporaire + alerte admin |
| Transaction avec montant = 0 € suivie d'une annulation | Pattern de test de bypass | Journalisation + alerte |
| Même `nonce` présenté deux fois | Tentative de rejeu | Rejet immédiat + incident de sécurité |
| Plugin modifié entre deux transactions | Hash différent du manifeste installé | Suspension du plugin + alerte |
| Acheteur et vendeur = même CID | Transaction circulaire suspecte | Blocage automatique |

---

## Intégrité du service de notarisation

### Exigence fondamentale

Une notarisation n'a de valeur que si son registre est **infalsifiable et vérifiable indépendamment** de Civium. Si Civium disparaît, la preuve doit rester vérifiable.

### Architecture du registre

```
Document (reste sur le nœud)
  └── Hash SHA-256 calculé localement
  └── Soumis au service de notarisation Civium
       └── Civium construit un enregistrement :
            {
              "hash_document": "sha256:...",
              "timestamp": "2026-05-12T14:32:00Z",
              "cid_reseau": "civium:...",
              "nonce": "..."
            }
       └── Ancré dans OpenTimestamps (Bitcoin blockchain)
       └── Certificat retourné et stocké sur le nœud
```

**OpenTimestamps** est un standard ouvert d'horodatage via la blockchain Bitcoin. Le certificat est vérifiable par n'importe qui, sans faire confiance à Civium.

### Garanties

| Garantie | Mécanisme |
|---|---|
| Le contenu du document ne quitte jamais le nœud | Seul le hash SHA-256 est transmis |
| La date ne peut pas être antidatée | Ancrage Bitcoin — le bloc a une date publique |
| Civium ne peut pas modifier un certificat émis | La preuve est sur la blockchain, pas chez Civium |
| Vérification indépendante possible | OpenTimestamps est open source, vérifiable hors Civium |

### Ce que la notarisation ne prouve pas

- **Pas la légalité du contenu** — Civium notarise un hash, pas un document
- **Pas l'identité réelle des signataires** — seulement leurs CID
- **Pas la valeur probante légale absolue** — varie selon la juridiction ; compléter avec un avocat pour usage judiciaire

---

## Sécurité du badge organisation légale

### Vecteurs d'usurpation

| Vecteur | Protection |
|---|---|
| Soumettre un SIRET légitime d'une autre entreprise | **Actuel :** vérification que le nom du réseau correspond à la raison sociale. **Prévu :** vérification que le CID correspond à un admin déclaré dans le Kbis (nécessite une API Kbis — non disponible aujourd'hui) |
| Créer une association fictive juste pour obtenir le badge | Délai de vérification : l'association doit exister depuis > 3 mois dans le JO |
| Réutiliser un badge après dissolution de l'entité | Vérification périodique automatique (tous les 6 mois) via les APIs publiques |
| Badge volé d'un réseau légitime | Le badge est lié au CID — non transférable |

### Procédure de vérification

```
1. Réseau soumet : type d'entité + numéro légal + document justificatif (optionnel)
2. Civium interroge l'API publique correspondante :
   └── Entreprise → API Sirene (data.gouv.fr)
   └── Association → API Journal Officiel (jo.fr)
   └── Collectivité → Annuaire des collectivités (DILA)
3. Vérification : entité active + nom cohérent avec le réseau
4. Si validé → badge émis, lié au CID, signé par Civium
5. Si rejeté → motif communiqué, possibilité de recours
```

### Révocation du badge

Un badge peut être révoqué si :
- L'entité légale est dissoute ou radiée
- Une usurpation est détectée
- Le réseau ne passe pas la vérification périodique

La révocation est immédiate et visible dans l'annuaire.

---

## Protection de la licence white-label

### Qu'est-ce qui est protégé

La licence white-label protège la **marque et l'usage commercial**, pas le code (qui est open source). Un déployeur ne peut pas :
- Redistribuer la licence à un tiers
- Déployer plus de réseaux que ce que la licence autorise

La mention "Propulsé par Civium" est **optionnelle selon le niveau de licence** :

| Niveau | Mention Civium |
|---|---|
| Petite structure (< 500 membres) | Obligatoire — ex : "Propulsé par Civium" en pied de page |
| Structure moyenne (500–5 000 membres) | Optionnelle — peut être retirée moyennant supplément |
| Grande organisation (> 5 000 membres) | Libre — marque totalement indépendante de Civium |

### Mécanisme de vérification

Chaque déploiement white-label reçoit un **token de licence** signé par Civium :

```json
{
  "licence_id": "wl_a3f9...",
  "titulaire_cid": "civium:...",
  "domaine": "connect.maville.fr",
  "max_reseaux": 500,
  "expire": null,
  "signature_civium": "Ed25519:<sig>"
}
```

- Le nœud vérifie le token au démarrage
- `expire: null` = licence perpétuelle (pas d'abonnement)
- Le token est lié au domaine et au CID du titulaire — non transférable
- En cas de litige, le token signé est la preuve contractuelle

---

## Sécurité du RSC face aux plugins malveillants

Un plugin commercial ayant accès à des données de paiement est une cible privilégiée. Les protections en vigueur (README.md) sont complétées par :

### Exigences supplémentaires pour les plugins `commerce`

| Exigence | Standard | Commerce |
|---|---|---|
| Manifeste valide | Requis | Requis |
| Sandbox WASM | Requis | Requis |
| Builds reproductibles | Certifiés seulement | **Tous** |
| Audit de permissions | Certifiés seulement | **Tous** |
| Revue de code Civium | Non | **Obligatoire** |
| Test de non-contournement commission | Non | **Obligatoire** |

Aucun plugin déclarant `commerce.transaction` ne peut être publié sur le RSC sans revue de code manuelle par l'équipe Civium.

### Quarantaine des nouveaux plugins commerce

Tout plugin commerce nouvellement publié est placé en **quarantaine de 30 jours** :
- Visible dans le RSC avec badge "En observation"
- Installable, mais avec avertissement explicite à l'admin
- Surveillé pour comportements anormaux (volume, patterns d'accès)
- Sorti de quarantaine après 30 jours sans incident signalé

---

## Conformité PCI DSS

Dès que Civium manipule des flux de paiement réels, la conformité **PCI DSS** (Payment Card Industry Data Security Standard) s'applique aux opérateurs de nœuds traitant des paiements.

### Niveau de conformité applicable

Civium utilise Stripe Connect ou Mollie comme provider de paiement — ces providers sont eux-mêmes certifiés PCI DSS niveau 1. En déléguant le traitement des données de carte à ces providers, Civium vise le niveau **SAQ A** ou **SAQ A-EP** selon l'implémentation retenue.

```
Données de carte bancaire
  └── Jamais stockées par Civium
  └── Jamais transmises au nœud Civium
  └── Traitées exclusivement par Stripe/Mollie (PCI DSS niveau 1)
  └── Civium reçoit uniquement : statut de paiement + identifiant transaction
```

**SAQ A vs SAQ A-EP — le niveau exact dépend de l'implémentation :**

| Implémentation | Niveau applicable |
|---|---|
| Redirection complète vers Stripe Checkout (aucun JS Civium dans le flux carte) | **SAQ A** — le plus léger |
| Formulaire de paiement Stripe.js/Elements intégré dans l'UI Civium | **SAQ A-EP** — légèrement plus exigeant |
| Tout autre cas (traitement partiel côté nœud) | **SAQ D** — à éviter |

**Recommandation :** implémenter via redirection Stripe Checkout pour rester en SAQ A. À confirmer avec le provider de paiement retenu avant tout lancement.

### Obligations pour les opérateurs de nœuds

Un opérateur de nœud qui active les transactions commerciales doit :
- Utiliser HTTPS (TLS 1.2+) sur toutes les connexions
- Ne jamais logger les données de carte (même partielles)
- Mettre à jour le nœud Civium dans les 30 jours suivant un patch de sécurité critique
- Déclarer tout incident de sécurité à Civium dans les 72h (conformité RGPD également)

---

## Pen testing et audits

### Calendrier

| Événement | Fréquence | Périmètre |
|---|---|---|
| Audit de sécurité externe | Avant chaque version majeure | Protocole complet |
| Pen test infrastructure | Annuel | Nœuds officiels Civium |
| Revue de code sécurité | À chaque PR sur civium-core | Diff uniquement |
| Test de non-régression sécurité | CI/CD automatique | Suite de tests de sécurité |

### Périmètre du pen test

Le pen test doit couvrir en priorité :
1. Contournement de la commission CIL (injection, bypass WASM)
2. Falsification du registre de notarisation
3. Usurpation de badge légal
4. Escalade de privilèges inter-réseaux (bypass APC)
5. Déni de service sur les nœuds officiels

### Publication des résultats

Les rapports d'audit sont publiés intégralement — favorables ou non — dans le dépôt public Civium, dans un délai maximum de 90 jours après réception.

---

## Gestion des incidents

### Niveaux de sévérité

| Niveau | Définition | Délai de réponse | Délai de résolution |
|---|---|---|---|
| **P0 — Critique** | Fuite de données, contournement commission, RCE | 2h | 24h |
| **P1 — Haute** | Bypass permissions, fraude avérée, badge usurpé | 8h | 72h |
| **P2 — Moyenne** | DoS, fuite métadonnées, anomalie paiement | 24h | 7 jours |
| **P3 — Faible** | Comportement inattendu sans impact financier | 72h | Prochain cycle |

### Procédure P0 / P1

```
1. Détection (monitoring automatique ou signalement)
2. Isolation immédiate du composant affecté
   └── Suspension du service concerné (notarisation, transactions...)
   └── Notification aux opérateurs de nœuds affectés
3. Analyse de l'impact (données exposées, transactions compromises)
4. Correctif développé et testé
5. Déploiement du patch
6. Post-mortem public dans les 30 jours
```

### Notification aux utilisateurs affectés

En cas d'incident P0/P1 affectant des données personnelles ou financières :
- Notification aux réseaux affectés dans les **72h** (obligation RGPD)
- Communication publique sur le dépôt Civium
- Rapport détaillé dans les 30 jours

---

## Divulgation responsable

### Comment signaler une vulnérabilité

Toute vulnérabilité de sécurité doit être signalée **de manière privée** avant toute publication :

```
Email chiffré : security@civium.net
Clé PGP      : publiée sur civium.net/security
Objet        : [SECURITY] description courte
```

### Engagement de Civium

| Engagement | Délai |
|---|---|
| Accusé de réception | 48h |
| Confirmation de la vulnérabilité | 7 jours |
| Correctif ou plan d'action | 30 jours |
| Crédit public au chercheur (si souhaité) | À la publication du correctif |

### Ce qui est dans le périmètre

- Protocole Civium (civium-core)
- Infrastructure officielle (nœuds bootstrap, RSC, service de notarisation)
- Plugins officiels Civium
- CIL et mécanismes de commission

### Ce qui est hors périmètre

- Nœuds tiers opérés par des tiers (leur responsabilité)
- Plugins tiers publiés par des développeurs indépendants
- Attaques nécessitant un accès physique aux appareils des membres

### Programme bug bounty

Un programme de bug bounty formel sera lancé avant la v1 publique. Les montants seront publiés dans la gouvernance du projet. Ordre de grandeur indicatif :

| Criticité | Montant indicatif |
|---|---|
| P0 — Critique | 500–2 000 € |
| P1 — Haute | 200–500 € |
| P2 — Moyenne | 50–200 € |
| P3 — Faible | Mention publique |
