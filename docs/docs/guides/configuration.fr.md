# Configuration

DocAnvil utilise deux fichiers de configuration à la racine de votre projet : `docanvil.toml` pour les paramètres du projet et `nav.toml` pour la structure de navigation.

## docanvil.toml

::::tabs
:::tab{title="Minimal"}
```toml
[project]
name = "Mes Docs"
```
:::
:::tab{title="Complet"}
```toml
[project]
name = "Mes Docs"
content_dir = "docs"

[build]
output_dir = "dist"
base_url = "/mon-projet/"

[theme]
custom_css = "theme/custom.css"
color_mode = "both"

[theme.variables]
color-primary = "#059669"
font-body = "Georgia, serif"

[search]
enabled = true

[charts]
enabled = true
mermaid_version = "11"

[locale]
default = "en"
enabled = ["en", "fr"]
auto_detect = true

[locale.display_names]
en = "English"
fr = "Français"

[locale.flags]
en = "🇺🇸"

[version]
current = "v2"
enabled = ["v1", "v2"]

[version.display_names]
v1 = "v1.0"
v2 = "v2.0 (latest)"

[pdf]
cover_page = true
author = "Votre nom"
paper_size = "A4"

[doctor]
max_paragraph_words = 150
heading_adjacent_separator = true
```
:::
::::

:::warning{title="Champ obligatoire"}
Le champ `name` sous `[project]` est obligatoire. DocAnvil ne pourra pas démarrer sans lui.
:::

### Section `[project]`

| Clé | Défaut | Description |
|-----|---------|-------------|
| `name` | *(obligatoire)* | Nom du projet affiché dans la barre latérale et les titres de pages |
| `content_dir` | `"docs"` | Répertoire contenant vos fichiers Markdown |

### Section `[build]`

| Clé | Défaut | Description |
|-----|---------|-------------|
| `output_dir` | `"dist"` | Répertoire où le site statique est généré |
| `base_url` | `"/"` | Préfixe de chemin URL pour les déploiements dans des sous-répertoires (ex. `"/mon-projet/"`) |
| `site_url` | `None` | URL complète du site (ex. `"https://exemple.com/"`) pour les URLs canoniques, les balises hreflang, et le sitemap |

:::note{title="Recommandé pour l'i18n"}
Définir `site_url` est fortement recommandé lors de l'utilisation de la localisation. Cela permet les URLs hreflang absolues, les balises `<link>` canoniques, et les balises meta `og:url` — toutes importantes pour le SEO multilingue.
:::

### Section `[theme]`

| Clé | Défaut | Description |
|-----|---------|-------------|
| `name` | `None` | Réservé pour la sélection de thème future |
| `custom_css` | `None` | Chemin vers un fichier CSS personnalisé chargé après le thème par défaut |
| `color_mode` | `"light"` | Mode de couleur : `"light"`, `"dark"`, ou `"both"` (clair + sombre avec bascule) |
| `variables` | `{}` | Surcharges de variables CSS injectées en tant que propriétés `:root` |

Les variables sont spécifiées sous forme de paires clé-valeur où la clé est le nom de la variable CSS (sans `--`) et la valeur est n'importe quelle valeur CSS valide :

```toml
[theme.variables]
color-primary = "#059669"
color-bg = "#fafafa"
font-body = "Inter, sans-serif"
content-max-width = "960px"
```

Consultez [[reference/css-variables|Variables CSS]] pour la liste complète des variables disponibles.

### Section `[search]`

| Clé | Défaut | Description |
|-----|---------|-------------|
| `enabled` | `true` | Activer ou désactiver la recherche plein texte |

Lorsqu'elle est activée, DocAnvil génère un fichier `search-index.json` à la compilation et ajoute un champ de recherche dans l'en-tête. La recherche est propulsée par MiniSearch.js, chargé depuis un CDN à la première utilisation. Définissez `enabled = false` pour supprimer l'interface de recherche et passer la génération de l'index.

### Section `[charts]`

| Clé | Défaut | Description |
|-----|---------|-------------|
| `enabled` | `true` | Activer ou désactiver le rendu des diagrammes Mermaid |
| `mermaid_version` | `"11"` | Version majeure de Mermaid.js à charger depuis le CDN |

Lorsqu'il est activé, les pages contenant des blocs `:::mermaid` chargeront Mermaid.js et rendront les diagrammes côté client. Lorsqu'il est désactivé, le contenu `:::mermaid` est rendu comme du texte préformaté.

