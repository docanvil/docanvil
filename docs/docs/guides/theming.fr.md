---
{
  "title": "Thèmes",
  "slug": "theming"
}
---
# Thèmes

L'apparence de DocAnvil est personnalisable via trois couches, chacune s'appuyant sur la précédente :

1. **Surcharges de variables CSS** dans `docanvil.toml` — changements rapides de couleurs et de polices
2. **Fichier CSS personnalisé** — contrôle total sur n'importe quel élément
3. **Surcharges de templates** — remplacer entièrement la mise en page HTML avec des templates Tera

## Démarrage rapide : générateur de thème

La façon la plus rapide de personnaliser les couleurs de votre site est le générateur de thème interactif :

```bash
docanvil theme
```

Cela vous demande une couleur primaire et secondaire, puis génère un `theme/custom.css` complet avec toutes les variables de couleur dérivées et met à jour votre `docanvil.toml` automatiquement. Lancez `docanvil serve` ensuite pour prévisualiser le résultat.

Consultez [[reference/cli|Commandes CLI]] pour la liste complète des options (`--overwrite`, `--path`).

## Mode sombre

DocAnvil prend en charge le mode clair, sombre, ou les deux avec une bascule automatique. Définissez `color_mode` dans votre `docanvil.toml` :

```toml
[theme]
color_mode = "both"  # "light" (par défaut) | "dark" | "both"
```

|  <div style="width:60px">Mode</div>  | Comportement |
|------|----------|
| `light` | Palette claire uniquement (par défaut) |
| `dark` | Palette sombre uniquement — arrière-plans sombres, texte clair |
| `both` | Clair par défaut, avec une bascule soleil/lune dans l'en-tête et auto-détection via `prefers-color-scheme` du système |

### Utiliser le générateur de thème

La façon la plus simple de configurer le mode sombre est via le générateur de thème :

```bash
docanvil theme
```

Sélectionnez "Both (light + dark with toggle)" lorsqu'on vous demande le mode de couleur. Vous serez invité à saisir des couleurs primaires et secondaires distinctes pour chaque mode, et le générateur produira un unique `theme/custom.css` avec les variables claires dans `:root`, les variables sombres dans `[data-theme="dark"]`, et un bloc `@media (prefers-color-scheme: dark)` pour l'auto-détection système.

### Fonctionnement de la bascule

Quand `color_mode = "both"` :

- Un bouton icône soleil/lune apparaît dans l'en-tête
- À la première visite, la préférence du système est respectée via `prefers-color-scheme`
- Cliquer sur la bascule alterne entre clair et sombre et sauvegarde le choix dans `localStorage`
- Le choix persiste à travers les navigations de pages et les sessions du navigateur
- Un script de prévention du flash dans `<head>` garantit que la page s'affiche immédiatement dans le bon mode

### CSS mode sombre manuel

Si vous préférez écrire votre propre CSS pour le mode sombre plutôt que d'utiliser le générateur, structurez votre `theme/custom.css` ainsi :

```css
/* Mode clair */
:root {
  --color-primary: #6366f1;
  /* ... autres variables claires ... */
}

/* Mode sombre — bascule explicite */
[data-theme="dark"] {
  --color-bg: #0f172a;
  --color-text: #f1f5f9;
  /* ... autres variables sombres ... */
}

/* Mode sombre — préférence système */
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --color-bg: #0f172a;
    --color-text: #f1f5f9;
    /* ... mêmes variables sombres ... */
  }
}
```

Le sélecteur `[data-theme="dark"]` gère le choix explicite de l'utilisateur via la bascule, tandis que le bloc `@media` gère la préférence système lorsqu'aucun choix explicite n'a été fait.

## Variables CSS dans la configuration

La façon la plus simple de personnaliser le thème. Ajoutez des variables sous `[theme.variables]` dans `docanvil.toml` :

```toml
[theme.variables]
color-primary = "#059669"
color-primary-light = "#34d399"
color-link = "#059669"
color-link-hover = "#047857"
```

Ces valeurs sont injectées comme un bloc de style `:root` après le thème par défaut, remplaçant les valeurs intégrées. Les noms de variables omettent le préfixe `--` — DocAnvil l'ajoute automatiquement.

## Fichier CSS personnalisé

Pour plus de contrôle, pointez `custom_css` vers une feuille de style :

```toml
[theme]
custom_css = "theme/custom.css"
```

Ce fichier se charge après le thème par défaut et les surcharges de variables de configuration, donc il a la spécificité CSS la plus élevée. Utilisez-le pour :

