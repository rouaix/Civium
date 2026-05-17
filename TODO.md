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



## Demandes du concepteur - Priorité basse
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