### Section `[locale]`

| Clé | Défaut | Description |
|-----|---------|-------------|
| `default` | `None` | Code de locale par défaut (ex. `"fr"`). Requis pour activer l'i18n. |
| `enabled` | `[]` | Liste des codes de locale activés (ex. `["en", "fr", "de"]`) |
| `auto_detect` | `true` | Détecter automatiquement la langue du navigateur et rediriger à la première visite |
| `display_names` | `{}` | Noms lisibles pour les locales affichés dans le sélecteur de langue |
| `flags` | `{}` | Surcharges d'emoji de drapeau pour les locales (ex. `{"en": "🇺🇸"}` pour utiliser le drapeau américain) |

Lorsque `default` et `enabled` sont tous les deux définis, DocAnvil passe en mode multilingue : chaque locale obtient son propre préfixe d'URL (`/en/`, `/fr/`), sa propre navigation et son propre index de recherche, et un sélecteur de langue apparaît dans l'en-tête.

```toml
[locale]
default = "en"
enabled = ["en", "fr", "de"]
auto_detect = true

[locale.display_names]
en = "English"
fr = "Français"
de = "Deutsch"

[locale.flags]
en = "🇺🇸"    # Utiliser le drapeau américain plutôt que le drapeau britannique par défaut
```

:::note{title="Besoin de détails ?"}
Consultez [[guides/localisation|Localisation]] pour un guide complet sur la mise en place de docs multilingues, incluant le nommage des fichiers, la navigation par locale, et la couverture des traductions.
:::

### Section `[version]`

| Clé | Défaut | Description |
|-----|---------|-------------|
| `current` | *(dernier de `enabled`)* | Le code de version actuelle/dernière — utilisé pour la redirection racine et la bannière de version obsolète. Par défaut, le dernier élément de `enabled` si non défini. |
| `enabled` | `[]` | Liste des noms de répertoires de versions à compiler (ex. `["v1", "v2"]`). Chacun doit avoir un sous-répertoire correspondant dans le répertoire de contenu. |
| `display_names` | `{}` | Noms lisibles affichés dans le sélecteur de version (ex. `{"v2": "v2.0 (latest)"}`) |

Lorsque `enabled` est non vide, DocAnvil passe en mode multi-version : chaque version obtient son propre préfixe d'URL (`/v1/`, `/v2/`), sa propre navigation et son propre index de recherche, et un sélecteur de version apparaît dans l'en-tête. Les pages des versions antérieures affichent automatiquement une bannière redirigeant vers la dernière version.

```toml
[version]
current = "v2"
enabled = ["v1", "v2"]

[version.display_names]
v1 = "v1.0"
v2 = "v2.0 (latest)"
```

:::note{title="Besoin de détails ?"}
Consultez [[guides/versioning|Versionnement]] pour un guide complet sur la mise en place de docs multi-versions, incluant l'organisation des répertoires, la navigation par version, le sélecteur de version, et la combinaison avec l'i18n.
:::

### Section `[pdf]`

| Clé | Défaut | Description |
|-----|---------|-------------|
| `author` | `None` | Nom de l'auteur affiché sur la page de couverture et dans l'en-tête courant |
| `cover_page` | `false` | Ajouter une page de titre avec le nom du projet et l'auteur avant la table des matières |
| `paper_size` | `"A4"` | Format de papier : `"A3"`, `"A4"`, `"A5"`, `"Letter"`, `"Legal"`, `"Tabloid"` (insensible à la casse) |
| `custom_css` | `None` | Chemin (relatif à la racine du projet) vers un fichier CSS injecté dans le PDF |

:::note{title="Besoin de détails ?"}
Consultez [[guides/pdf-export|Export PDF]] pour le guide complet : pages de couverture, formats de papier, support RTL, export par locale, et CSS personnalisé.
:::

### Section `[doctor]`

| Clé | Défaut | Description |
|-----|---------|-------------|
| `max_paragraph_words` | `150` | Seuil de nombre de mots pour la vérification de lisibilité `long-paragraph`. Utilisez `0` pour désactiver la vérification entièrement. |
| `heading_adjacent_separator` | `true` | Avertit quand un titre est directement adjacent à une règle horizontale. Définissez à `false` pour désactiver. |

