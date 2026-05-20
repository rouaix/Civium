<?php

/**
 * Application web Civium — SPA Alpine.js + WASM.
 *
 * GET /app → vérifie la session, sert le client web
 */
class AppController
{
    protected Base $f3;

    public function __construct()
    {
        $this->f3 = Base::instance();
    }

    public function app(): void
    {
        if (session_status() === PHP_SESSION_NONE) {
            session_name('civium_sess');
            session_start();
        }

        if (empty($_SESSION['civium_email'])) {
            $this->f3->reroute('/auth');
            return;
        }

        $this->f3->set('title', 'Civium — Application');
        $this->f3->set('user_email', htmlspecialchars($_SESSION['civium_email'], ENT_QUOTES));

        echo Template::instance()->render('app.html');
    }
}
