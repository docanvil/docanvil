---
{
  "description": "Configurez votre projet avec docanvil.toml et organisez la barre latérale avec nav.toml."
}
---

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

[syntax]
enabled = true
theme = "base16-ocean.dark"
line_numbers = false

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

[edit]
repo = "https://github.com/org/repo"
branch = "main"

[last_updated]
enabled = true

[redirects]
"old-faq" = "help/faq"

[llms]
enabled = true
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
| `draft_links` | `"text"` | Rendu des wiki-links vers des [[writing/front-matter|pages brouillon]] dans les compilations qui les excluent. `"text"` affiche le texte du lien sans rien signaler ; `"warn"` affiche aussi un avertissement, donc `--strict` échoue |

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

### Section `[syntax]`

| Clé | Défaut | Description |
|-----|---------|-------------|
| `enabled` | `true` | Coloration syntaxique des blocs de code à la compilation |
| `theme` | `"base16-ocean.dark"` | Thème de coloration |
| `line_numbers` | `false` | Numéroter les lignes de tous les blocs de code |

Un bloc peut toujours activer ou désactiver les numéros avec `numbers` ou `numbers="false"`. Consultez [[writing/code-blocks-from-files|Blocs de code depuis des fichiers]] pour les numéros de ligne, les légendes et l'affichage de code directement depuis des fichiers sources.

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

### Section `[edit]`

Ajoute un lien « Edit this page » à chaque page, qui pointe vers sa source Markdown sur GitHub, GitLab ou Bitbucket. Les lecteurs peuvent proposer une correction depuis leur navigateur, et vous la recevez sous forme de pull request (ou merge request) à relire comme n'importe quel autre changement.

| Clé | Défaut | Description |
|-----|---------|-------------|
| `repo` | `None` | L'adresse web de votre dépôt, par ex. `"https://github.com/org/repo"`. La définir active les liens de modification |
| `branch` | `"main"` | La branche sur laquelle portent les modifications |
| `provider` | déduit | `"github"`, `"gitlab"` ou `"bitbucket"`. Nécessaire uniquement pour les instances auto-hébergées ; `github.com`, `gitlab.com` et `bitbucket.org` sont détectés automatiquement |
| `root` | détecté automatiquement | L'emplacement de votre projet DocAnvil dans le dépôt, par ex. `"docs"`. Détecté en cherchant le dossier `.git` le plus proche : vous n'en avez besoin que si vous compilez hors d'un dépôt Git |

```toml
[edit]
repo = "https://github.com/org/repo"
branch = "main"
```

Les liens pointent toujours vers le fichier à partir duquel la page a été générée : les pages traduites (`page.fr.md`) et les anciennes versions (`docs/v1/page.md`) renvoient vers leur propre source. Pour masquer le lien sur une seule page, définissez `"edit_link": false` dans son [[writing/front-matter|front matter]]. Les liens de modification n'apparaissent ni sur la page 404 ni dans les exports PDF.

Pendant que `docanvil serve` tourne, le lien devient **Open in editor** et ouvre la source de la page dans votre éditeur local, avec ou sans section `[edit]`. Voir [[reference/cli|`docanvil serve`]] pour choisir votre éditeur.

:::note{title="Git auto-hébergé"}
GitHub Enterprise et GitLab auto-hébergé fonctionnent en définissant `provider`. Bitbucket Server et Data Center ne sont pas encore pris en charge, car ils ne proposent pas de lien de modification direct. `docanvil doctor` vous avertit si vos paramètres `[edit]` ne peuvent pas produire de liens valides.
:::

### Section `[last_updated]`

Indique aux lecteurs quand chaque page a changé pour la dernière fois, pour qu'ils voient d'un coup d'œil que la documentation est entretenue. La date provient de votre historique Git et s'affiche sous la page, à côté de « Edit this page ».

| Clé | Défaut | Description |
|-----|---------|-------------|
| `enabled` | `false` | Affiche une date « Last updated » sur chaque page |
| `source` | `"git"` | D'où viennent les dates : `"git"` utilise votre historique de commits, `"front-matter"` n'utilise que les dates que vous définissez vous-même et n'exécute jamais Git |

```toml
[last_updated]
enabled = true
```

