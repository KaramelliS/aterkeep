# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="logo aterkeep" width="280"/>
</p>

**Gestionnaire de serveur Aternos & dashboard 24/7.** Un seul binaire Rust (~1.7 Mo) garde votre serveur Minecraft Aternos gratuit en ligne en permanence, avec un panel web moderne — zéro automatisation navigateur, HTTP pur.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a>
</p>

## Fonctionnalités

- **Confirmation automatique de la file** — quand ton tour arrive, Aternos ouvre une fenêtre d'environ 30 secondes ; sans réponse tu repars à la fin. C'est l'étape qui rend le 24/7 sans surveillance possible.
- **Se connecte à ta place** — avec ton compte Aternos, aterkeep obtient le cookie de session lui-même et le renouvelle à son expiration. Pas de DevTools, pas de copier-coller mensuel.
- **Bot anti-idle** — un client Minecraft qui rejoint dès que le serveur tourne, pour qu'il ne soit pas arrêté parce qu'il est vide
- **Boucle keep-alive** — vérifie toutes les 90 s, redémarre le serveur automatiquement (désactivable)
- **Dashboard web** — statut en direct, démarrage/arrêt/redémarrage, interrupteur auto-start
- **Console serveur** — log serveur en direct depuis le navigateur
- **Éditeur de paramètres** — lire/modifier `server.properties` depuis le panel
- **Liste des joueurs** — qui est en ligne
- **Request inspector** — chaque appel HTTP avec sa réponse JSON (pédagogique)
- **14 langues** — UI commutable dans l'en-tête
- **Session chiffrée** — cookies en AES-256-GCM, la clé ne quitte jamais votre machine

## Prérequis

- Windows 10/11 (utilise `curl.exe` intégré)
- Toolchain Rust (uniquement pour compiler)

## Installation

```powershell
cd rust
cargo build --release
# binaire : target/release/aterkeep.exe
```

## Installation (une fois)

Lance le binaire et ouvre **http://127.0.0.1:4041**. L'assistant demande trois
choses : la langue du panneau, un mot de passe, la session Aternos.

Le **mot de passe du panneau** protège le panneau *et* chiffre la session. La clé
n'est **jamais écrite sur le disque** : elle est dérivée du mot de passe à chaque
démarrage (PBKDF2-HMAC-SHA256, 600 000 itérations). **Aucune récupération.**

**Deux façons pour la session :**

**1. Compte Aternos (par défaut).** Saisis ton identifiant et ton mot de passe :
aterkeep se connecte en HTTP pur et récupère le cookie lui-même. Si ton compte a
plusieurs serveurs, l'assistant demande lequel maintenir. Les identifiants sont
stockés dans `config/session.enc`, sous le même chiffrement AES-256-GCM que les
cookies, et ne sont envoyés qu'à `aternos.org`.

> Impossible avec l'**authentification à deux facteurs** ou si Aternos exige un
> **captcha** ; les deux sont signalés par un message dédié.

**2. Coller les cookies (repli).** Sur `aternos.org` : F12 → **Network** → F5,
copie toute la ligne `cookie:` d'une requête, puis exécute `window.AJAX_TOKEN`
dans la **Console**. Une session créée ainsi **ne se renouvelle pas seule**.

## Lancer

```powershell
.\target\release\aterkeep.exe
```

Ouvrir **http://127.0.0.1:4041**.

## Onglets du panel

| Onglet | Fonction |
|---|---|
| **Statut** | badge d'état, contrôles, auto-start, log live, request inspector |
| **Console** | flux de log serveur (rafraîchissement 10 s) |
| **Paramètres** | modifier `server.properties` et enregistrer |
| **Joueurs** | liste des joueurs en ligne |

**Interrupteur auto-start important :** off = le serveur ne redémarre jamais. **Arrêter** le coupe automatiquement.

## Durée de la session

Aternos délivre le cookie avec `Max-Age=2592000` — **exactement 30 jours**,
mesuré sur la réponse de connexion, pas deviné.

**Avec un compte :** rien à faire. À l'expiration, le démon se reconnecte et
continue — une ligne dans le journal.

**Avec des cookies collés :** le panneau affiche un badge `SESSION` et une
bannière expliquant que la session a expiré — ce n'est pas un serveur arrêté —
avec un bouton qui ramène à l'assistant.

Le panneau affiche aussi l'**âge de la session**, et après la première
expiration, la durée de la précédente.

Démarrage automatique après un redémarrage : **[docs/AUTOSTART.md](docs/AUTOSTART.md)** (tâche planifiée Windows avec DPAPI, systemd, Termux:Boot).

## Sécurité

- Session chiffrée au repos (`session.enc`, AES-256-GCM)
- **Aucun fichier de clé sur le disque** — la clé est dérivée du mot de passe (PBKDF2, 600 000 itérations, sel aléatoire par installation). Un dossier `config/` copié ne sert à rien sans le mot de passe
- **Le panneau exige une connexion** — tous les points d'accès derrière un cookie de session `HttpOnly`
- Chaînes API chiffrées dans le binaire, décodées à l'exécution avec votre clé
- Panel lié à `127.0.0.1` uniquement

## ⚠ Before you buy: Aternos' terms

Aternos' own support documentation says:

> *"Trying to bypass Aternos system by using bots, scripts, or other tricks to
> keep your server on 24/7 is against our rules… The system automatically
> checks for artificial activity."*
> — [24/7 Hosting](https://support.aternos.org/hc/en-us/articles/31771896948253-24-7-Hosting)

That describes this product. **Using aterkeep may get your server or your
Aternos account suspended or deleted.** There is no way for us to prevent that,
and the anti-idle bot makes the activity easier to spot, not harder.

This is sold as-is, for use on accounts you control and are willing to risk.
If that is not acceptable to you, do not buy it — a paid Minecraft host is the
supported way to run a server around the clock.

## Licence

**aterkeep est un logiciel commercial — ce n'est pas un logiciel libre.**

Le code source est publié uniquement à des fins de transparence et d'évaluation.
L'usage personnel et non commercial est autorisé. La redistribution, la revente,
les œuvres dérivées et l'usage commercial sont **interdits**. Conditions
complètes : [LICENSE](LICENSE).

## Acheter une licence

L'usage commercial, la redistribution, le white-labelling et l'accès au code du
moteur keep-alive (`aterkeep-core`) nécessitent une licence commerciale payante.

**Contact :** berlaylc2138@gmail.com

## Avertissement

Projet indépendant — sans lien avec Aternos GmbH ni Mojang Studios.
