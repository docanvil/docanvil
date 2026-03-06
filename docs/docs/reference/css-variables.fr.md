---
{
  "title": "Variables CSS",
  "slug": "css-variables"
}
---
# Variables CSS

Le thème par défaut de DocAnvil est entièrement construit sur des propriétés personnalisées CSS (variables). Surchargez n'importe laquelle d'entre elles pour personnaliser l'apparence de votre site sans écrire de CSS complexe.

## Comment surcharger

::::tabs
:::tab{title="docanvil.toml"}
Définissez les variables dans la section `[theme.variables]`. Omettez le préfixe `--` :

```toml
[theme.variables]
color-primary = "#059669"
font-body = "Georgia, serif"
content-max-width = "960px"
```
:::
:::tab{title="custom.css"}
Surchargez les variables dans un bloc `:root` dans votre fichier CSS personnalisé :

```css
:root {
  --color-primary: #059669;
  --font-body: Georgia, serif;
  --content-max-width: 960px;
}
```
:::
::::

## Couleurs

| Variable | Défaut | Description |
|----------|---------|-------------|
| `--color-primary` | `#6366f1` | Couleur d'accent principale (liens, états actifs, bordures) |
| `--color-primary-light` | `#818cf8` | Variante primaire plus claire (bordure h1, bordure blockquote) |
| `--color-bg` | `#ffffff` | Arrière-plan de la page |
| `--color-bg-secondary` | `#f8fafc` | Arrière-plan secondaire (barre latérale, en-têtes de tableaux, blockquotes) |
| `--color-text` | `#1e293b` | Couleur du texte principal |
| `--color-text-muted` | `#64748b` | Texte atténué (titres h4, séparateurs, pied de page) |
| `--color-border` | `#e2e8f0` | Couleur de bordure utilisée partout |
| `--color-link` | `#6366f1` | Couleur du texte des liens |
| `--color-link-hover` | `#4f46e5` | Couleur des liens au survol |
| `--color-code-bg` | `#f1f5f9` | Arrière-plan du code inline et des blocs de code |
| `--color-note-bg` | `#eef2ff` | Arrière-plan de l'admonition Note |
| `--color-note-border` | `#818cf8` | Bordure gauche de l'admonition Note |
| `--color-warning-bg` | `#fff7ed` | Arrière-plan de l'admonition Warning |
| `--color-warning-border` | `#f97316` | Bordure gauche de l'admonition Warning |

## Typographie

| Variable | Défaut | Description |
|----------|---------|-------------|
| `--font-body` | `system-ui, -apple-system, "Segoe UI", Roboto, sans-serif` | Police du corps de texte |
| `--font-mono` | `"SF Mono", Consolas, "Liberation Mono", Menlo, monospace` | Police à espacement fixe pour le code |
| `--font-size-base` | `16px` | Taille de police de base |
| `--font-size-sm` | `0.875rem` | Petite taille de police (en-têtes de tableaux, notes de bas de page) |
| `--line-height-tight` | `1.3` | Interligne serré pour les titres |
| `--heading-letter-spacing` | `-0.02em` | Espacement des lettres pour les titres |

## Mise en page

| Variable | Défaut | Description |
|----------|---------|-------------|
| `--sidebar-width` | `260px` | Largeur de la barre de navigation latérale |
| `--content-max-width` | `800px` | Largeur maximale de la zone de contenu |

## Barre latérale

| Variable | Défaut | Description |
|----------|---------|-------------|
| `--color-sidebar-hover` | `#eef2ff` | Arrière-plan au survol d'un lien dans la barre latérale |
| `--color-sidebar-active-bg` | `#eef2ff` | Arrière-plan du lien actif dans la barre latérale |
| `--color-sidebar-active-text` | `#4f46e5` | Couleur du texte du lien actif dans la barre latérale |

## Navigation

| Variable | Défaut | Description |
|----------|---------|-------------|
| `--nav-filter-bg` | `#ffffff` | Arrière-plan du champ de filtre |
| `--nav-filter-border` | `var(--color-border)` | Couleur de bordure du champ de filtre |
| `--nav-group-toggle-hover` | `rgba(99, 102, 241, 0.06)` | Arrière-plan au survol de la bascule de groupe |

## Ombres

| Variable | Défaut | Description |
|----------|---------|-------------|
| `--shadow-sm` | `0 1px 2px rgba(0, 0, 0, 0.05)` | Petite ombre (blocs de code, admonitions) |
| `--shadow-md` | `0 4px 6px -1px rgba(0, 0, 0, 0.07), 0 2px 4px -2px rgba(0, 0, 0, 0.05)` | Ombre moyenne (popovers, images) |

## Rayons de bordure

| Variable | Défaut | Description |
|----------|---------|-------------|
| `--radius-sm` | `4px` | Petit rayon (code inline, éléments nav, champ de filtre) |
| `--radius-md` | `6px` | Rayon moyen (blocs de code, tableaux, popovers) |
| `--radius-lg` | `8px` | Grand rayon (admonitions) |

## Transitions

| Variable | Défaut | Description |
|----------|---------|-------------|
| `--transition-fast` | `150ms ease` | Transitions rapides (états de survol) |
| `--transition-normal` | `200ms ease` | Transitions normales (rotation du chevron) |

## Focus

| Variable | Défaut | Description |
|----------|---------|-------------|
| `--color-focus-ring` | `rgba(99, 102, 241, 0.4)` | Couleur du contour de focus pour la navigation au clavier |

:::note
Toutes les valeurs de couleur utilisent la notation hex ou rgba. Les piles de polices utilisent la syntaxe CSS standard avec des alternatives. Les tailles acceptent n'importe quelle unité de longueur CSS (px, rem, em).
:::

## Pages associées

- [[guides/theming|Thèmes]] — comment appliquer les surcharges de variables et le CSS personnalisé
- [[guides/configuration|Configuration]] — référence `docanvil.toml`
