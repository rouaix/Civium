<section class="max-w-3xl mx-auto px-4 py-16">
  <div class="text-center mb-14">
    <h1 class="text-3xl font-bold mb-4">Cas d'usage</h1>
    <p class="text-gray-500">Civium s'adapte à toute communauté qui veut garder le contrôle de ses outils numériques.</p>
  </div>

  <?php
  $cases = [
    ['id' => 'famille', 'emoji' => '👨‍👩‍👧‍👦', 'title' => 'Famille', 'subtitle' => 'Remplace Google Photos, WhatsApp, Dropbox', 'desc' => 'Un espace privé où coexistent album photo partagé, agenda familial, coffre-fort de documents (actes, contrats), messagerie E2E, caisse commune — tout en local, sans dépendance aux GAFAM.',
      'features' => ['Album photo sur votre NAS ou Raspberry Pi', 'Agenda partagé entre tous les membres', 'Documents importants (actes, contrats, ordonnances)', 'Messagerie E2E entre membres de la famille', 'Contrôle parental pour les enfants']],
    ['id' => 'association', 'emoji' => '🤝', 'title' => 'Association', 'subtitle' => 'Remplace Facebook Groups, Slack, HelloAsso', 'desc' => 'Gestion des membres, votes, agenda, communication interne, comptabilité, appels à projets, marketplace de services entre membres — avec une gouvernance réelle, pas simulée.',
      'features' => ['Adhésions et liste des membres', 'Votes collectifs avec quorum configurable', 'Agenda des événements et réunions', 'Communication interne chiffrée', 'Connexion avec des associations partenaires']],
    ['id' => 'quartier', 'emoji' => '🏘️', 'title' => 'Quartier', 'subtitle' => 'Remplace Nextdoor, Facebook Quartier', 'desc' => 'Annuaire de voisinage, troc et dons, signalement de problèmes urbains, concertation citoyenne, événements locaux, covoiturage, bibliothèque partagée — connecté au réseau de la mairie si elle le souhaite.',
      'features' => ['Annuaire des habitants (opt-in)', 'Petites annonces, troc, dons', 'Événements et sorties de quartier', 'Concertation et budgets participatifs', 'Connexion avec la mairie (si accord)']],
    ['id' => 'entreprise', 'emoji' => '🏢', 'title' => 'Entreprise', 'subtitle' => 'Remplace Slack, Notion, Teams', 'desc' => 'Gestion de projets, documents partagés, facturation, RH, communication interne — chaque connexion avec un prestataire externe contractualisée, chaque accès audité.',
      'features' => ['Communication interne chiffrée', 'Gestion de projets et documents', 'Connexions sécurisées avec prestataires', 'Audit complet des accès', 'Synchronisation optionnelle avec Slack, Notion…']],
  ];
  foreach ($cases as $c): ?>
  <div id="<?= $c['id'] ?>" class="mb-14 scroll-mt-20">
    <div class="flex items-start gap-4 mb-5">
      <span class="text-4xl"><?= $c['emoji'] ?></span>
      <div>
        <h2 class="text-xl font-bold"><?= $c['title'] ?></h2>
        <p class="text-sm text-gray-400"><?= $c['subtitle'] ?></p>
      </div>
    </div>
    <p class="text-gray-600 mb-4"><?= $c['desc'] ?></p>
    <ul class="space-y-2">
      <?php foreach ($c['features'] as $f): ?>
        <li class="flex items-start gap-2 text-sm text-gray-600">
          <span class="text-gray-400 mt-0.5">✓</span>
          <?= htmlspecialchars($f) ?>
        </li>
      <?php endforeach; ?>
    </ul>
  </div>
  <?php endforeach; ?>

  <div class="bg-gray-50 rounded-xl p-6 text-center">
    <p class="text-gray-600 mb-4">Civium peut aussi servir une institution, une école, un hôpital ou une collectivité territoriale — toute structure qui veut une infrastructure numérique souveraine.</p>
    <a href="/comment-ca-marche" class="text-sm text-gray-900 font-medium underline underline-offset-2">Comprendre le protocole →</a>
  </div>
</section>
