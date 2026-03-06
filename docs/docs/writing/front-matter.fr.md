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
| `description` | Chaîne | Génère les balises `<meta name="description">` et `<meta property="og:description">` pour les moteurs de recherche et les aperçus de liens |
| `author` | Chaîne | Génère la balise `<meta name="author">` |
| `date` | Chaîne | Génère la balise `<meta property="article:published_time">` pour les moteurs de recherche et le partage social |

Les champs inconnus sont ignorés silencieusement — vous pouvez ajouter vos propres métadonnées personnalisées sans déclencher d'erreurs.

## Remplacement du titre

Par défaut, DocAnvil dérive les titres des pages à partir des noms de fichiers — `getting-started.md` devient "Getting Started". Le `title` du front matter remplace ce comportement partout :

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

Un simple titre suffit pour remplacer le titre dérivé du nom de fichier et définir la balise `<title>` de la page.

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

Les pages sans front matter fonctionnent exactement comme avant — le titre est dérivé du nom de fichier et aucune balise meta supplémentaire n'est ajoutée.

## Format de date

Le champ `date` est transmis tel quel à la balise meta `article:published_time`. Le format ISO 8601 (`YYYY-MM-DD`) est recommandé pour une meilleure compatibilité avec les moteurs de recherche et les plateformes sociales.
