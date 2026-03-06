# Versionnement

DocAnvil supporte les sites de documentation multi-versions par défaut. Chaque version obtient son propre préfixe d'URL, sa propre navigation, et son propre index de recherche — et un sélecteur de version dans l'en-tête permet aux lecteurs de naviguer entre les versions. Une bannière automatique les informe qu'ils consultent une version antérieure.

## Activer le versionnement

Ajoutez une section `[version]` à votre `docanvil.toml` :

```toml
[version]
current = "v2"
enabled = ["v1", "v2"]

[version.display_names]
v1 = "v1.0"
v2 = "v2.0 (latest)"
```

C'est tout ce qu'il faut configurer. DocAnvil cherchera désormais des sous-répertoires de version dans votre répertoire de contenu et compilera un arbre de site distinct pour chaque version.

## Structure des répertoires

Les versions se trouvent dans des sous-répertoires à l'intérieur de votre répertoire de contenu — pas comme des suffixes de fichiers. Cela permet de garder les anciennes versions autonomes et faciles à figer :

```text
docs/
  v1/
    index.md
    getting-started.md
    api/overview.md
  v2/
    index.md
    getting-started.md
    new-feature.md
    api/overview.md
```

Chaque répertoire de version est un arbre de contenu indépendant. Vous pouvez ajouter, supprimer, ou réorganiser des pages entre les versions sans affecter les autres.

:::note{title="Pourquoi des répertoires, pas des suffixes ?"}
Une approche par suffixe (`page.v1.md`) entraîne des centaines de fichiers mélangés dans un seul répertoire, complique le gel des anciennes versions, et se combine mal avec les suffixes de locale. L'approche par répertoires garde chaque version propre et autonome.
:::

## Structure de sortie

Chaque version obtient son propre répertoire dans la sortie de compilation :

```text
dist/
  index.html            # Redirection meta-refresh vers /v2/index.html
  js/docanvil.js        # Ressources partagées (une seule copie)
  robots.txt
  sitemap.xml           # Toutes les versions incluses
  404.html
  v1/
    index.html
    getting-started.html
    api/overview.html
    search-index.json   # Index de recherche v1
  v2/
    index.html
    getting-started.html
    new-feature.html
    api/overview.html
    search-index.json   # Index de recherche v2
```

Le `index.html` racine est une simple redirection meta-refresh — sans JavaScript requis, et compatible avec tout hébergeur de fichiers statiques.

## Navigation

DocAnvil construit un arbre de navigation distinct pour chaque version, donc la barre latérale de chaque version n'affiche que les pages de cette version.

### Navigation auto-découverte

Quand vous n'avez pas de `nav.toml`, DocAnvil auto-découvre les pages pour chaque version séparément. La nav v1 ne montre que les pages v1, la nav v2 ne montre que les pages v2.

### nav.toml par version

Vous pouvez créer des fichiers de navigation spécifiques à chaque version :

- `nav.v2.toml` — utilisé pour la compilation v2
- `nav.v1.toml` — utilisé pour la compilation v1
- `nav.toml` — repli pour toute version sans son propre fichier

Les slugs dans les fichiers nav référencent les **slugs de base** dans le répertoire de version — pas le chemin complet préfixé par la version. Une page à `docs/v2/guides/setup.md` a le slug de base `guides/setup`, donc votre fichier nav utilise :

<pre><code class="language-toml">&#91;[nav]]
page = "guides/setup"
</code></pre>

Le même slug fonctionne dans les fichiers nav de toutes les versions — DocAnvil le résout vers la page de la version correcte.

## Sélecteur de version

Quand plusieurs versions sont activées, un sélecteur de version apparaît dans la barre d'en-tête. Il affiche :

- Le code de version actuelle (ex. « v2 »)
- Un menu déroulant listant toutes les versions activées avec leurs noms d'affichage
- La version actuelle surlignée
- Quand la page actuelle n'existe pas dans une version cible, le lien renvoie vers la page d'accueil de cette version

Le sélecteur est masqué quand une seule version est configurée (même comportement que le sélecteur de locale avec une seule locale).

## Bannière de version obsolète

Quand un lecteur consulte une version non actuelle, une bannière apparaît en haut de la page :

> ⚠️ Vous consultez la documentation pour **v1**. [Passer à la dernière version (v2)](#)

Cela utilise le paramètre `version.current` pour déterminer ce que « dernière version » signifie. Si vous n'avez pas défini `current`, il prend par défaut le dernier élément de `enabled`.

La bannière renvoie directement à la même page dans la dernière version si elle existe, ou à la page d'accueil de la dernière version sinon.

## Wiki-Links

Les wiki-links se résolvent dans la version actuelle. `[[getting-started]]` écrit dans une page v1 renvoie à la version v1 de cette page. Vous n'avez pas besoin d'ajouter des préfixes de version à vos liens.

## Recherche

Chaque version obtient son propre index de recherche (`v1/search-index.json`, `v2/search-index.json`). L'interface de recherche charge automatiquement le bon index pour la version actuelle, donc les lecteurs ne voient que les résultats de leur version.

## Combinaison avec l'i18n

Le versionnement et la localisation se combinent naturellement. Activez les deux fonctionnalités dans votre configuration :

```toml
[version]
current = "v2"
enabled = ["v1", "v2"]

[locale]
default = "en"
enabled = ["en", "fr"]
```

Puis utilisez des suffixes de locale dans les répertoires de version :

```text
docs/
  v2/
    index.en.md
    index.fr.md
    guides/
      setup.en.md
      setup.fr.md
```

La sortie imbrique les locales dans les versions :

```text
dist/
  v2/
    en/
      index.html
      guides/setup.html
      search-index.json
    fr/
      index.html
      guides/setup.html
      search-index.json
```

Les fichiers nav par version peuvent aussi être spécifiques à une locale. DocAnvil les résout avec la priorité suivante :

1. `nav.{version}.{locale}.toml` — version + locale spécifique
2. `nav.{version}.toml` — version spécifique
3. `nav.{locale}.toml` — locale spécifique
4. `nav.toml` — repli global

## Vérifications Doctor

La commande `docanvil doctor` exécute des vérifications spécifiques aux versions quand `version.enabled` est non vide :

| Vérification | Sévérité | Ce qu'elle détecte |
|-------|----------|-----------------|
| `current-not-in-enabled` | Erreur | `version.current` spécifie une version absente de la liste `enabled` |
| `version-dir-missing` | Erreur | Une version activée n'a pas de sous-répertoire correspondant dans le répertoire de contenu |
| `empty-version` | Avertissement | Un répertoire de version existe mais ne contient aucun fichier `.md` |

Lancez `docanvil doctor --fix` pour créer automatiquement les répertoires de version manquants.

## Compatibilité ascendante

Quand aucune section `[version]` n'existe dans `docanvil.toml` (ou que `enabled` est vide) :

- Les pages sont compilées dans le répertoire de sortie racine (sans préfixes de version)
- Aucun sélecteur de version n'apparaît
- Aucune bannière de version obsolète n'est rendue
- Doctor ignore les vérifications de version

Les projets existants fonctionnent sans aucune modification.

## Pages associées

- [[guides/configuration|Configuration]] — options de configuration `[version]`
- [[guides/localisation|Localisation]] — combiner versionnement et docs multilingues
- [[reference/project-structure|Structure du projet]] — organisation des répertoires versionnés
- [[reference/cli|Commandes CLI]] — vérifications doctor pour les versions
