<?php
require 'vendor/autoload.php';

$f3 = Base::instance();
$f3->set('DEBUG', 0);
$f3->set('UI', 'views/');

// Pages
$f3->route('GET /', function($f3) {
    $f3->set('page', 'home');
    $f3->set('title', 'Civium — Des réseaux souverains, connectés par choix');
    echo Template::instance()->render('layout.php');
});

$f3->route('GET /comment-ca-marche', function($f3) {
    $f3->set('page', 'how');
    $f3->set('title', 'Comment ça marche — Civium');
    echo Template::instance()->render('layout.php');
});

$f3->route('GET /cas-d-usage', function($f3) {
    $f3->set('page', 'usecases');
    $f3->set('title', 'Cas d\'usage — Civium');
    echo Template::instance()->render('layout.php');
});

$f3->route('GET /feuille-de-route', function($f3) {
    $f3->set('page', 'roadmap');
    $f3->set('title', 'Feuille de route — Civium');
    echo Template::instance()->render('layout.php');
});

$f3->route('GET /contribuer', function($f3) {
    $f3->set('page', 'contribute');
    $f3->set('title', 'Contribuer — Civium');
    echo Template::instance()->render('layout.php');
});

// Inscription liste d'attente
$f3->route('POST /inscription', function($f3) {
    $email = filter_var($f3->get('POST.email'), FILTER_VALIDATE_EMAIL);
    if ($email) {
        $db = new \DB\SQL('sqlite:' . __DIR__ . '/data/waitlist.db');
        $db->exec('CREATE TABLE IF NOT EXISTS waitlist (id INTEGER PRIMARY KEY, email TEXT UNIQUE, created_at TEXT)');
        try {
            $db->exec('INSERT INTO waitlist (email, created_at) VALUES (?, ?)', [$email, date('c')]);
            $status = 'ok';
        } catch (\Exception $e) {
            $status = 'duplicate';
        }
    } else {
        $status = 'invalid';
    }
    header('Content-Type: application/json');
    echo json_encode(['status' => $status]);
});

$f3->run();
