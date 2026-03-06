# Localisation

DocAnvil prend en charge les sites de documentation multilingues nativement. Chaque locale obtient son propre préfixe d'URL, sa navigation, son index de recherche, et un sélecteur de langue dans l'en-tête — le tout à partir d'un seul répertoire de contenu.

## Activer l'i18n

Ajoutez une section `[locale]` à votre `docanvil.toml` :

```toml
[locale]
default = "en"
enabled = ["en", "fr"]
auto_detect = true

[locale.display_names]
en = "English"
fr = "Français"
```

C'est tout. DocAnvil cherchera désormais des suffixes de locale dans vos noms de fichiers et construira un arbre de site distinct pour chaque langue.

## Convention de nommage des fichiers

Ajoutez le code de locale comme suffixe avant l'extension `.md` :

```text
docs/
  index.en.md          # Page d'accueil en anglais
  index.fr.md          # Page d'accueil en français
  guides/
    getting-started.en.md
    getting-started.fr.md
    advanced.en.md      # Pas encore de traduction française
```

Les fichiers **sans** suffixe de locale sont traités comme la locale par défaut. Donc si votre défaut est `"en"`, alors `index.md` et `index.en.md` sont équivalents — les deux deviennent la version anglaise.

:::note{title="Pas de faux positifs"}
Seuls les suffixes qui correspondent à votre liste `enabled` sont reconnus comme codes de locale. Un fichier comme `api.v2.md` ne sera pas traité à tort comme une locale — `v2` ne figure pas dans votre liste d'activation.
:::

## Structure de sortie

Chaque locale obtient son propre répertoire dans la sortie compilée :

```text
dist/
  js/docanvil.js        # Ressources partagées (une seule copie)
  robots.txt
  sitemap.xml           # Contient toutes les locales
  404.html              # Liens vers la page d'accueil de chaque locale
  en/
    index.html
    guides/
      getting-started.html
    search-index.json   # Index de recherche anglais
  fr/
    index.html
    guides/
      getting-started.html
    search-index.json   # Index de recherche français
```

Toutes les locales obtiennent des préfixes d'URL — même la locale par défaut. Cela maintient des URLs cohérentes et prévisibles.

## Navigation

DocAnvil construit un arbre de navigation distinct pour chaque locale, de sorte que la barre latérale de chaque langue n'affiche que les pages dans cette langue.

### Navigation auto-découverte

Lorsque vous n'avez pas de `nav.toml`, DocAnvil découvre automatiquement les pages pour chaque locale séparément. La navigation anglaise n'affiche que les pages anglaises, la navigation française n'affiche que les pages françaises.

### nav.toml par locale

Vous pouvez créer des fichiers de navigation spécifiques à chaque locale :

- `nav.fr.toml` — utilisé pour la compilation française
- `nav.en.toml` — utilisé pour la compilation anglaise
- `nav.toml` — repli pour toute locale sans son propre fichier

Les slugs dans les fichiers nav référencent les **slugs de base** sans le suffixe de locale. Une page à `docs/guides/setup.en.md` a le slug de base `guides/setup`, donc votre fichier nav utilise :

<pre><code class="language-toml">&#91;[nav]]
page = "guides/setup"
</code></pre>

Le même slug fonctionne dans les fichiers nav anglais et français — DocAnvil le résout vers la page de la locale correcte.

## Wiki-links

Les wiki-links se résolvent **dans la même locale**. Si vous écrivez `[[getting-started]]` dans une page française, cela pointe vers la version française de cette page. Vous n'avez pas besoin d'ajouter des suffixes de locale à vos liens.

Cela signifie que vos fichiers de contenu peuvent être traduits indépendamment sans mettre à jour les liens internes.

## Recherche

Chaque locale obtient son propre index de recherche (`en/search-index.json`, `fr/search-index.json`). L'interface de recherche charge automatiquement le bon index pour la locale actuelle, de sorte que les utilisateurs ne voient que des résultats dans leur langue.

## Sélecteur de langue

Lorsque l'i18n est activé, un sélecteur de langue apparaît dans la barre d'en-tête. Il affiche :

- Un emoji drapeau avec le code de locale actuel (ex. 🇫🇷 FR)
- Un menu déroulant listant toutes les locales activées avec emoji drapeau et noms d'affichage
- La locale actuelle mise en évidence
- Les traductions non disponibles grisées (lorsqu'une page n'existe pas dans cette locale)

Cliquer sur une locale navigue vers la même page dans la langue sélectionnée. Si la page n'existe pas dans la locale cible, le lien pointe vers la page d'accueil de cette locale.

### Emoji drapeaux

DocAnvil assigne automatiquement des emoji drapeaux selon les codes de locale — `en` obtient 🇬🇧, `fr` obtient 🇫🇷, `de` obtient 🇩🇪, etc. Les codes de locale inconnus obtiennent un globe 🌐.

