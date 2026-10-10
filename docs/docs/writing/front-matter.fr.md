---
{
  "title": "Front Matter",
  "description": "Ajoutez des métadonnées à vos pages avec le front matter JSON pour les titres, le SEO, et plus encore"
}
---

# Front Matter

Le front matter est un bloc de métadonnées JSON placé en haut d'un fichier Markdown, délimité par `---`. DocAnvil l'analyse pour définir les titres de pages, générer des balises meta SEO, et renseigner les métadonnées Open Graph dans le HTML généré.

## Syntaxe de base

Placez un bloc JSON tout au début de votre fichier Markdown :

```markdown
---
{
  "title": "Démarrage rapide",
  "description": "Apprendre à installer et configurer DocAnvil",
  "author": "Jane Doe",
  "date": "2024-01-15"
}
---

Le contenu de la page commence ici.
```

Le bloc front matter est supprimé du rendu final — il n'affecte que les métadonnées.

## Champs pris en charge

Tous les champs sont optionnels. Vous pouvez en inclure n'importe quelle combinaison, ou omettre complètement le front matter.

| Champ | Type | Effet |
|-------|------|--------|
| `title` | Chaîne | Remplace le titre de la page dans l'onglet du navigateur, la barre de navigation, l'index de recherche, les fils d'Ariane, et le slug de l'URL |
| `slug` | Chaîne | Remplace directement le slug de l'URL — prioritaire sur le slug dérivé du titre |
| `description` | Chaîne | Affichée en sous-titre sous le titre de la page, et génère les balises `<meta name="description">` et `<meta property="og:description">` pour les moteurs de recherche et les aperçus de liens |
| `author` | Chaîne | Génère la balise `<meta name="author">` |
| `date` | Chaîne | Génère la balise `<meta property="article:published_time">` pour les moteurs de recherche et le partage social |
| `edit_link` | Booléen | Définissez à `false` pour masquer le lien « Edit this page » (et « Open in editor » sous `docanvil serve`) sur cette page (voir `[edit]` dans [[guides/configuration\|Configuration]]) |
| `last_updated` | Chaîne ou Booléen | La date de dernière modification de la page, au format `"YYYY-MM-DD"`. Remplace la date tirée de l'historique Git. Définissez à `false` pour masquer la date sur cette page (voir `[last_updated]` dans [[guides/configuration\|Configuration]]) |
| `draft` | Booléen | Définissez à `true` pour garder la page hors des compilations de production pendant que vous y travaillez (voir [Pages brouillon](#pages-brouillon)) |
| `redirect_from` | Liste de chaînes | Anciens chemins qui doivent renvoyer les lecteurs vers cette page, pour que les liens existants continuent de fonctionner (voir [Redirections](#redirections)) |
| `llms` | Booléen | Définissez à `false` pour exclure cette page de `llms.txt` et `llms-full.txt` (voir `[llms]` dans [[guides/configuration\|Configuration]]) |

Les champs inconnus sont ignorés silencieusement — vous pouvez ajouter vos propres métadonnées personnalisées sans déclencher d'erreurs.

## Remplacement du titre

Par défaut, le titre d'une page est son premier `# Titre`, ou, à défaut, son nom de fichier (`getting-started.md` devient "Getting Started"). Le `title` du front matter remplace ce comportement partout :

- La balise `<title>` dans l'en-tête HTML
- Le libellé dans la barre de navigation latérale
- L'index de recherche
- Les fils d'Ariane
- Le slug de l'URL et le nom du fichier de sortie

```markdown
---
{
  "title": "Guide de démarrage rapide"
}
---

# Démarrer avec DocAnvil

Contenu ici...
```

Dans cet exemple, la barre latérale et l'onglet du navigateur affichent "Guide de démarrage rapide", tandis que le contenu affiche son propre titre `# Démarrer avec DocAnvil`.

:::note
La plupart du temps, `title` est inutile — le `# Titre` nomme déjà la page partout. Utilisez-le pour un libellé plus court que le titre, ou une URL basée sur un autre nom. Un titre issu du `# Titre` ne change jamais l'URL : vous pouvez reformuler vos titres librement.
:::

### URLs propres à partir des titres

Lorsqu'un `title` est défini, le slug de l'URL est dérivé du titre plutôt que du nom de fichier. C'est particulièrement utile pour les fichiers avec des préfixes organisationnels :

| Nom de fichier | Titre | URL de sortie |
|----------|-------|------------|
| `01-introduction.md` | `"Introduction"` | `/introduction.html` |
| `03-setup-guide.md` | `"Guide d'installation"` | `/guide-dinstallation.html` |
| `guides/01-basics.md` | `"Les bases"` | `/guides/les-bases.html` |

Le préfixe de répertoire est toujours préservé — seule la partie du nom de fichier change.

:::note{title="Les pages index sont exemptées"}
Les pages nommées `index.md` conservent leur slug quelle que soit la valeur du champ `title`. L'URL `index` est une convention bien établie et n'est jamais remplacée par le titre. Utilisez le champ explicite `slug` si vous avez besoin de la modifier.
:::

## Remplacement du slug

Pour un contrôle total sur l'URL de sortie, utilisez le champ `slug`. Il est prioritaire sur le nom de fichier et le slug dérivé du titre.

```markdown
---
{
  "title": "Démarrer avec DocAnvil",
  "slug": "quickstart"
}
---
```

Cette page sera écrite à `/quickstart.html` tout en affichant "Démarrer avec DocAnvil" comme titre.

La valeur `slug` est automatiquement normalisée en format compatible avec les URLs — les espaces deviennent des tirets et les caractères spéciaux sont supprimés.

### Liens rétrocompatibles

Lorsqu'un slug change (via `title` ou `slug`), les wiki-links utilisant l'ancien slug basé sur le nom de fichier continuent de fonctionner. Par exemple, si `01-setup.md` reçoit le titre "Guide d'installation", `01-setup` et `guide-dinstallation` pointent tous deux vers la même page.

Cela couvre les liens internes à votre documentation. L'ancienne URL, elle, disparaît : si la page était déjà publiée, ajoutez l'ancien slug à [`redirect_from`](#redirections) pour que les favoris et les liens depuis d'autres sites continuent de fonctionner.

## Pages brouillon

Marquez une page comme brouillon pour continuer à y travailler ouvertement — la committer, la relire, la prévisualiser — sans la publier tout de suite :

```markdown
---
{
  "draft": true
}
---

# Nouveau guide de déploiement

En cours de rédaction...
```

- **`docanvil serve`** affiche les brouillons comme n'importe quelle page, avec un bandeau **Draft** en haut pour ne pas l'oublier.
- **`docanvil build`** exclut complètement les brouillons : pas de fichier HTML, et aucune entrée dans la barre latérale, la recherche, le sitemap ou les liens précédent/suivant. La sortie de la compilation indique combien ont été ignorés.
- **`docanvil export pdf`** les exclut également.

Quand la page est prête, supprimez `"draft": true` (ou mettez-le à `false`) et elle sera publiée à la prochaine compilation.

### Liens et navigation vers les brouillons

Pas besoin de faire le ménage autour d'un brouillon avant de compiler :

- Une entrée de `nav.toml` qui pointe vers un brouillon est ignorée. Un groupe dont toutes les pages sont des brouillons disparaît jusqu'à ce que l'une d'elles soit publiée.
- Un wiki-link d'une page publiée vers un brouillon affiche son texte sans lien, pour que les lecteurs ne tombent jamais sur une page manquante.
- Une traduction en brouillon (par exemple `guide.fr.md`) n'est pas signalée comme traduction manquante.

Si vous préférez repérer les liens vers des pages non publiées, définissez `draft_links = "warn"` dans la section `[build]` de votre [[guides/configuration|configuration]]. Ces liens affichent alors un avertissement, et `docanvil build --strict` échoue.

### Prévisualiser les brouillons

`docanvil build --drafts` inclut les brouillons dans une compilation statique, pratique pour les déploiements de prévisualisation d'une pull request. Les brouillons gardent leur bandeau et reçoivent une balise `noindex`, pour que les moteurs de recherche les ignorent même si la prévisualisation est publique.

:::note
Les brouillons concernent la publication, pas la confidentialité. Le Markdown reste dans votre dépôt, donc n'utilisez pas `draft` pour ce qui doit rester secret.
:::

## Redirections

Quand vous renommez ou déplacez une page, son ancienne URL cesse de fonctionner : les favoris, les résultats de recherche et les liens depuis d'autres sites aboutissent sur la page 404. Listez les anciens chemins dans le front matter de la page, et DocAnvil laisse une redirection à chacun d'eux :

```markdown
---
{
  "redirect_from": ["setup", "getting-started/install"]
}
---

# Installation
```

Chaque entrée est un ancien slug : l'ancien chemin de la page depuis le dossier de contenu, sans `.md`, comme dans un wiki-link. Il n'est pas relatif au dossier de la page : si `setup.md` est devenu `guides/install.md`, l'entrée est `"setup"`. Une chaîne seule fonctionne aussi : `"redirect_from": "setup"`.

- **Pages traduites :** mettez `redirect_from` sur une seule traduction (en général la langue par défaut) et chaque langue reçoit sa propre redirection. `/en/setup.html` mène à la page anglaise et `/fr/setup.html` à la page française.
- **Sites versionnés :** la redirection reste dans la version de la page. `docs/v2/guides/install.md` avec `"redirect_from": ["setup"]` redirige `/v2/setup.html`.
- **Chemins du site :** une entrée commençant par `/`, comme `"/old/install.html"`, est utilisée telle quelle et mène à la page dans la version actuelle et la langue par défaut.

Les lecteurs arrivent aussitôt sur la page, à la même `#section` si l'ancien lien en avait une, et les moteurs de recherche apprennent que la page a déménagé. Pour les redirections qui ne concernent pas une seule page (pages supprimées, déplacements en masse, liens vers un autre site) et pour garder les URL fonctionnelles en activant les langues ou les versions, voir `[redirects]` dans [[guides/configuration|Configuration]].

:::note
Une redirection ne remplace jamais une vraie page. Si un chemin de `redirect_from` est encore une page, la page reste et la compilation vous avertit.
:::

## Balises meta SEO

Lorsque des champs de front matter sont présents, DocAnvil génère les balises HTML meta correspondantes dans le `<head>` de la page :

```html
<meta name="description" content="Apprendre à installer et configurer DocAnvil">
<meta property="og:description" content="Apprendre à installer et configurer DocAnvil">
<meta name="author" content="Jane Doe">
<meta property="article:published_time" content="2024-01-15">
```

Chaque page reçoit également automatiquement ces balises Open Graph, indépendamment du front matter :

```html
<meta property="og:title" content="Démarrage rapide">
<meta property="og:type" content="article">
```

## Exemples

Quelques modèles de front matter courants pour vous lancer.

### Minimal — titre uniquement

Un simple titre suffit pour remplacer le titre de la page (issu de son `# Titre` ou de son nom de fichier) et définir la balise `<title>`.

```markdown
---
{
  "title": "Référence API"
}
---
```

### Métadonnées complètes

Incluez `description`, `author` et `date` pour renseigner les balises Open Graph et `<meta>`.

```markdown
---
{
  "title": "Guide de déploiement",
  "description": "Déployer votre site DocAnvil sur Netlify, Vercel, ou GitHub Pages",
  "author": "Équipe Documentation",
  "date": "2024-06-01"
}
---
```

### Slug personnalisé

Utilisez `slug` pour contrôler l'URL de sortie, indépendamment du nom de fichier.

```markdown
---
{
  "title": "Foire aux questions",
  "slug": "faq"
}
---
```

La sortie sera `/faq.html` au lieu de `/foire-aux-questions.html`.

### Sans front matter

Les pages sans front matter fonctionnent exactement comme avant — le titre vient du premier `# Titre` (ou du nom de fichier) et aucune balise meta supplémentaire n'est ajoutée.

## Format de date

Le champ `date` est transmis tel quel à la balise meta `article:published_time`. Le format ISO 8601 (`YYYY-MM-DD`) est recommandé pour une meilleure compatibilité avec les moteurs de recherche et les plateformes sociales.

`date` indique quand une page a été publiée ; `last_updated` indique quand elle a changé pour la dernière fois. `last_updated` doit être une vraie date `YYYY-MM-DD` (ou `false`). Toute autre valeur est ignorée, et `docanvil doctor` la signale comme une erreur.