- Des surcharges de variables supplémentaires dans un bloc `:root`
- Des sélecteurs personnalisés ciblant des éléments spécifiques
- De nouveaux styles pour vos propres classes (via les attributs inline)

## Personnalisations courantes

:::code-group
```toml
# docanvil.toml — changer la couleur d'accent et la police
[theme.variables]
color-primary = "#059669"
color-primary-light = "#34d399"
color-link = "#059669"
font-body = "Georgia, serif"
```

```css
/* theme/custom.css — zone de contenu plus large avec blocs de code sombres */
.content {
  max-width: 960px;
}

.content pre {
  background: #1e293b;
  color: #e2e8f0;
  border-color: #334155;
}

.content pre code {
  color: inherit;
}
```
:::

### Ordre de chargement

Les styles sont appliqués dans cet ordre (le dernier l'emporte) :

1. Thème par défaut (`style.css` intégré dans le binaire)
2. Variables de configuration (`[theme.variables]` → `:root { ... }`)
3. Fichier CSS personnalisé (chemin `custom_css`)

:::warning{title="Spécificité"}
Si une règle CSS personnalisée ne semble pas prendre effet, vérifiez que votre sélecteur est suffisamment spécifique pour remplacer le thème par défaut. `.content pre` est plus spécifique que simplement `pre`.
:::

## Surcharges de templates

Pour un contrôle complet sur la structure HTML, surchargez le template Tera par défaut. Créez un fichier à `theme/templates/layout.html` dans votre projet :

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>{{ page_title }} — {{ project_name }}</title>
  <style>{{ default_css | safe }}</style>
  {% if css_overrides %}
  <style>:root { {{ css_overrides }} }</style>
  {% endif %}
  {% if custom_css_path %}
  <link rel="stylesheet" href="/{{ custom_css_path }}">
  {% endif %}
  {% block head %}{% endblock %}
</head>
<body>
  {% block sidebar %}
  <nav class="sidebar">
    <div class="project-name">{{ project_name }}</div>
    {{ nav_html | safe }}
  </nav>
  {% endblock %}

  <main class="content">
    {% block content %}
    {{ content | safe }}
    {% endblock %}

    {% block footer %}
    <div class="footer">
      Built with <a href="https://github.com/docanvil/docanvil">DocAnvil</a>
    </div>
    {% endblock %}
  </main>

  {% block scripts %}{% endblock %}
</body>
</html>
```

### Blocs de template

| Bloc | Rôle |
|-------|---------|
| `head` | Contenu `<head>` supplémentaire (polices, balises meta, analytics) |
| `header` | Barre d'en-tête avec nom du projet et recherche |
| `sidebar` | La barre de navigation latérale |
| `content` | Zone de contenu principale |
| `footer` | Pied de page sous le contenu |
| `scripts` | JavaScript en fin de body |

### Variables de template

| Variable | Type | Description |
|----------|------|-------------|
| `page_title` | Chaîne | Titre de la page courante |
| `project_name` | Chaîne | Nom du projet depuis `docanvil.toml` |
| `default_css` | Chaîne | La feuille de style complète par défaut (utilisez le filtre `safe`) |
| `css_overrides` | Chaîne | Surcharges de variables CSS depuis la configuration |
| `custom_css_path` | Chaîne | Chemin vers le fichier CSS personnalisé, si configuré |
| `nav_html` | Chaîne | HTML de navigation rendu (utilisez le filtre `safe`) |
| `content` | Chaîne | HTML de la page rendu (utilisez le filtre `safe`) |
| `live_reload` | Booléen | Si le serveur de développement est en cours d'exécution |
| `search_enabled` | Booléen | Si la recherche plein texte est activée |
| `mermaid_enabled` | Booléen | Si le rendu des diagrammes Mermaid est activé |
| `mermaid_version` | Chaîne | Version majeure de Mermaid.js à charger depuis le CDN |
| `color_mode` | Chaîne | Mode de couleur : `"light"`, `"dark"`, ou `"both"` |

:::note
Le template par défaut inclut du JavaScript pour la commutation des onglets, le repli/développement de la barre latérale, le filtrage de navigation, le positionnement des popovers, la recherche, et le rendu des diagrammes Mermaid. Si vous surchargez le bloc `scripts`, vous devrez réimplémenter les fonctionnalités que vous souhaitez conserver.
:::

## Pages associées

- [[reference/css-variables|Variables CSS]] — liste complète de chaque variable et sa valeur par défaut
- [[guides/configuration|Configuration]] — référence `docanvil.toml` et `nav.toml`
