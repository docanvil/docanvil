---
{
  "description": "Publiez la documentation de plusieurs versions côte à côte, avec un sélecteur de version et une navigation et une recherche propres à chaque version."
}
---

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

## Compiler depuis les tags Git

Conservez un seul arbre de documentation et compilez les versions depuis leurs tags Git :

```toml
[version]
current = "v1.2.3"
enabled = ["v1.2.3", "master"]

[version.git]
refs = { "v1.2.3" = "refs/tags/v1.2.3" }
working_tree = "master"

[version.display_names]
master = "Master (non publié)"
```

`refs` associe les codes de version des URL aux références Git locales. `working_tree` nomme la version compilée depuis votre copie de travail, modifications non commitées comprises. Ce nom est une étiquette : vous pouvez choisir `main` ou `development`. DocAnvil ne change pas de branche.

### Découvrir les tags de version

Pour publier toutes les versions correspondant à votre convention de tags, utilisez `tag_patterns` :

```toml
[version]
current = "v1.2.3"
enabled = ["master"]

[version.git]
tag_patterns = ["v1.*"]
working_tree = "master"
# include_prereleases = true
```

DocAnvil découvre les tags locaux correspondants et les ajoute à `enabled`, en utilisant le nom de chaque tag comme code de version dans les URL. `*` correspond à une suite quelconque de caractères et `?` à un seul caractère ; le motif doit correspondre au nom entier du tag. `v1.*` sélectionne les versions v1, tandis que `v1.2.*` sélectionne uniquement les versions v1.2.

Les tags de version sémantique, avec ou sans préfixe `v`, sont triés de la plus ancienne à la plus récente : `v1.9.0` précède `v1.10.0`. Les autres noms sont triés alphabétiquement après les versions sémantiques. Les versions découvertes précèdent celles activées explicitement, comme `master`. Les motifs qui se chevauchent et les tags activés explicitement ne créent pas de doublons ; une correspondance `refs` explicite est prioritaire pour un même code.

Les préversions comme `v1.3.0-beta.1` sont exclues de la découverte par défaut. Activez `include_prereleases = true` pour les inclure. Ce filtre concerne les tags de version sémantique ; une préversion associée explicitement reste disponible indépendamment de ce paramètre.

Les tags historiques peuvent contenir des traductions absentes ou des liens cassés signalés par les vérifications strictes actuelles. Utilisez un motif plus précis si ces instantanés font échouer la compilation stricte ; les tags conservent la documentation telle qu’elle a été publiée.

Chaque motif doit trouver au moins un tag local admissible, et `current` doit nommer une version activée ou découverte. Ce choix reste explicite : découvrir un nouveau tag ne change ni la version recommandée ni les liens `/latest/`. La compilation ne récupère pas les tags manquants. Chaque tag ajoute un arbre de documentation complet ; précisez le motif si le site devient trop volumineux.

Chaque tag fournit son répertoire de contenu, sa navigation, ses inclusions, ses composants personnalisés et ses ressources de contenu. La configuration de travail contrôle le thème commun, les langues, la publication et la version courante. Les configurations historiques n'ont pas besoin de nouveaux paramètres. Les images relatives sont publiées sous le préfixe de leur version ; les URL absolues restent inchangées.

Avec les sources Git, définissez explicitement `current`. La racine redirige vers cette version et `/latest/` fournit des liens stables vers ses pages. `/latest/guides/setup.html` redirige vers `/v1.2.3/guides/setup.html`, tandis que `/master/guides/setup.html` montre les changements non publiés. Avec la localisation, utilisez `/latest/fr/guides/setup.html`. `/latest/` ouvre l'accueil dans la langue par défaut. Modifier `current` déplace ces alias vers une autre version activée.

Master affiche une bannière de documentation non publiée. Les pages des tags utilisent leur commit pour les liens de modification et les dates. Les liens vers l'éditeur local sont réservés à la copie de travail. L'export PDF utilise aussi les sources et la navigation de la version choisie.

Les versions sans correspondance Git utilisent toujours des sous-répertoires : les deux approches peuvent coexister. Les codes doivent être des segments d'URL uniques composés de lettres, chiffres, points, tirets, traits de soulignement ou signes plus. Les noms des ressources générées sont réservés, ainsi que `latest` pour les sites Git.

Git doit être installé et les références doivent exister localement. La compilation ne récupère jamais de tags automatiquement. Avec GitHub Actions, utilisez `fetch-depth: 0` dans `actions/checkout` pour disposer des tags et de l'historique. Une référence absente fait échouer la compilation sans remplacer la sortie précédente, sauf si vous avez demandé `--clean`.

Les instantanés utilisent `git archive` : les attributs d'export s'appliquent, le contenu des sous-modules est omis et les liens symboliques ne sont pas pris en charge. Les préférences de fin de ligne propres à la machine n'affectent pas l'export ; les attributs de fin de ligne commités restent respectés. Les fichiers nécessaires doivent être présents au tag choisi.

Pour migrer un site publié, activez `[redirects] unprefixed = true` afin de conserver les URL simples et celles préfixées par une langue pour les pages de la version courante.

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
| `version-source-invalid` | Erreur | Une référence Git ou un projet historique ne peut pas être résolu ou analysé |

Lancez `docanvil doctor --fix` pour créer automatiquement les répertoires de version manquants.

## Compatibilité ascendante

Quand aucune section `[version]` n'existe dans `docanvil.toml` (ou que `enabled` est vide) :

- Les pages sont compilées dans le répertoire de sortie racine (sans préfixes de version)
- Aucun sélecteur de version n'apparaît
- Aucune bannière de version obsolète n'est rendue
- Doctor ignore les vérifications de version

Les projets existants fonctionnent sans aucune modification.

Activer le versionnement sur un site déjà en ligne déplace toutes les pages dans un dossier de version. Pour que les anciennes adresses continuent de fonctionner, définissez `unprefixed = true` dans la section `[redirects]` de votre [[guides/configuration|configuration]].

## Pages associées

- [[guides/configuration|Configuration]] — options de configuration `[version]`
- [[guides/localisation|Localisation]] — combiner versionnement et docs multilingues
- [[reference/project-structure|Structure du projet]] — organisation des répertoires versionnés
- [[reference/cli|Commandes CLI]] — vérifications doctor pour les versions
