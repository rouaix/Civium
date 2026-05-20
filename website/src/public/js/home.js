function waitlist() {
  const base = document.documentElement.dataset.base || '';
  return {
    email: '',
    sent: false,
    loading: false,
    error: '',
    async submit() {
      if (!this.email) return;
      this.loading = true;
      this.error = '';
      try {
        const res = await fetch(base + '/inscription', {
          method: 'POST',
          headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
          body: 'email=' + encodeURIComponent(this.email),
        });
        const data = await res.json();
        if (data.status === 'ok' || data.status === 'duplicate') {
          this.sent = true;
        } else if (data.status === 'invalid') {
          this.error = 'Adresse e-mail invalide.';
        } else {
          this.error = 'Une erreur est survenue, veuillez réessayer.';
        }
      } catch {
        this.error = 'Impossible de contacter le serveur.';
      } finally {
        this.loading = false;
      }
    },
  };
}
