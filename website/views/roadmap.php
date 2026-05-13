<section class="max-w-3xl mx-auto px-4 py-16">
  <div class="text-center mb-14">
    <h1 class="text-3xl font-bold mb-4">Feuille de route</h1>
    <p class="text-gray-500">Civium se construit de bas en haut — protocole de base d'abord, écosystème ensuite.</p>
  </div>

  <?php
  $phases = [
    ['label' => 'Priorité', 'title' => 'Site web de présentation', 'status' => 'En cours', 'color' => 'blue',
     'items' => ['Page d\'accueil et proposition de valeur', 'Cas d\'usage par type de communauté', 'Feuille de route publique', 'Formulaire d\'inscription liste d\'attente']],
    ['label' => 'Phase 0', 'title' => 'MVP — Protocole de base', 'status' => 'À venir', 'color' => 'gray',
     'items' => ['Identité cryptographique (CID Ed25519)', 'Transport P2P (libp2p / DHT)', 'Messagerie E2E dans un réseau', 'Connexion inter-réseaux avec accord signé', 'Application Desktop (Tauri) + CLI']],
    ['label' => 'Phase 1', 'title' => 'Gouvernance & Annuaires', 'status' => 'À venir', 'color' => 'gray',
     'items' => ['Votes collectifs et quorum', 'Annuaire de réseaux et de membres', 'Fédération d\'annuaires', 'Contrôle parental (réseaux famille)']],
    ['label' => 'Phase 2', 'title' => 'Services & Intégrations', 'status' => 'À venir', 'color' => 'gray',
     'items' => ['Système de plugins (WASM sandbox)', 'Plugins : Agenda, Documents, Marketplace', 'Connecteurs SaaS (Google Calendar, Stripe…)', 'Accès IA via MCP', 'Registre de Services Civium (RSC)']],
    ['label' => 'Phase 3', 'title' => 'Applications & Écosystème', 'status' => 'À venir', 'color' => 'gray',
     'items' => ['Application mobile iOS / Android', 'Application web (PWA)', 'Interopérabilité ActivityPub (Mastodon, PeerTube…)', 'Cercle 3 (pair E2E) + récupération sociale']],
    ['label' => 'Phase 4', 'title' => 'Maturité', 'status' => 'À venir', 'color' => 'gray',
     'items' => ['Certification des plugins', 'Audit de sécurité externe', 'SDK pour intégrateurs tiers', 'Gouvernance du projet Civium']],
  ];

  foreach ($phases as $i => $p):
    $isFirst = $i === 0;
  ?>
  <div class="flex gap-5 mb-10">
    <div class="flex flex-col items-center">
      <div class="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold <?= $isFirst ? 'bg-gray-900 text-white' : 'bg-gray-100 text-gray-500' ?>">
        <?= $isFirst ? '★' : $i ?>
      </div>
      <?php if ($i < count($phases) - 1): ?>
        <div class="flex-1 w-px bg-gray-200 my-2"></div>
      <?php endif; ?>
    </div>
    <div class="flex-1 pb-2">
      <div class="flex items-center gap-3 mb-1">
        <span class="text-xs font-medium text-gray-400"><?= $p['label'] ?></span>
        <span class="text-xs px-2 py-0.5 rounded-full <?= $isFirst ? 'bg-blue-100 text-blue-700' : 'bg-gray-100 text-gray-500' ?>"><?= $p['status'] ?></span>
      </div>
      <h2 class="font-semibold text-gray-900 mb-3"><?= $p['title'] ?></h2>
      <ul class="space-y-1.5">
        <?php foreach ($p['items'] as $item): ?>
          <li class="flex items-start gap-2 text-sm text-gray-500">
            <span class="mt-1 text-gray-300">◦</span>
            <?= htmlspecialchars($item) ?>
          </li>
        <?php endforeach; ?>
      </ul>
    </div>
  </div>
  <?php endforeach; ?>

  <div class="text-center mt-10">
    <a href="https://github.com/rouaix/Civium" target="_blank" class="text-sm text-gray-500 hover:text-gray-900 underline underline-offset-2">Suivi détaillé sur GitHub →</a>
  </div>
</section>