Avec `source = "git"`, la date d'une page est celle du commit le plus récent qui l'a modifiée, ou qui a modifié un fichier qu'elle intègre avec `:::include` ou `file="…"`. Mettez à jour un fragment d'instructions d'installation partagé, et toutes les pages qui l'incluent avancent aussi. DocAnvil utilise la date d'auteur du commit : rebaser ou fusionner une pull request en squash ne donne donc pas l'impression que toutes les pages ont été modifiées le jour de la fusion.

Pour fixer la date d'une page à la main, utilisez `last_updated` dans son [[writing/front-matter|front matter]] :

```markdown
---
{
  "last_updated": "2026-10-09"
}
---
```

Définissez-le à `false` pour masquer la date sur cette page.

Les dates s'affichent dans la langue de la page (« 9 octobre 2026 », « 9 October 2026 ») et alimentent aussi la balise meta `article:modified_time` de la page ainsi que les entrées `<lastmod>` de `sitemap.xml`, ce qui aide les moteurs de recherche à repérer le contenu récent. Les pages traduites et les anciennes versions reçoivent chacune la date de leur propre fichier source.

:::warning{title="Récupérez tout l'historique en CI"}
La plupart des systèmes de CI ne clonent par défaut que le dernier commit. Avec cet historique superficiel, toutes les pages afficheraient la même date : DocAnvil émet donc un avertissement et les builds `--strict` échouent. Récupérez plutôt tout l'historique : `fetch-depth: 0` sur `actions/checkout` de GitHub, ou `clone: depth: full` sur Bitbucket Pipelines. Voir [[deployment/github-pages|GitHub Pages]] et [[deployment/bitbucket-pipelines|Bitbucket Pipelines]].
:::

Bon à savoir :

- **Les fichiers renommés ou déplacés** affichent la date du commit qui les a déplacés.
- **Les modifications non commitées** ne comptent pas : une page affiche sa dernière date commitée jusqu'à ce que vous commitiez.
- **Les fichiers d'un sous-module Git** n'apportent pas de date.
- **Pas de dépôt Git ?** DocAnvil émet un avertissement et se rabat sur les dates du front matter. Définissez `source = "front-matter"` si c'est ce que vous voulez, et l'avertissement disparaît.

`docanvil doctor` signale les clones superficiels et les dépôts manquants, ainsi que tout `last_updated` du front matter qui n'est pas une date valide.

### Section `[redirects]`

Garde les anciennes URL fonctionnelles quand des pages sont renommées, déplacées ou supprimées, pour que les favoris, les résultats de recherche et les liens depuis d'autres sites mènent toujours quelque part d'utile. Chaque entrée associe un ancien chemin à sa nouvelle destination :

| Clé | Défaut | Description |
|-----|---------|-------------|
| `"ancien/chemin"` | — | Autant d'entrées que nécessaire : l'ancien chemin à gauche, le nouveau à droite |
| `unprefixed` | `false` | Redirige chaque `page.html` vers la version actuelle dans la langue par défaut (voir plus bas) |

```toml
[redirects]
"old-faq" = "help/faq"
"getting-started/install" = "guides/install"
"/blog/" = "https://blog.example.com"
```

Les chemins s'écrivent de trois façons :

- **Un slug de page**, comme `help/faq` : la même chose que dans un [[writing/wiki-links|wiki-link]]. Un `.html` ou un `/` final ne pose pas de problème. Sur un site traduit ou versionné, la redirection est écrite pour chaque langue et chaque version où la page cible existe : `"old-faq" = "help/faq"` donne `/en/old-faq.html`, `/fr/old-faq.html`, et ainsi de suite.
- **Un chemin du site** commençant par `/`, comme `/old-blog.html`, ou `/blog/` pour `/blog/index.html`. Il est utilisé tel quel, une seule fois.
- **Un autre site** : une cible contenant `://`, comme `https://blog.example.com`. Uniquement à droite.

Chaque ancienne URL devient une petite page qui renvoie aussitôt le lecteur vers la nouvelle, en gardant l'éventuelle `#section` du lien. Elle indique aussi aux moteurs de recherche où la page est partie, pour que son référencement passe à la nouvelle URL. Cela fonctionne sur n'importe quel hébergement statique, sans rien configurer côté serveur, et les pages de redirection n'apparaissent jamais dans la barre latérale, la recherche ou `sitemap.xml`.

Pour une page que vous avez renommée ou déplacée, le plus simple est souvent d'indiquer l'ancien chemin dans son propre [[writing/front-matter|front matter]] avec `redirect_from`. Il suit la page et couvre toutes ses traductions. La table `[redirects]` convient aux pages supprimées, ou pour déplacer beaucoup de pages d'un coup. Si les deux envoient le même ancien chemin à des endroits différents, le front matter l'emporte.