La section `[doctor]` configure le linter de lisibilité `docanvil doctor`. Les paramètres par défaut sont intentionnellement permissifs — réduisez le seuil pour des standards d'écriture plus stricts.

```toml
[doctor]
max_paragraph_words = 100              # Signaler les paragraphes de plus de 100 mots
heading_adjacent_separator = false     # Désactiver la vérification séparateur adjacent à un titre
```

:::note{title="Besoin de détails ?"}
Consultez [[reference/cli|Commandes CLI → Vérifications de lisibilité]] pour la liste complète des vérifications, leurs niveaux de sévérité, et ce que chacune détecte.
:::

## nav.toml

Le fichier de navigation contrôle la structure de la barre latérale. Il utilise la syntaxe de tableaux d'objets de TOML et prend en charge les pages, les séparateurs et les groupes.

### Entrées de page

L'entrée la plus simple pointe vers une page par son slug (le chemin du fichier relatif à `content_dir`, sans l'extension `.md`) :

<pre><code class="language-toml">&#91;[nav]]
page = "index"

&#91;[nav]]
page = "guides/getting-started"
</code></pre>

### Surcharges de libellés

Par défaut, le libellé dans la barre latérale est dérivé du slug (`getting-started` devient "Getting Started"). Remplacez-le avec `label` :

<pre><code class="language-toml">&#91;[nav]]
page = "guides/getting-started"
label = "Installation"
</code></pre>

### Séparateurs

Ajoutez des séparateurs visuels entre les sections. Un séparateur avec libellé affiche du texte :

<pre><code class="language-toml">&#91;[nav]]
separator = "Guides"
</code></pre>

Un séparateur sans libellé trace une ligne horizontale :

<pre><code class="language-toml">&#91;[nav]]
separator = true
</code></pre>

### Groupes

Les groupes créent des sections réductibles dans la barre latérale. Chaque groupe a un `label` et un tableau d'enfants dans `group` :

<pre><code class="language-toml">&#91;[nav]]
label = "Référence"
group = [
  { page = "reference/cli", label = "Commandes CLI" },
  { page = "reference/project-structure" },
  { page = "reference/css-variables", label = "Variables CSS" },
]
</code></pre>

### En-têtes de groupe cliquables

Ajoutez un champ `page` pour rendre l'en-tête du groupe lui-même un lien cliquable :

<pre><code class="language-toml">&#91;[nav]]
label = "Écrire du contenu"
page = "writing/markdown"
group = [
  { page = "writing/wiki-links", label = "Liens &amp; Popovers" },
  { page = "writing/components" },
]
</code></pre>

Cliquer sur "Écrire du contenu" navigue vers la page Markdown, tandis que la flèche développe le groupe.

### Séparateurs enfants

Vous pouvez ajouter des séparateurs à l'intérieur des groupes pour organiser les enfants :

<pre><code class="language-toml">&#91;[nav]]
label = "Référence"
group = [
  { page = "reference/cli", label = "Commandes CLI" },
  { separator = "Projet" },
  { page = "reference/project-structure" },
  { page = "reference/css-variables", label = "Variables CSS" },
]
</code></pre>

### Découverte automatique

Vous pouvez utiliser l'option de découverte automatique pour ajouter sélectivement un dossier à la navigation :

<pre><code class="language-toml">&#91;[nav]]
autodiscover = "api"
</code></pre>

Vous pouvez aussi utiliser la découverte automatique avec un groupe réductible :

<pre><code class="language-toml">&#91;[nav]]
label = "Référence"
autodiscover = "reference"
</code></pre>

### Découverte automatique par défaut

Si `nav.toml` est absent, DocAnvil découvre automatiquement tous les fichiers `.md` du répertoire de contenu et construit la navigation à partir de la structure des répertoires. Les fichiers sont triés alphabétiquement et les noms de répertoires deviennent des libellés de groupe.

## Pages associées

- [[guides/theming|Thèmes]] — variables CSS, feuilles de style personnalisées, et surcharges de templates
- [[guides/pdf-export|Export PDF]] — guide complet d'export PDF
- [[guides/versioning|Versionnement]] — mise en place de docs multi-versions et sélecteur de version
- [[reference/project-structure|Structure du projet]] — comment les fichiers sont mappés aux pages et aux slugs

:::note
L'en-tête comprend un champ de filtre qui recherche les libellés de pages en temps réel. Cela fonctionne avec n'importe quelle structure de navigation.
:::
