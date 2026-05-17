# TODO.md — Civium

---

## A Développer ou corriger dans desktop — Priorité haute
- L'app doit s'ouvrir sur un fil d'actualité qui affiche toutes les activités de tous les réseaux. Avec le type d'activité, le nom du réseau et le nom du user. Et le contenu de l'activité (message, événement, etc.)
- Les message du serveur principal ne s'affichent pas dans l'app desktop.
- L'app doit pouvoir signaler un spam ou un abus etc. C'est à dire envoyer un message au serveur principal.
- L'app doit pouvoir demander à rejoindre un réseau, au serveur principal avec un simple clic dans une liste des réseaux publics.
- L'app doit afficher l'annuaire des réseaux publics (rejoindre sans invitation) et privés.
- La messagerie est destinée à échanger des messages privés entre users et réseaux. On doit donc pouvoir choisir à qui on envoi les messages.
- Les messages peuvent contenir du texte, des fichiers, (audio, vidéo, images, pdf, etc.) et des événements (calendrier, tâches, etc.)
- supprimer l'icone à côté du nom du résaeau civium.
- dans les messages il faut aussi afficher le nom du réseau et le nom du user qui envoi le message.
- LA gpartie gouvernance est à revoir. imcompréhensible et inutilisable en l'état. c'est trop compliquer le user doit juste cliquer sur des boutons.
- le lien Fédération avec d'autres réseaux est faux : https://www.rouaix.com/civium/civium/users/civ18N7G42tR

## A Développer ou corriger dans websuite — Priorité haute
- Bugg dans /auth le lien de connexion est envoyé à n'importe qui et connecte le nouveau user à n'importe quel réseau sur le serveur principal.
- si un usesr se connecte via /auth et que sont email n'est connue, cela doit créer un nouveau réseaux (Noeud) et donc un nouveau user.
- On doit pouvoir supprimer les messages et alertes du seveur et elles doivent dans ce cas disparaitre dans l'app.
- dans admin on doit pouvoir dans la liste des réseaux, agir sur le réseau (supprimer, désactiver, etc.)
- dans admin on doit pouvoir dans la liste des users, agir sur le user (supprimer, désactiver, etc.)
- dans admin on doit pouvoir dans la liste des messages, agir sur le message (supprimer, désactiver, etc.)
- dans admin on doit pouvoir dans la liste des alertes, agir sur l'alerte (supprimer, désactiver, etc.)
- dans les mails envoyés par le serveur, l'url affichée est fausse : https://www.rouaix.com/civium/civium/auth/verify?token=2e652448fb6008edc4b36f658945983671a8c8eecc01b2af32dc9c75e4fa807d HTTP 404 (GET /civium/auth?erreur=lien_expire)
- dans website, je dois pouvoir me connecter ou créer un nouveau noeud(réseau) avec login et mot de passe. et que cela m'envoi par email les infos nécessaire pour se connecter à l'app desktop.



## Déploiement PHP / Infrastructure — Priorité haute

> Ces critères de succès de la ROADMAP sont bloqués par l'absence de déploiement du serveur PHP sur `https://www.rouaix.com/civium`.

- Décider du domaine et de l'hébergement (civium.net ou civium.fr — Scaleway, ou rester sur rouaix.com/civium)
- Déployer le site PHP F3 en production (Apache/Nginx + PHP 8.x + MySQL)
- Vérifier que `POST /api/register` reçoit bien les enregistrements desktop et les stocke
- Vérifier que `GET /api/networks` retourne les réseaux enregistrés
- Tester le flux complet : créer un réseau dans l'app desktop → apparaît dans `/api/networks` en < 5 s
- Tester le retry exponentiel : couper Internet lors de la création → ré-enregistrement automatique à la reconnexion
- Tester le flux magic link de bout en bout : email → lien → session → saisie `secret_b58` + PIN → clé dans IndexedDB
- Tester l'émission d'une alerte fraude depuis `/admin` → réception email admin + affichage bandeau dans l'app desktop
- Configurer les variables SMTP dans `config.ini` (SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASS)
- Configurer ADMIN_TOKEN en production (variable d'environnement ou `config.ini`)
- Corriger l'URL dans les mails magic link (actuellement `/civium/auth/verify` → doit être `/civium/auth/verify`) — voir bug dans websuite


## Interopérabilité ActivityPub — Priorité moyenne

- Valider qu'un message Civium posté dans un réseau avec ActivityPub activé apparaît dans un fil Mastodon abonné (critère Phase 4 non coché)
- Tester `GET /.well-known/webfinger` et `GET /users/<cid>` depuis une instance Mastodon externe
- Tester la réception d'une note Mastodon via `POST /inbox` → apparaît en message Civium dans le Dashboard


## Phase 5 — Maturité — Priorité basse

- Planifier et mandater un audit de sécurité externe (cryptographie, P2P, CIL, plugin sandbox WASM)
- Créer la gouvernance du projet Civium lui-même (association loi 1901 ou fondation) — statuts, membres fondateurs, premier vote


## Demandes du concepteur - Priorité basse

  ---
  Mobile

  - Parité fonctionnelle avec desktop/website (mêmes plugins, ergonomie tactile)

  ---
  Plugin futur (backlog)

  - Partage de ressources matérielles : distribution de calcul entre machines (rendu 3D, LLM distribué…) — à planifier après les points précédents

---