---
{
  "title": "Structure du projet",
  "slug": "project-structure"
}
---
# Structure du projet

Un projet DocAnvil a une organisation de répertoires simple. Voici la structure complète :

```text
mon-projet/
  docanvil.toml           # Configuration du projet
  nav.toml                # Structure de navigation (optionnel)
  docs/                   # Répertoire de contenu (configurable)
    index.md              # Page d'accueil
    guides/
      getting-started.md
      configuration.md
    reference/
      cli.md
  theme/                  # Personnalisation du thème
    custom.css            # Feuille de style personnalisée
    templates/            # Surcharges de templates (optionnel)
      layout.html
  dist/                   # Sortie de compilation (générée)
    index.html
    search-index.json     # Index de recherche plein texte (quand la recherche est activée)
    guides/
      getting-started.html
    theme/
      custom.css
```

## Fichiers et répertoires

Voici le rôle de chaque partie du projet.

### `docanvil.toml`

Le fichier de configuration principal. Contient le nom du projet, le chemin du répertoire de contenu, le chemin de sortie, et les paramètres de thème. Consultez [[guides/configuration|Configuration]] pour la référence complète.

### `nav.toml`

Contrôle la structure de navigation de la barre latérale. Définit l'ordre des pages, les séparateurs, les groupes, et les surcharges de libellés. Si ce fichier est absent, DocAnvil découvre automatiquement les pages depuis le répertoire de contenu et construit la navigation alphabétiquement.

### Répertoire de contenu

Le répertoire `docs/` (configurable via `content_dir` dans `docanvil.toml`) contient tous vos fichiers Markdown sources. Tout fichier `.md` placé ici devient une page de la documentation.

Les sous-répertoires créent des segments de chemin URL. La structure de répertoires est directement mappée à la structure de sortie.

### Répertoire de thème

Le répertoire `theme/` contient les fichiers de personnalisation :

- `custom.css` — vos surcharges CSS, chargées après le thème par défaut
- `templates/layout.html` — surcharge de template complète optionnelle utilisant Tera

### Répertoire de sortie

Le répertoire `dist/` (configurable via `output_dir`) est généré par `docanvil build`. Il contient le site statique complet prêt pour le déploiement, incluant un fichier `search-index.json` quand la recherche est activée. Utilisez `--clean` pour le supprimer avant de recompiler.

### Organisation i18n

Quand la localisation est activée, les fichiers de contenu utilisent des suffixes de locale et la sortie est répartie dans des répertoires par locale :

```text
mon-projet/
  docanvil.toml           # Inclut la section [locale]
  nav.toml                # Navigation par défaut (repli)
  nav.fr.toml             # Navigation spécifique au français (optionnel)
  docs/
    index.en.md           # Page d'accueil en anglais
    index.fr.md           # Page d'accueil en français
    guides/
      getting-started.en.md
      getting-started.fr.md
  dist/                   # Sortie de compilation
    js/docanvil.js         # Partagé entre les locales
    robots.txt
    sitemap.xml            # Contient toutes les locales
    404.html
    en/
      index.html
      guides/
        getting-started.html
      search-index.json    # Index de recherche anglais
    fr/
      index.html
      guides/
        getting-started.html
      search-index.json    # Index de recherche français
```

Consultez [[guides/localisation|Localisation]] pour un guide complet sur la mise en place de docs multilingues.

## Découverte des pages

DocAnvil découvre les pages en parcourant récursivement le répertoire de contenu et en collectant tous les fichiers `.md`. Chaque fichier devient une page avec un slug, un titre, et un chemin de sortie.

### Dérivation des slugs

Le slug est le chemin du fichier relatif au répertoire de contenu, avec l'extension `.md` supprimée et les antislashs normalisés en barres obliques :

| Fichier source | Slug | Fichier de sortie | Titre |
|-------------|------|-------------|-------|
| `docs/index.md` | `index` | `index.html` | Home |
| `docs/guides/getting-started.md` | `guides/getting-started` | `guides/getting-started.html` | Getting Started |
| `docs/reference/cli.md` | `reference/cli` | `reference/cli.html` | Cli |
| `docs/writing/wiki-links.md` | `writing/wiki-links` | `writing/wiki-links.html` | Wiki Links |

### Surcharges de slugs par front matter

Quand une page a un `title` dans son front matter, le slug est dérivé du titre plutôt que du nom de fichier. Un champ front matter `slug` explicite est prioritaire sur les deux. Cela vous permet d'utiliser des préfixes organisationnels dans les noms de fichiers tout en conservant des URLs propres :

| Fichier source | Front Matter | Slug | Fichier de sortie |
|-------------|-------------|------|-------------|
| `docs/01-intro.md` | `{"title": "Introduction"}` | `introduction` | `introduction.html` |
| `docs/guides/01-setup.md` | `{"title": "Guide d'installation"}` | `guides/guide-dinstallation` | `guides/guide-dinstallation.html` |
| `docs/faq-page.md` | `{"slug": "faq"}` | `faq` | `faq.html` |

Les préfixes de répertoire sont préservés — seule la partie du nom de fichier change. Les pages nommées `index.md` sont exemptées des slugs dérivés du titre. Consultez [[writing/front-matter|Front Matter]] pour tous les détails.

Les wiki-links utilisant l'ancien slug basé sur le nom de fichier continuent de se résoudre après une surcharge de slug.

### Slugs tenant compte de la locale

Quand l'i18n est activé, le suffixe de locale est supprimé avant la construction du slug. La locale est suivie séparément et le chemin de sortie inclut un préfixe de locale :

| Fichier source | Slug | Locale | Fichier de sortie |
|-------------|------|--------|-------------|
| `docs/index.en.md` | `index` | `en` | `en/index.html` |
| `docs/index.fr.md` | `index` | `fr` | `fr/index.html` |
| `docs/guides/setup.en.md` | `guides/setup` | `en` | `en/guides/setup.html` |
| `docs/guide.md` | `guide` | *(défaut)* | `en/guide.html` |

Les fichiers sans suffixe de locale reçoivent la locale par défaut. Consultez [[guides/localisation|Localisation]] pour les détails.

### Génération des titres

Les titres sont dérivés du dernier composant du chemin du slug :

- `index` devient "Home"
- Les tirets et underscores deviennent des espaces
- Chaque mot est mis en majuscule (`getting-started` → "Getting Started")

Les titres apparaissent dans la navigation de la barre latérale (à moins d'être remplacés par `label` dans `nav.toml`) et dans la balise `<title>` de la page.

### Navigation par découverte automatique

Quand `nav.toml` est absent, l'arbre de navigation est construit à partir de la structure des répertoires :

- Les fichiers `.md` de premier niveau deviennent des éléments nav racines
- Les sous-répertoires deviennent des groupes réductibles, étiquetés avec le nom du répertoire en title case
- Les fichiers dans les répertoires deviennent les enfants de leur groupe
- Tout est trié alphabétiquement

:::note
Les fichiers sont triés par chemin lors de la découverte, garantissant un ordre de navigation déterministe quelle que soit l'ordination du système de fichiers.
:::

## Pages associées

- [[guides/configuration|Configuration]] — options `docanvil.toml` et `nav.toml`
- [[guides/theming|Thèmes]] — variables CSS, feuilles de style personnalisées, et surcharges de templates
- [[guides/getting-started|Installation]] — créer votre premier projet
