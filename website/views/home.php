<!-- Hero -->
<section class="max-w-5xl mx-auto px-4 pt-20 pb-16 text-center">
  <div class="inline-block bg-gray-100 text-gray-600 text-xs font-medium px-3 py-1 rounded-full mb-6">Protocole open-source — en développement</div>
  <h1 class="text-4xl md:text-5xl font-bold text-gray-900 leading-tight mb-6">
    Des réseaux souverains,<br>connectés par choix.
  </h1>
  <p class="text-lg text-gray-500 max-w-2xl mx-auto mb-10">
    Civium remplace WhatsApp, Slack, Facebook Groups et les autres — sans céder vos données, sans algorithme, sans dépendance. Chaque communauté garde le contrôle total.
  </p>
  <!-- Waitlist form -->
  <div x-data="waitlist()" class="flex flex-col sm:flex-row gap-3 justify-center max-w-md mx-auto">
    <input
      x-model="email"
      type="email"
      placeholder="votre@email.fr"
      class="flex-1 border border-gray-300 rounded-lg px-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-gray-900"
      :disabled="sent"
    >
    <button
      @click="submit()"
      class="bg-gray-900 text-white px-5 py-2.5 rounded-lg text-sm font-medium hover:bg-gray-700 disabled:opacity-50 transition"
      :disabled="sent || loading"
    >
      <span x-show="!sent">Rejoindre la liste d'attente</span>
      <span x-show="sent">Inscrit ✓</span>
    </button>
  </div>
  <p x-data="waitlist()" class="text-xs text-gray-400 mt-3">Pas de spam. Notification à la sortie du MVP uniquement.</p>
</section>

<!-- Problème → Solution -->
<section class="bg-gray-50 py-16">
  <div class="max-w-5xl mx-auto px-4">
    <h2 class="text-2xl font-bold text-center mb-12">Le problème que Civium résout</h2>
    <div class="grid md:grid-cols-3 gap-6">
      <div class="bg-white border border-gray-200 rounded-xl p-6">
        <div class="text-2xl mb-3">🔒</div>
        <h3 class="font-semibold mb-2">Les plateformes centralisées</h3>
        <p class="text-sm text-gray-500">WhatsApp, Slack, Facebook Groups — riches en fonctionnalités, mais vos données leur appartiennent. Si la plateforme change de politique, votre communauté disparaît avec elle.</p>
      </div>
      <div class="bg-white border border-gray-200 rounded-xl p-6">
        <div class="text-2xl mb-3">🏝️</div>
        <h3 class="font-semibold mb-2">Les outils auto-hébergés</h3>
        <p class="text-sm text-gray-500">Nextcloud, Mattermost, Discourse — souverains, mais cloisonnés. Chaque outil est une île. Pas d'interopérabilité, pas de gouvernance collective.</p>
      </div>
      <div class="bg-gray-900 text-white rounded-xl p-6">
        <div class="text-2xl mb-3">⚡</div>
        <h3 class="font-semibold mb-2">Civium — la troisième voie</h3>
        <p class="text-sm text-gray-300">Toutes les fonctionnalités des plateformes centralisées, avec la souveraineté des outils auto-hébergés, et l'interopérabilité que ni l'un ni l'autre ne propose.</p>
      </div>
    </div>
  </div>
</section>

<!-- Le contrôle -->
<section class="py-16 max-w-5xl mx-auto px-4">
  <h2 class="text-2xl font-bold text-center mb-4">Le contrôle, c'est quoi concrètement ?</h2>
  <p class="text-center text-gray-500 mb-12 max-w-xl mx-auto">Les plateformes vous offrent des outils en échange de vos données. Civium vous offre les mêmes outils sans cette contrepartie.</p>
  <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-4">
    <div class="flex gap-4 p-5 border border-gray-100 rounded-xl">
      <span class="text-xl">🗄️</span>
      <div><h3 class="font-medium mb-1">Vos données sur votre nœud</h3><p class="text-sm text-gray-500">Personne n'y a accès sans votre permission explicite.</p></div>
    </div>
    <div class="flex gap-4 p-5 border border-gray-100 rounded-xl">
      <span class="text-xl">📋</span>
      <div><h3 class="font-medium mb-1">Vos règles</h3><p class="text-sm text-gray-500">Chaque communauté définit sa gouvernance, ses cercles de confiance, ses connexions.</p></div>
    </div>
    <div class="flex gap-4 p-5 border border-gray-100 rounded-xl">
      <span class="text-xl">🔑</span>
      <div><h3 class="font-medium mb-1">Votre identité</h3><p class="text-sm text-gray-500">Un identifiant cryptographique qui vous appartient, portable entre tous les réseaux.</p></div>
    </div>
    <div class="flex gap-4 p-5 border border-gray-100 rounded-xl">
      <span class="text-xl">🧩</span>
      <div><h3 class="font-medium mb-1">Votre écosystème</h3><p class="text-sm text-gray-500">Vous choisissez vos plugins — vous ne subissez pas les fonctionnalités imposées.</p></div>
    </div>
    <div class="flex gap-4 p-5 border border-gray-100 rounded-xl">
      <span class="text-xl">🛡️</span>
      <div><h3 class="font-medium mb-1">Votre indépendance</h3><p class="text-sm text-gray-500">Si Civium disparaît demain, votre nœud continue de fonctionner.</p></div>
    </div>
    <div class="flex gap-4 p-5 border border-gray-100 rounded-xl">
      <span class="text-xl">🔗</span>
      <div><h3 class="font-medium mb-1">Vos connexions</h3><p class="text-sm text-gray-500">Chaque lien entre réseaux est explicite, contractualisé et révocable.</p></div>
    </div>
  </div>