**Activer les langues ou les versions** déplace toutes les pages : `/guides/install.html` devient `/en/guides/install.html` ou `/v2/guides/install.html`. Avec `unprefixed = true`, DocAnvil laisse une redirection à chaque ancienne adresse, vers la version actuelle dans la langue par défaut :

```toml
[redirects]
unprefixed = true
```

:::note
`unprefixed` partage la table avec vos chemins : une page dont le slug est littéralement `unprefixed` doit utiliser la forme chemin du site, `"/unprefixed.html" = "…"`.
:::

Une redirection vers une page qui n'existe pas, une redirection dont l'ancien chemin est encore une vraie page (la page l'emporte toujours), deux redirections qui envoient le même chemin à des endroits différents, et des redirections qui tournent en boucle affichent chacune un avertissement à la compilation : `docanvil build --strict` échoue donc. `docanvil doctor` vérifie toutes les redirections à l'avance et indique la ligne concernée dans `docanvil.toml` ou dans le front matter de la page. Une redirection vers une [[writing/front-matter|page brouillon]] est ignorée sans avertissement jusqu'à la publication de la page.

### Section `[llms]`

Écrit des fichiers [llms.txt](https://llmstxt.org) : une carte de votre documentation en Markdown simple, que les outils et assistants d'IA peuvent lire au lieu d'analyser votre HTML. Pratique quand on interroge un assistant sur votre projet, ou qu'on confie votre documentation à un agent de code.

| Clé | Défaut | Description |
|-----|---------|-------------|
| `enabled` | `false` | Écrit `llms.txt` (et `llms-full.txt`) avec `docanvil build` |
| `description` | — | Un résumé du projet en une ligne, affiché sous son nom |
| `full` | `true` | Écrit aussi `llms-full.txt`, avec le Markdown de toutes les pages dans un seul fichier. Définissez à `false` pour n'avoir que l'index |

```toml
[llms]
enabled = true
description = "DocAnvil transforme le Markdown en sites de documentation rapides et faciles à parcourir."
```

`llms.txt` commence par le nom du projet et sa `description`, puis liste chaque page sous forme de lien, regroupées comme dans votre barre latérale. La `description` du front matter de chaque page s'affiche à côté de son lien : cela vaut la peine de les écrire.

```markdown
# Ma documentation

> DocAnvil transforme le Markdown en sites de documentation rapides et faciles à parcourir.

## Guides

- [Installation](https://docs.example.com/guides/install.html): Installer DocAnvil sur macOS, Linux ou Windows
- [Configuration](https://docs.example.com/guides/configuration.html)
```

`llms-full.txt` contient le Markdown de toutes les pages, dans le même ordre, chacune précédée de son titre et d'un lien `Source:`. C'est votre Markdown tel que vous l'avez écrit, mis au propre pour que d'autres outils puissent le lire :

- les [[writing/includes|fragments]] `:::include` et les [[writing/code-blocks-from-files|blocs de code]] `file="…"` sont remplis
- les [[writing/wiki-links|wiki-links]] deviennent des liens Markdown classiques avec des URL complètes
- les ajouts propres à DocAnvil, comme `{#id-perso}` sur les titres ou `numbers` sur les blocs de code, sont retirés
- les blocs de code restent exactement tels qu'écrits, et les [[writing/components|composants]] apparaissent avec leur syntaxe `:::`

Définissez `site_url` sous `[build]` pour que les liens soient des URL complètes que les outils d'IA peuvent suivre. Sans elle, ils sont relatifs, et `docanvil doctor` vous le rappelle.

**Les sites traduits et versionnés** ont un `llms.txt` et un `llms-full.txt` dans chaque dossier de langue et de version (`/fr/llms.txt`, `/v2/en/llms.txt`). Les outils d'IA cherchent `/llms.txt` à la racine du site : celui-là couvre la version actuelle dans la langue par défaut, et se termine par des liens vers tous les autres.

Les [[writing/front-matter|pages brouillon]] sont exclues, et vous pouvez exclure n'importe quelle autre page avec `"llms": false` dans son front matter. `docanvil serve` n'écrit pas ces fichiers.

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

Par défaut, le libellé dans la barre latérale est le titre de la page — son premier `# Titre`, ou le slug à défaut (`getting-started` devient "Getting Started"). Remplacez-le avec `label` :

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
