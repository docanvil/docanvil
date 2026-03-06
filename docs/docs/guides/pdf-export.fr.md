---
{
  "title": "Export PDF"
}
---
# Export PDF

DocAnvil peut exporter l'intégralité de votre site de documentation en un seul PDF prêt pour l'impression, en utilisant Chrome ou Chromium — sans outillage externe, sans Python, sans Pandoc.

Le PDF exporté comprend une table des matières, toutes vos pages dans l'ordre de navigation, des blocs de code avec coloration syntaxique, des diagrammes Mermaid, et une page de couverture optionnelle. Des en-têtes courants et des numéros de page sont ajoutés automatiquement.

## Prérequis

L'export PDF nécessite **Google Chrome** ou **Chromium** installé. DocAnvil lance le navigateur en mode headless et utilise le Chrome DevTools Protocol (CDP) pour rendre la page et l'imprimer en PDF.

DocAnvil recherche Chrome aux emplacements suivants (dans l'ordre) :

::::tabs
:::tab{title="macOS"}
- `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`
- `chromium` sur le PATH
:::
:::tab{title="Windows"}
- `%ProgramFiles%\Google\Chrome\Application\chrome.exe`
- `%ProgramFiles(x86)%\Google\Chrome\Application\chrome.exe`
:::
:::tab{title="Linux"}
- `google-chrome`, `google-chrome-stable`
- `chromium-browser`, `chromium`, `chrome`
- Tout ce qui précède sur le PATH
:::
::::

Si Chrome n'est pas trouvé, la commande se termine avec un message d'erreur clair.

## Utilisation de base

```bash
docanvil export pdf --out guide.pdf
```

| Option | Requis | Défaut | Description |
|--------|----------|---------|-------------|
| `--out` | Oui | — | Chemin de sortie pour le fichier PDF |
| `--path` | Non | `.` | Chemin vers la racine du projet |
| `--locale` | Non | défaut du projet | Locale à exporter. Passez `all` pour générer un PDF par locale activée. |

Les répertoires parents du chemin de sortie sont créés automatiquement.

:::code-group
```bash
# Exporter pour le projet courant
docanvil export pdf --out guide.pdf
```

```bash
# Exporter un projet dans un autre répertoire
docanvil export pdf --out guide.pdf --path ../mes-docs
```

```bash
# Supprimer la sortie de progression (utile dans les scripts)
docanvil export pdf --out guide.pdf --quiet
```
:::

## Configuration

L'export PDF est configuré dans la section `[pdf]` de `docanvil.toml` :

```toml
[pdf]
author = "Votre nom"
cover_page = true
paper_size = "A4"
custom_css = "theme/pdf.css"
```

| Clé | Défaut | Description |
|-----|---------|-------------|
| `author` | `None` | Nom de l'auteur affiché sur la page de couverture et dans l'en-tête courant |
| `cover_page` | `false` | Ajouter une page de titre avec le nom du projet et l'auteur |
| `paper_size` | `"A4"` | Format de papier — voir [Format de papier personnalisé](#format-de-papier-personnalise) pour les valeurs supportées |
| `custom_css` | `None` | Chemin (relatif à la racine du projet) vers un fichier CSS injecté dans le PDF |

## Page de couverture

Quand `cover_page = true`, une page de titre est ajoutée avant la table des matières. Elle affiche le nom du projet comme grand titre centré avec l'auteur en dessous.

```toml
[pdf]
cover_page = true
author = "L'équipe DocAnvil"
```

## Format de papier personnalisé {#format-de-papier-personnalise}

```toml
[pdf]
paper_size = "Letter"
```

Formats supportés (insensible à la casse) :

| Format | Dimensions |
|------|------------|
| `A3` | 297 × 420 mm (11,69 × 16,54 po) |
| `A4` | 210 × 297 mm (8,27 × 11,69 po) — **par défaut** |
| `A5` | 148 × 210 mm (5,83 × 8,27 po) |
| `Letter` | 8,5 × 11 po |
| `Legal` | 8,5 × 14 po |
| `Tabloid` | 11 × 17 po |

Les valeurs non reconnues reviennent silencieusement au format A4.

## Export par locale

Si votre projet utilise la [[guides/localisation|localisation]], vous pouvez générer un PDF distinct pour chaque locale activée en une seule commande :

```bash
# Un PDF par locale — le code de locale est inséré avant l'extension
docanvil export pdf --out guide.pdf --locale all
# → guide.en.pdf, guide.fr.pdf, guide.de.pdf …
```

Vous pouvez aussi exporter une seule locale spécifique :

```bash
docanvil export pdf --out guide-fr.pdf --locale fr
```

Chaque PDF utilise l'ordre de navigation et le contenu des pages de sa locale. Les pages sans traduction pour cette locale sont silencieusement ignorées.

:::note
`--locale all` nécessite que l'i18n soit configuré (`[locale]` avec `default` et `enabled` dans `docanvil.toml`). L'exécuter sur un projet sans i18n retourne une erreur claire.
:::

## Support des langues RTL

Les locales de droite à gauche sont détectées automatiquement. Quand vous exportez dans une locale RTL, Chrome met en page l'intégralité du PDF de droite à gauche — aucune configuration requise.

```bash
docanvil export pdf --out guide-ar.pdf --locale ar
```

Codes de locale RTL supportés : `ar` (arabe), `he` (hébreu), `ur` (ourdou), `fa` (persan/farsi), `ug` (ouïghour).

## CSS PDF personnalisé

Pour un contrôle précis de l'apparence du PDF, fournissez un fichier CSS :

```toml
[pdf]
custom_css = "theme/pdf.css"
```

Le fichier est injecté après les styles PDF par défaut, donc toute règle que vous écrivez remplace les valeurs par défaut. Quelques cibles utiles :

```css
/* Changer la police */
body {
  font-family: "Source Serif 4", serif;
  font-size: 10.5pt;
}

/* Blocs de code plus compacts */
pre {
  font-size: 8.5pt;
}

/* Supprimer la coloration des liens à l'impression */
a {
  color: inherit;
  text-decoration: none;
}

/* Marges plus larges */
@page {
  margin: 3cm;
}
```

## Diagrammes Mermaid

Les blocs `:::mermaid` sont rendus en SVG avant la capture du PDF. DocAnvil attend jusqu'à 15 secondes que tous les diagrammes soient terminés. Si certains sont encore en attente après le délai, le PDF est généré avec ce qui a été rendu.

Mermaid.js est chargé depuis un CDN, donc le rendu Mermaid nécessite une connexion internet. Dans les environnements CI hors ligne, désactivez les graphiques :

```toml
[charts]
enabled = false
```

## En-têtes et pieds de page courants

Chaque page reçoit :

- **En-tête gauche** — nom du projet
- **En-tête droit** — nom de l'auteur (si `author` est configuré)
- **Pied de page droit** — numéro de page

Ces éléments sont injectés par le moteur d'impression de Chrome et ne sont pas affectés par le CSS personnalisé.

## Conseils

- Lancez `docanvil build` d'abord pour confirmer que votre contenu est sans erreur. Les wiki-links cassés et les problèmes de rendu apparaîtront dans le PDF comme dans le site HTML.
- Le PDF suit exactement l'ordre de votre `nav.toml` — la table des matières et la séquence des chapitres correspondent à ce que les lecteurs voient en ligne.
- Utilisez `--quiet` pour supprimer la sortie de progression dans les scripts automatisés.

## Pages associées

- [[guides/localisation|Localisation]] — configurer des docs multilingues
- [[guides/configuration|Configuration]] — référence complète `docanvil.toml` incluant `[pdf]`
- [[reference/cli|Commandes CLI]] — toutes les sous-commandes et options
