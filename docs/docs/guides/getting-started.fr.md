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

::::tabs
:::tab{title="Depuis crates.io"}
```bash
# Installer depuis crates.io (une fois publié)
cargo install docanvil
```
:::
:::tab{title="Depuis GitHub"}
```bash
# Compiler depuis les sources
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
