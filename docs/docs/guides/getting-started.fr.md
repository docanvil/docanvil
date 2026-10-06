---
{
  "title": "Installation",
  "slug": "getting-started",
  "description": "Guide de démarrage rapide pour installer DocAnvil et créer votre première documentation"
}
---
# Installation

Installez DocAnvil et créez votre premier site de documentation.

## Installer DocAnvil

Les scripts d'installation téléchargent un binaire précompilé pour votre plateforme (environ 5 Mo), le vérifient avec les sommes de contrôle SHA-256 publiées avec la version, et l'ajoutent à votre `PATH`. Aucune chaîne d'outils Rust n'est nécessaire.

::::tabs
:::tab{title="macOS / Linux"}
```bash
curl -fsSL https://github.com/docanvil/docanvil/releases/latest/download/install.sh | sh
```

Installe dans `~/.local/bin`. Si ce dossier n'est pas encore dans votre `PATH`, le script vous indique la ligne à ajouter à votre profil shell.
:::
:::tab{title="Windows"}
```powershell
irm https://github.com/docanvil/docanvil/releases/latest/download/install.ps1 | iex
```

Installe dans `%LOCALAPPDATA%\docanvil\bin` et l'ajoute à votre `PATH` utilisateur. Ouvrez ensuite un nouveau terminal.
:::
:::tab{title="Depuis crates.io"}
```bash
# Compile DocAnvil et ses dépendances (nécessite une chaîne d'outils Rust)
cargo install docanvil
```
:::
:::tab{title="Depuis les sources"}
```bash
git clone https://github.com/docanvil/docanvil.git
cd docanvil
cargo install --path .
```
:::
::::

Vérifiez l'installation :

```bash
docanvil --help
```

### Options d'installation

| Option | Variable d'environnement | Effet |
|---|---|---|
| `--version 1.2.0` | `DOCANVIL_VERSION` | Installer une version précise plutôt que la dernière |
| `--install-dir DIR` | `DOCANVIL_INSTALL_DIR` | Installer ailleurs, par exemple `/usr/local/bin` en CI |
| `--force` | | Réinstaller même si cette version est déjà présente |
| `--quiet` | | N'afficher que les erreurs |

Passez des options au script redirigé avec `sh -s --` :

```bash
curl -fsSL https://github.com/docanvil/docanvil/releases/latest/download/install.sh | sh -s -- --version 1.2.0
```

Sous Windows, définissez les variables d'environnement avant de lancer la commande (par exemple `$env:DOCANVIL_VERSION = "1.2.0"`).

Relancer le script alors que cette version est déjà installée ne fait rien et se termine sans erreur : vous pouvez l'utiliser sans risque en CI.

## Mettre à jour

```bash
docanvil update          # chercher une nouvelle version et mettre à jour, avec confirmation
docanvil update --check  # indiquer seulement si une nouvelle version existe
docanvil update --yes    # mettre à jour sans confirmation
```

`docanvil serve` affiche aussi une ligne d'information quand une nouvelle version est disponible. La vérification a lieu au plus une fois par jour et jamais en CI. Définissez `DOCANVIL_NO_UPDATE_CHECK=1` pour la désactiver.

Si vous avez installé DocAnvil avec `cargo install`, mettez-le à jour avec `cargo install docanvil --force`.

## Créer un projet

Créez un nouveau projet de documentation avec `docanvil new` :

```bash
docanvil new mes-docs
```

Cela génère la structure suivante :

```text
mes-docs/
  docanvil.toml        # Configuration du projet
  nav.toml             # Structure de navigation
  docs/                # Votre contenu Markdown
    index.md           # Page d'accueil
    guides/
      getting-started.md
      configuration.md
  theme/
    custom.css         # Vos surcharges CSS
```

## Lancer le serveur de développement

```bash
cd mes-docs
docanvil serve
```

Le serveur de développement démarre par défaut sur [http://localhost:3000](http://localhost:3000). Vous pouvez changer l'hôte et le port :

```bash
docanvil serve --host 0.0.0.0 --port 8080
```

## Écrire votre première page

Créez un nouveau fichier Markdown n'importe où dans le répertoire `docs/` :

```markdown
# Ma nouvelle page

Bienvenue dans ma documentation !

- Prend en charge le texte **gras**, *italique*, et ~~barré~~
- Ajoutez [[index|des liens vers d'autres pages]] avec la syntaxe wiki-link
```

Enregistrez le fichier et votre navigateur se rechargera automatiquement. La page est découverte et ajoutée à la navigation.

## Compiler pour la production

Quand vous êtes prêt à déployer, générez le site statique :

```bash
docanvil build
```

La sortie va dans le répertoire `dist/` par défaut. Téléversez-le sur n'importe quel hébergeur statique — GitHub Pages, Netlify, Vercel, S3, ou un simple serveur web.

Utilisez `--clean` pour supprimer le répertoire de sortie avant de compiler :

```bash
docanvil build --clean
```

Pour les pipelines CI/CD, utilisez `--strict` pour faire échouer le build lorsqu'il y a des avertissements :

```bash
docanvil build --strict
```

## Checklist

- [x] Installer DocAnvil
- [x] Lancer `docanvil new` pour créer un projet
- [x] Démarrer le serveur de développement avec `docanvil serve`
- [ ] Écrire vos pages en Markdown
- [ ] Personnaliser le thème
- [ ] Compiler et déployer avec `docanvil build`

## Prochaines étapes

- [[guides/configuration|Configurez]] votre projet et votre navigation
- Découvrez les [[writing/markdown|fonctionnalités Markdown]] et les [[writing/components|composants]]
- [[guides/theming|Personnalisez le thème]] pour correspondre à votre identité visuelle

:::note
DocAnvil surveille tous les fichiers de votre projet. Les modifications apportées aux fichiers Markdown, aux fichiers de configuration, au CSS et aux templates déclenchent toutes un rechargement en direct.
:::