</section>

<!-- Cas d'usage aperçu -->
<section class="bg-gray-50 py-16">
  <div class="max-w-5xl mx-auto px-4">
    <h2 class="text-2xl font-bold text-center mb-12">Pour qui ?</h2>
    <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-4 text-sm">
      <a href="/cas-d-usage#famille" class="bg-white border border-gray-200 rounded-xl p-5 hover:border-gray-400 transition">
        <div class="text-2xl mb-2">👨‍👩‍👧‍👦</div>
        <h3 class="font-semibold mb-1">Famille</h3>
        <p class="text-gray-500">Album photo, agenda, documents, messagerie — en local, sans Google ni WhatsApp.</p>
      </a>
      <a href="/cas-d-usage#association" class="bg-white border border-gray-200 rounded-xl p-5 hover:border-gray-400 transition">
        <div class="text-2xl mb-2">🤝</div>
        <h3 class="font-semibold mb-1">Association</h3>
        <p class="text-gray-500">Membres, votes, agenda, comptabilité, appels à projets — gouvernance réelle.</p>
      </a>
      <a href="/cas-d-usage#quartier" class="bg-white border border-gray-200 rounded-xl p-5 hover:border-gray-400 transition">
        <div class="text-2xl mb-2">🏘️</div>
        <h3 class="font-semibold mb-1">Quartier</h3>
        <p class="text-gray-500">Annuaire de voisinage, troc, événements, concertation citoyenne.</p>
      </a>
      <a href="/cas-d-usage#entreprise" class="bg-white border border-gray-200 rounded-xl p-5 hover:border-gray-400 transition">
        <div class="text-2xl mb-2">🏢</div>
        <h3 class="font-semibold mb-1">Entreprise</h3>
        <p class="text-gray-500">Projets, documents, RH, communication interne — chaque accès audité.</p>
      </a>
    </div>
    <div class="text-center mt-8">
      <a href="/cas-d-usage" class="text-sm text-gray-500 hover:text-gray-900 underline underline-offset-2">Voir tous les cas d'usage →</a>
    </div>
  </div>
</section>

<!-- CTA final -->
<section class="py-20 max-w-5xl mx-auto px-4 text-center">
  <h2 class="text-2xl font-bold mb-4">Suivre le projet</h2>
  <p class="text-gray-500 mb-8 max-w-lg mx-auto">Civium est en développement actif. Inscrivez-vous pour être notifié à la sortie du MVP, ou contribuez directement sur GitHub.</p>
  <div class="flex flex-col sm:flex-row gap-3 justify-center">
    <a href="#" @click.prevent="document.querySelector('input[type=email]').focus(); window.scrollTo({top:0,behavior:'smooth'})"
       class="bg-gray-900 text-white px-6 py-2.5 rounded-lg text-sm font-medium hover:bg-gray-700 transition">
      Rejoindre la liste d'attente
    </a>
    <a href="https://github.com/rouaix/Civium" target="_blank"
       class="border border-gray-300 text-gray-700 px-6 py-2.5 rounded-lg text-sm font-medium hover:border-gray-500 transition">
      Voir sur GitHub
    </a>
  </div>
</section>

<script>
function waitlist() {
  return {
    email: '',
    sent: false,
    loading: false,
    async submit() {
      if (!this.email) return;
      this.loading = true;
      try {
        const res = await fetch('/inscription', {
          method: 'POST',
          headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
          body: 'email=' + encodeURIComponent(this.email)
        });
        const data = await res.json();
        if (data.status === 'ok' || data.status === 'duplicate') this.sent = true;
      } finally {
        this.loading = false;
      }
    }
  }
}
</script>
