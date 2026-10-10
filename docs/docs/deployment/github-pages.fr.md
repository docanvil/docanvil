---
{
  "title": "GitHub Pages",
  "slug": "github-pages",
  "description": "Déployez votre documentation DocAnvil sur GitHub Pages avec GitHub Actions"
}
---
# Déploiement sur GitHub Pages

Ce guide met en place un workflow GitHub Actions qui compile votre documentation DocAnvil et la publie sur GitHub Pages à chaque push sur `main`. C'est la configuration utilisée par cette documentation.

## Prérequis

Vous aurez besoin de :

- Un dépôt GitHub contenant votre projet DocAnvil
- GitHub Pages configuré pour déployer depuis **GitHub Actions** : allez dans **Settings → Pages → Build and deployment → Source** et choisissez **GitHub Actions**

Aucun secret ni jeton n'est nécessaire. Le workflow utilise les permissions Pages intégrées à GitHub.

## Définir l'URL de base

La plupart des dépôts sont publiés en tant que **site de projet** à l'adresse `https://<utilisateur>.github.io/<dépôt>/`. Toutes les pages se trouvent sous `/<dépôt>/`, il faut donc l'indiquer à DocAnvil dans `docanvil.toml` :

```toml
[build]
base_url = "/mon-depot/"
site_url = "https://mon-utilisateur.github.io/mon-depot/"
```

- **`base_url`** est le préfixe de chemin de chaque lien et ressource. Sans lui, le CSS, le JS et les liens de navigation pointent vers la racine du domaine et le site s'affiche sans style.
- **`site_url`** est l'adresse publique complète. DocAnvil l'utilise pour `sitemap.xml`, `llms.txt`, les URL canoniques et les autres balises SEO.

Si vous publiez un **site utilisateur ou d'organisation** (un dépôt nommé `<utilisateur>.github.io`) ou utilisez un domaine personnalisé, le site est servi depuis la racine : gardez `base_url = "/"` et définissez `site_url` sur cette adresse.

:::note
`docanvil serve` utilise toujours `/` comme URL de base et compile dans son propre dossier, donc les aperçus locaux fonctionnent quel que soit le `base_url` défini.
:::

## Configuration du workflow

Créez `.github/workflows/docs.yml` :

```yaml
name: Publish docs

on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Installe la dernière version de DocAnvil (somme de contrôle vérifiée)
      - name: Install DocAnvil
        run: curl -fsSL https://github.com/docanvil/docanvil/releases/latest/download/install.sh | DOCANVIL_INSTALL_DIR=/usr/local/bin sh

      - name: Build docs
        run: docanvil build --strict

      - uses: actions/upload-pages-artifact@v3
        with:
          path: dist

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

Quelques points à noter :

- **`permissions`** accorde au workflow juste ce qu'il faut pour publier : `pages: write` et `id-token: write` sont requis par `deploy-pages`.
- **`concurrency`** garantit qu'un seul déploiement s'exécute à la fois. Si vous poussez deux fois de suite, l'exécution la plus ancienne est annulée.
- **`--strict`** fait échouer la compilation au moindre avertissement : wiki-links cassés, images manquantes et problèmes similaires bloquent le déploiement avant toute mise en ligne.
- **`workflow_dispatch`** ajoute un bouton **Run workflow** dans l'onglet Actions, pratique après avoir modifié les paramètres de Pages.

## Adapter à la structure de votre projet

Le workflow ci-dessus suppose que `docanvil.toml` se trouve à la racine du dépôt. Si votre documentation est dans un sous-dossier, indiquez-le à la compilation et publiez la sortie de ce dossier :

```yaml
      - name: Build docs
        run: docanvil build --strict --path docs

      - uses: actions/upload-pages-artifact@v3
        with:
          path: docs/dist
```

Pour ne publier que lorsque la documentation change, ajoutez un filtre de chemins au déclencheur :

```yaml
on:
  push:
    branches: [main]
    paths:
      - "docs/**"
      - ".github/workflows/docs.yml"
```

## Dates de dernière mise à jour

Par défaut, `actions/checkout` ne récupère que le dernier commit. Si vous avez activé [[guides/configuration|`[last_updated]`]], toutes les pages afficheraient alors la même date : DocAnvil émet un avertissement et `--strict` fait échouer le build. Récupérez plutôt tout l'historique :

```yaml
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
```

## Domaines personnalisés

Définissez le domaine dans **Settings → Pages → Custom domain**. Comme le workflow déploie un artefact plutôt qu'une branche, aucun fichier `CNAME` n'est nécessaire dans la sortie. Mettez ensuite à jour `docanvil.toml` pour que les liens soient construits pour la racine de votre domaine :

```toml
[build]
base_url = "/"
site_url = "https://docs.example.com/"
```

## Ce qui se passe à chaque push

Lorsque vous poussez sur `main` :

1. GitHub Actions récupère votre dépôt
2. Le script d'installation télécharge la dernière version de DocAnvil depuis GitHub Releases et vérifie sa somme de contrôle
3. `docanvil build --strict` compile le site dans `dist/`, et échoue si quelque chose doit être corrigé
4. La sortie est envoyée comme artefact Pages puis déployée
5. La nouvelle version est en ligne, généralement en moins d'une minute, à l'URL affichée sur l'exécution du workflow

:::note
Le script d'installation récupère la dernière version stable à chaque fois. Si vous avez besoin de compilations reproductibles avec une version fixée, définissez aussi `DOCANVIL_VERSION`, par exemple `DOCANVIL_INSTALL_DIR=/usr/local/bin DOCANVIL_VERSION=1.2.0 sh`. Consultez [[guides/getting-started|Installation]] pour toutes les options d'installation.
:::
