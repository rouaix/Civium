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
