# aterkeep

<p align="center">
  <img src="rust/static/logo.svg" alt="logo aterkeep" width="280"/>
</p>

**Gestionnaire de serveur Aternos & dashboard 24/7.** Un seul binaire Rust (~1.7 Mo) garde votre serveur Minecraft Aternos gratuit en ligne en permanence, avec un panel web moderne — zéro automatisation navigateur, HTTP pur.

<p align="center">
  <a href="README.md">English</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.de.md">Deutsch</a>
</p>

## Fonctionnalités

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

## Export de session (une fois)

1. Ouvrir **https://aternos.org** et se connecter.
2. `F12` → **Console** : `window.AJAX_TOKEN` → `token` ; `window.generateAjaxToken()` → partie après `:` → `sec`
3. `F12` → **Application → Cookies → https://aternos.org** : copier `ATERNOS_SESSION` et `ATERNOS_SERVER`
4. Créer `http/session.json` (format : [English README](README.md#setup--export-your-session-once)) :

```json
{
  "token": "PASTE_AJAX_TOKEN",
  "sec": "PASTE_GENERATE_AJAX_TOKEN_VALUE",
  "cookies": [
    { "name": "ATERNOS_SESSION", "value": "PASTE_SESSION_VALUE" },
    { "name": "ATERNOS_SERVER", "value": "PASTE_SERVER_ID" }
  ]
}
```

5. Importer :

```powershell
cd rust
.\target\release\aterkeep.exe import ..\http\session.json
```

Lors de l'installation, vous définissez un **mot de passe du panneau** : il protège le panneau *et* chiffre la session. La clé n'est **jamais écrite sur le disque** ; elle est dérivée du mot de passe à chaque démarrage. Tout est regroupé dans un seul dossier `config/`. **Aucune récupération possible en cas d'oubli.** Pour un fonctionnement sans surveillance : `ATERKEEP_KEY='votre-mot-de-passe' ./aterkeep`

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

## Durée de vie de la session

Les cookies de session Aternos durent **~30 jours**. Quand le panel affiche `OTURUM BİTTİ`/`LOGGED OUT`, répétez l'export et réimportez.

## Sécurité

- Session chiffrée au repos (`session.enc`, AES-256-GCM)
- **Aucun fichier de clé sur le disque** — la clé est dérivée du mot de passe (PBKDF2, 600 000 itérations, sel aléatoire par installation). Un dossier `config/` copié ne sert à rien sans le mot de passe
- **Le panneau exige une connexion** — tous les points d'accès derrière un cookie de session `HttpOnly`
- Chaînes API chiffrées dans le binaire, décodées à l'exécution avec votre clé
- Panel lié à `127.0.0.1` uniquement

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
