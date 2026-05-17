# TODO.md — Civium

---

## Demandes du concepteur

 Desktop — UX prioritaire

  1. ✅ Restructuration de la navigation
  - Éléments de configuration → panneau ⚙ Paramètres
  - Adresses P2P, ports, RCC, hub, MCP, appareils → Paramètres
  - Page principale = messagerie, membres, gouvernance, activité

  2. ✅ Affichage statut nœud
  - Seul le point vert "En ligne" / "Hors ligne" sur l'écran principal
  - Adresses multiaddr, ports → Paramètres

  3. ✅ Plugins comme menus
  - Plugins actifs comme entrées de navigation dans la barre latérale
  - Chaque section accessible individuellement (Messages, Membres, Gouvernance, Agenda, Documents, Activité, Extensions…)

  4. ✅ Invitation simplifiée
  - Lien d'invitation avec description claire + bouton "Copier"

  5. ✅ Créer/gérer plusieurs réseaux
  - Bouton "+" pour créer un réseau (avec choix de type : privé, annuaire, RRM)
  - Page de création avec formulaire lisible

  ---
  Website — Priorité haute

  6. ✅ Application web complète
  - Partie publique/admin existante ✓
  - Interface utilisateur complète en PHP+Alpine.js (app.html)
  - Connexion via PIN + clé secrète (IndexedDB, sans WASM)
  - Ed25519 signing via TweetNaCl.js
  - Hub API : pull messages, push messages, rejoindre un réseau
  - Sidebar avec réseaux + navigation par section
  - Messages, Membres, Activité, Paramètres

  ---
  Mobile

  7. Parité fonctionnelle
  - Mêmes plugins que desktop/website
  - Adapté mobile (ergonomie tactile)

  ---
  Plugin futur (backlog)

  8. Partage de ressources matérielles
  - Distribution de calcul entre machines (rendu 3D, LLM distribué…)
  - À planifier après les points précédents

---


### Dans desktop et website — Priorité haute

1 de nombreuses incohérence dans l'usage et la création de réseaux.
  - ✅ je ne dois pas pouvoir créer plusieurs réseaux dans mon application. (Dashboard : bouton "+" masqué si réseau déjà existant)
  - je dois pouvoir me connecter à plusieurs réseaux.
  - lorsque j'utilise un plugin, je dois pouvoir choisir si j'envoi l'information à un réseaux ou à un autre, ou a un membre ou un ensemble de membres. ETC.
  - ✅ Je dois pouvoir inviter un membre par mail (Dashboard : champ email + bouton "Envoyer par email" → mailto: avec lien + adresses P2P)
  - losque j'invite un membre par mail il doit pouvoir créer automatiquement sont propre réseaux et se connecter au mien automatiquement.
  - ✅ le type annuaire est réservé à la recherchede membres dans un réseaux ou de reseaux. C'est un plugin qui gère cela pas un type de réseaux. (Dashboard : formulaire création réseau simplifié, types annuaire/RRM supprimés)
  - un membre invité sur un réseaux peut installer l'application et utiliser le réseaux déjà existant qui l'a invité.
  - Il faut pouvoir gérer les droits users dans les réseaux.
  - Il y a une incohérence, si chaque user est un réseau alors comment les regrouper dans un réseau ?
  - Il faut pouvoir créer des réseaux privés et des réseaux publics.
  - Il faut pouvoir créer des réseaux de réseaux exemple dans une famille, il y a un réseau pour la famille et un réseau pour chaque user de la famille.


### Dans Website — Priorité haute

1 ✅ On doit pouvoir créer son propre réseau via le site avant même d'avoir installé l'application
  - app.html : bouton "+" → "Créer un réseau" (POST /hub/network/register signé Ed25519)
2 ✅ Une fois l'application installée on doit pouvoir récupérer ses infos du site (import clé secrète). Ensuite l'application devient maître du réseau.
  - app.html : "Importer depuis l'application bureau" dans le setup
3 Si on n'installe pas l'application on peut tout faire avec le site.
  - Messagerie hub ✅ / gouvernance ❌ / agenda ❌
4 ✅ Bug résolu : nouveau compte bloqué par demande de clé secrète
  - app.html : écran de bienvenue avec choix "Créer une nouvelle identité" (génère keypair Ed25519 + CID blake3) ou "Importer"
5 il faut pouvoir créer un nœud en ligne et s'y connecter en ligne.
  - Un réseau créé via /app est hébergé sur le hub (nœud en ligne). ✅ partiel

