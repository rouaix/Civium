<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{{ @title }}</title>
  <meta name="description" content="Civium est un protocole de réseaux souverains. Chaque communauté garde le contrôle total de ses données, de ses règles et de ses connexions.">
  <meta property="og:title" content="{{ @title }}">
  <meta property="og:description" content="Des réseaux souverains, connectés par choix. L'alternative aux plateformes centralisées.">
  <meta property="og:image" content="/public/img/og.png">
  <meta property="og:type" content="website">
  <link rel="icon" href="/public/img/favicon.svg" type="image/svg+xml">
  <script src="https://cdn.tailwindcss.com"></script>
  <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>
  <link rel="stylesheet" href="/public/css/style.css">
</head>
<body class="bg-white text-gray-900 antialiased">

  <!-- Nav -->
  <nav class="border-b border-gray-100 sticky top-0 bg-white/90 backdrop-blur z-50" x-data="{ open: false }">
    <div class="max-w-5xl mx-auto px-4 flex items-center justify-between h-14">
      <a href="/" class="flex items-center gap-2 font-semibold text-gray-900">
        <img src="/public/img/logo.svg" alt="Civium" class="h-7">
        Civium
      </a>
      <div class="hidden md:flex items-center gap-6 text-sm text-gray-600">
        <a href="/comment-ca-marche" class="hover:text-gray-900">Comment ça marche</a>
        <a href="/cas-d-usage" class="hover:text-gray-900">Cas d'usage</a>
        <a href="/feuille-de-route" class="hover:text-gray-900">Feuille de route</a>
        <a href="/contribuer" class="hover:text-gray-900">Contribuer</a>
        <a href="https://github.com/rouaix/Civium" target="_blank" class="hover:text-gray-900">GitHub</a>
      </div>
      <!-- Mobile menu button -->
      <button @click="open = !open" class="md:hidden p-2 text-gray-500">
        <svg x-show="!open" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/></svg>
        <svg x-show="open" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
      </button>
    </div>
    <!-- Mobile menu -->
    <div x-show="open" class="md:hidden border-t border-gray-100 px-4 py-3 flex flex-col gap-3 text-sm text-gray-600">
      <a href="/comment-ca-marche" class="hover:text-gray-900">Comment ça marche</a>
      <a href="/cas-d-usage" class="hover:text-gray-900">Cas d'usage</a>
      <a href="/feuille-de-route" class="hover:text-gray-900">Feuille de route</a>
      <a href="/contribuer" class="hover:text-gray-900">Contribuer</a>
      <a href="https://github.com/rouaix/Civium" target="_blank" class="hover:text-gray-900">GitHub</a>
    </div>
  </nav>

  <main>
    <include href="{{ @page }}.php" />
  </main>

  <footer class="border-t border-gray-100 mt-24 py-10 text-sm text-gray-400">
    <div class="max-w-5xl mx-auto px-4 flex flex-col md:flex-row justify-between gap-4">
      <div>© 2026 Civium — Protocole ouvert, données souveraines.</div>
      <div class="flex gap-6">
        <a href="https://github.com/rouaix/Civium" target="_blank" class="hover:text-gray-600">GitHub</a>
        <a href="/contribuer" class="hover:text-gray-600">Contribuer</a>
      </div>
    </div>
  </footer>

</body>
</html>
