<section class="max-w-3xl mx-auto px-4 py-16">
  <div class="text-center mb-14">
    <h1 class="text-3xl font-bold mb-4">Comment ça marche</h1>
    <p class="text-gray-500">Civium repose sur trois concepts : des réseaux souverains, des cercles de confiance, et des connexions contractualisées.</p>
  </div>

  <!-- Concept 1 -->
  <div class="mb-14">
    <div class="flex items-center gap-3 mb-4">
      <span class="bg-gray-900 text-white text-xs font-bold px-2.5 py-1 rounded-full">1</span>
      <h2 class="text-xl font-bold">Votre réseau, votre nœud</h2>
    </div>
    <p class="text-gray-600 mb-4">Un réseau Civium est un espace numérique souverain hébergé sur votre propre nœud — un VPS, un NAS, un Raspberry Pi, ou une instance mutualisée. Vos données ne quittent jamais votre infrastructure sans votre accord explicite.</p>
    <div class="bg-gray-50 rounded-xl p-5 font-mono text-sm text-gray-600">
      <div>Réseau "asso-velo" ── hébergé sur votre serveur</div>
      <div class="ml-4 text-gray-400">├── membres : Alice, Bob, Carol</div>
      <div class="ml-4 text-gray-400">├── plugins : Messagerie, Agenda, Documents</div>
      <div class="ml-4 text-gray-400">└── données : sur votre nœud uniquement</div>
    </div>
  </div>

  <!-- Concept 2 -->
  <div class="mb-14">
    <div class="flex items-center gap-3 mb-4">
      <span class="bg-gray-900 text-white text-xs font-bold px-2.5 py-1 rounded-full">2</span>
      <h2 class="text-xl font-bold">Les cercles de confiance</h2>
    </div>
    <p class="text-gray-600 mb-4">Chaque relation dans Civium est placée dans un cercle. Plus le cercle est élevé, plus l'accès est riche — et plus la confiance accordée est grande.</p>
    <div class="grid grid-cols-2 md:grid-cols-4 gap-3 text-sm text-center">
      <div class="bg-gray-50 border border-gray-200 rounded-xl p-4">
        <div class="font-bold text-lg mb-1">0</div>
        <div class="font-medium mb-1">Annuaire</div>
        <div class="text-gray-400 text-xs">Nom et existence dans le réseau</div>
      </div>
      <div class="bg-gray-50 border border-gray-200 rounded-xl p-4">
        <div class="font-bold text-lg mb-1">1</div>
        <div class="font-medium mb-1">Connaissance</div>
        <div class="text-gray-400 text-xs">Profil partiel, messagerie basique</div>
      </div>
      <div class="bg-gray-50 border border-gray-200 rounded-xl p-4">
        <div class="font-bold text-lg mb-1">2</div>
        <div class="font-medium mb-1">Confiance</div>
        <div class="text-gray-400 text-xs">Profil complet, partage de contenu</div>
      </div>
      <div class="bg-gray-900 text-white rounded-xl p-4">
        <div class="font-bold text-lg mb-1">3</div>
        <div class="font-medium mb-1">Intime</div>
        <div class="text-gray-300 text-xs">Accès complet, E2E strict</div>
      </div>
    </div>
  </div>

  <!-- Concept 3 -->
  <div class="mb-14">
    <div class="flex items-center gap-3 mb-4">
      <span class="bg-gray-900 text-white text-xs font-bold px-2.5 py-1 rounded-full">3</span>
      <h2 class="text-xl font-bold">Des connexions contractualisées</h2>
    </div>
    <p class="text-gray-600 mb-4">Deux réseaux peuvent se connecter pour partager des données ou des services. Chaque connexion est formalisée dans un accord signé cryptographiquement — et révocable à tout moment par l'un ou l'autre.</p>
    <div class="bg-gray-50 rounded-xl p-5 text-sm text-gray-600">
      <div class="flex items-center gap-3 mb-3">
        <span class="bg-white border border-gray-200 rounded px-2 py-1 font-medium">asso-velo</span>
        <span class="text-gray-400">──[accord signé]──▶</span>
        <span class="bg-white border border-gray-200 rounded px-2 py-1 font-medium">quartier-sud</span>
      </div>
      <div class="text-xs text-gray-400 space-y-1">
        <div>✓ asso-velo expose : agenda événements (lecture seule)</div>
        <div>✓ quartier-sud expose : annuaire membres (partiel)</div>
        <div>✗ messages privés : non partagés des deux côtés</div>
      </div>
    </div>
  </div>

  <!-- Concept 4 -->
  <div>
    <div class="flex items-center gap-3 mb-4">
      <span class="bg-gray-900 text-white text-xs font-bold px-2.5 py-1 rounded-full">4</span>
      <h2 class="text-xl font-bold">Tout est plugin</h2>
    </div>
    <p class="text-gray-600 mb-4">Chaque fonctionnalité — messagerie, agenda, marketplace, visioconférence — est un plugin. Vous choisissez ce que votre réseau installe depuis un catalogue ouvert. Aucune fonctionnalité imposée.</p>
    <div class="flex flex-wrap gap-2 text-sm">
      <?php foreach (['Messagerie', 'Agenda', 'Documents', 'Marketplace', 'Visioconférence', 'Wiki', 'Sondages', 'Facturation', 'Gestion de projet', '+ des centaines d\'autres…'] as $plugin): ?>
        <span class="bg-gray-100 text-gray-600 px-3 py-1 rounded-full"><?= htmlspecialchars($plugin) ?></span>
      <?php endforeach; ?>
    </div>
  </div>

</section>