Pour remplacer le drapeau par défaut d'une locale (ex. utiliser le drapeau américain pour l'anglais), ajoutez une table `[locale.flags]` :

```toml
[locale.flags]
en = "🇺🇸"
```

C'est utile lorsqu'une langue est associée à plusieurs pays et que vous voulez correspondre à votre audience.

## Détection automatique du navigateur

Lorsque `auto_detect` est `true` (valeur par défaut), DocAnvil vérifie la langue du navigateur du visiteur à sa première visite :

1. Si l'utilisateur a déjà choisi une langue (stockée dans `localStorage`), ce choix est respecté
2. Sinon, le `navigator.language` du navigateur est comparé aux locales activées
3. S'il correspond à une locale différente de la page actuelle, l'utilisateur est redirigé
4. La langue détectée est sauvegardée dans `localStorage`, évitant les redirections répétées

Définissez `auto_detect = false` pour désactiver entièrement ce comportement.

## Traductions manquantes

Lorsqu'une page existe dans certaines locales mais pas toutes, DocAnvil émet un avertissement à la compilation :

```text
warning: page 'guides/advanced' has no translation for locale 'fr'
  hint: Create a file with the '.fr.md' suffix to add a translation.
```

En mode `--strict`, ces avertissements deviennent des erreurs et la compilation échoue. C'est utile en CI pour s'assurer que les traductions sont complètes avant le déploiement.

La commande `docanvil doctor` vérifie également la couverture des traductions lorsque l'i18n est activé, en signalant :

- **missing-translation** (Avertissement) — pages manquantes dans certaines locales
- **orphaned-locale** (Avertissement) — fichiers avec des suffixes de locale absents de la liste d'activation
- **missing-default-locale** (Erreur) — la locale par défaut n'a aucune page du tout

## SEO

DocAnvil génère des signaux SEO multilingues complets lorsque l'i18n est activé :

### Balises hreflang

Chaque page inclut des balises `<link rel="alternate" hreflang="...">` pointant vers chaque traduction disponible. Celles-ci indiquent aux moteurs de recherche quelles pages sont des traductions les unes des autres, évitant les problèmes de contenu dupliqué et s'assurant que les utilisateurs voient les résultats dans leur langue.

Une balise `hreflang="x-default"` est également émise, pointant vers la version de la locale par défaut. Elle sert de repli pour les utilisateurs dont la langue n'est pas dans votre liste.

### URLs canoniques et Open Graph

Lorsque `site_url` est configuré, chaque page obtient :

- `<link rel="canonical">` — l'URL définitive pour la page
- `<meta property="og:url">` — l'URL utilisée quand la page est partagée sur les réseaux sociaux
- `<meta property="og:locale">` — la locale de la page actuelle
- `<meta property="og:locale:alternate">` — balises pour chaque traduction

Ces balises utilisent des URLs absolues dérivées de `site_url`. Sans `site_url`, les balises hreflang fonctionnent toujours avec des URLs relatives, mais les URL canoniques et og:url sont omises.

### Sitemap

Le sitemap inclut toutes les pages dans toutes les locales avec des annotations hreflang `xhtml:link` :

```xml
<url>
  <loc>https://exemple.com/en/guides/setup.html</loc>
  <xhtml:link rel="alternate" hreflang="en" href="https://exemple.com/en/guides/setup.html"/>
  <xhtml:link rel="alternate" hreflang="fr" href="https://exemple.com/fr/guides/setup.html"/>
  <xhtml:link rel="alternate" hreflang="x-default" href="https://exemple.com/en/guides/setup.html"/>
</url>
```

Google recommande à la fois les balises hreflang dans la page et dans le sitemap — DocAnvil fait les deux automatiquement.

### Attribut lang HTML

La balise `<html>` inclut un attribut `lang` correspondant à la locale actuelle, ce qui aide les moteurs de recherche et les lecteurs d'écran.

:::note{title="Conseil"}
Définissez `site_url` dans votre `docanvil.toml` pour tirer le meilleur parti du SEO multilingue. Sans cela, les URLs canoniques et les liens hreflang absolus ne peuvent pas être générés.
:::

## Compatibilité ascendante

Lorsqu'aucune section `[locale]` n'existe dans `docanvil.toml` :

- Les pages sont compilées dans le répertoire de sortie racine (sans préfixes de locale)
- Aucun sélecteur de langue n'apparaît
- Les wiki-links se résolvent globalement comme avant
- L'index de recherche se trouve à `/search-index.json`
- Doctor passe les vérifications de traduction

Les projets existants en langue unique fonctionnent sans aucune modification.

## Pages associées

- [[guides/configuration|Configuration]] — options de configuration `[locale]`
- [[writing/wiki-links|Liens & Popovers]] — comment les wiki-links se résolvent dans les locales
- [[reference/project-structure|Structure du projet]] — organisation des répertoires i18n
- [[reference/cli|Commandes CLI]] — mode `--strict` et `docanvil doctor`
