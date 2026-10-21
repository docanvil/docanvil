---
{"title": "Référence OpenAPI", "slug": "openapi", "description": "Publiez une spécification OpenAPI JSON ou YAML avec vos guides."}
---

# Référence OpenAPI

DocAnvil peut générer une référence statique à partir d'une spécification OpenAPI 3.0 ou 3.1 locale. Cette première intégration produit une vue d'ensemble, des pages d'opérations et des pages de schémas réutilisables.

## Explorer l’exemple

Consultez [[example-api/index|l’exemple Documents API]] dans la documentation Master. Il présente quatre opérations, la pagination, les options d’authentification bearer/clé API, des exemples de requêtes et réponses, des erreurs réutilisables et un schéma récursif. Le serveur est fictif : aucune requête réelle n’est exécutée.

Sa source est `docs/docs/_examples/documents-api.json` dans le dépôt DocAnvil. Notre configuration le génère sous `example-api` uniquement pour Master.

## Configuration

Avec `content_dir = "docs"`, placez la spécification dans `docs/openapi.json`, puis ajoutez cette entrée de premier niveau dans `docanvil.toml` :

```toml
[[openapi]]
spec = "openapi.json"
output = "api"
```

`spec` est relatif au répertoire de contenu sélectionné. Les extensions `.json`, `.yaml` et `.yml` sont acceptées. Pour un fichier situé à côté de `docanvil.toml`, utilisez `spec = "../openapi.json"`. Les références relatives sont résolues depuis le fichier qui les contient.

`output` définit un espace d'URL et vaut `api` par défaut. Utilisez des segments composés de lettres, chiffres, tirets ou traits de soulignement. Plusieurs entrées `[[openapi]]` avec des espaces distincts permettent de documenter plusieurs API.

```sh
docanvil build
docanvil serve
docanvil doctor
```

La vue d'ensemble se trouve dans `api/index.html`. Les pages restent en mémoire : aucun Markdown n'est écrit dans les sources. Après une compilation réussie, le serveur surveille aussi les fichiers référencés hors du répertoire de contenu.

## Télécharger la spécification

L’en-tête de la vue d’ensemble propose **Download OpenAPI**. Cette action fonctionne sans JavaScript et télécharge la spécification de la version et de la langue affichées. Chaque espace d’URL possède son fichier ; aucune configuration supplémentaire n’est nécessaire.

- Un document unique avec des références internes est publié sans modification dans `openapi.json`, `openapi.yaml` ou `openapi.yml`, avec sa mise en forme et ses commentaires.
- Une spécification avec des références à des fichiers locaux devient `openapi.zip`. Extrayez le ZIP, puis ouvrez son point d’entrée `openapi.json` dans votre outil OpenAPI. Les documents référencés sont normalisés en JSON sous `references/`, avec des références réécrites vers des chemins relatifs à l’archive. Les cycles et fragments JSON Pointer sont préservés. Les commentaires YAML et la mise en forme d’origine ne sont pas conservés dans les ZIP.

Une référence explicite au fichier lui-même produit aussi un ZIP, même si un seul fichier source est chargé. Le téléchargement ne dépend ainsi ni du nom ni de l’emplacement d’origine. Les noms d’entrées ne contiennent aucun chemin source absolu ; leur ordre et leurs métadonnées sont déterministes.

Le téléchargement contient les documents complets chargés par le graphe de références local, y compris les définitions, annotations et exemples non affichés. Les exemples externes, images Markdown et autres ressources ne sont pas téléchargés ou inclus. Les clés `$ref` littérales dans les exemples/extensions restent inchangées. Ce téléchargement ne valide ni les schémas ni les exemples.

Avec `output = "api"`, le fichier se trouve à côté de `api/index.html`, sous les mêmes préfixes de version et de langue. Les versions Git utilisent les documents de leur instantané. Les modifications des spécifications et références mettent à jour le fichier lors des reconstructions de serve ; les anciens formats disparaissent de la sortie si le type de téléchargement change. Ces fichiers ne deviennent pas des pages de navigation, de recherche ou du sitemap. La sortie IA fournit un lien fondé sur `site_url`, ou sur `base_url` en son absence. L’action est masquée à l’impression.

Build et doctor refusent les conflits avec les pages, fichiers statiques, ressources et feuilles CSS copiées. Une redirection ne peut pas occuper le chemin d’un téléchargement ni un chemin situé dessous. En cas d’échec, la compilation conserve la sortie publiée précédente, sauf avec `--clean`. Les thèmes peuvent utiliser `.api-download` et `.api-overview-toolbar`.

## Liens et navigation

```markdown
Consultez [[api/index|la référence API]].
```

Dans `nav.toml`, ajoutez les pages générées :

```toml
[[nav]]
label = "Référence API"
autodiscover = "api"
```

Pour cibler une section générée, ajoutez son identifiant au slug publié :

```markdown
[[api/operations/listpets#parameters|Paramètres]]
[[api/schemas/pet#properties|Champs de Pet]]
```

Les fragments sont sensibles à la casse ; le codage en pourcentage est accepté. Les titres répétés reçoivent un suffixe comme `responses-1` : utilisez l’identifiant de la page rendue. Les liens Markdown ordinaires vers les pages `.html` générées fonctionnent aussi, relativement à l’URL publiée du guide.

Build et doctor vérifient les pages et fragments d’API générés, y compris les liens dans les descriptions. Une cible manquante produit un avertissement et fait échouer `--strict`. Un échec du build strict conserve la sortie précédente, sauf avec `--clean`. Les sorties d’IA et le PDF conservent les liens de sections. Cette vérification ne couvre pas les fragments des pages rédigées, les URL externes ni les ressources arbitraires.

L'identifiant `listPets` produit `api/operations/listpets.html`. Les identifiants et noms de schémas sont normalisés ; les collisions provoquent une erreur. Sans identifiant, une opération utilise la méthode, le chemin et une courte empreinte déterministe. Changer l'identifiant change l'URL : prévoyez des redirections. Ces premières règles pourront évoluer avant la sortie de V1.3.

Les pages figurent dans la recherche, le sitemap et les sorties destinées aux outils d'IA lorsqu'elles sont activées. Le PDF inclut les pages sélectionnées par la navigation. Les liens d'édition ciblent la spécification ou le fichier référencé ; les dates Git tiennent compte des dépendances chargées.

### Guides associés

Ajoutez `api_operations` au front matter JSON d’un guide pour l’afficher sous l’en-tête des opérations qu’il explique :

```markdown
---
{
  "title": "Créer votre premier document",
  "api_operations": [
    "example-api/operations/createdocument",
    "example-api/operations/getdocument"
  ]
}
---

# Créer votre premier document
```

Utilisez les slugs complets publiés des opérations, avec leur espace d’URL, sans `.html`, fragment, crochets wiki ni préfixe de version ou de langue. Les opérations sans `operationId` utilisent leur slug généré à partir de la méthode et du chemin. Un guide peut viser plusieurs opérations ou API ; les doublons sont éliminés.

Le bloc **Related guides** utilise le titre et l’URL publiés du guide, y compris les remplacements de slug, et trie les liens par titre. Il apparaît uniquement lorsqu’un guide est associé. Les liens restent dans la même version et la même langue ; chaque traduction choisit ses opérations dans son propre front matter. Aucune autre langue n’est utilisée en repli. Les versions Git reprennent les associations et titres de leur instantané.

Les brouillons ne sont jamais listés, même en aperçu. Avec OpenAPI configuré, les valeurs mal formées et les cibles absentes ou autres que des opérations générées produisent des avertissements `openapi-related-guide` avec leur emplacement source, dans build et doctor. Le mode strict les refuse ; une compilation normale ignore les entrées invalides. Sans `[[openapi]]`, le champ est inactif.

Ces liens fonctionnent sans JavaScript et restent présents dans les exports PDF et IA. Les modifications des guides sont prises en compte lors des reconstructions de serve. La classe `.api-related-guides` permet de personnaliser le bloc. Consultez le [[guides/documents-api|parcours Documents API]], puis le lien associé depuis [[example-api/operations/createdocument|l’opération de création]].

### Badges de méthode dans la navigation

Les tableaux d’opérations de la vue d’ensemble utilisent les mêmes badges et couleurs de méthode que la navigation.

Les liens d’opérations affichent un petit badge de méthode HTTP dans la barre latérale par défaut. Cela fonctionne avec les groupes de tags, la découverte automatique, les pages explicites et les groupes liés, y compris les libellés personnalisés. Les liens des schémas et de la vue d’ensemble gardent leur présentation habituelle. La méthode est du texte réel, disponible sans JavaScript et pour les technologies d’assistance ; les libellés du fil d’Ariane et des exports restent identiques.

Si le libellé commence par sa méthode HTTP en majuscules suivie d’un espace, ce préfixe est omis dans la barre latérale : `GET /pets` devient un badge GET suivi de `/pets`. Le titre de la page reste complet.

Le `<li>` d’une opération porte `nav-openapi-operation` et une classe comme `nav-method-get` ou `nav-method-post`. Son lien porte `nav-operation-link` et `data-http-method="GET"` ; le badge et le libellé utilisent `nav-method-badge` et `nav-label`. La méthode provient de la spécification de la version sélectionnée, indépendamment du titre ou de l’URL.

Exemple à ajouter au CSS de votre thème :

```css
.sidebar .nav-method-post {
  --nav-method-bg: #d1fae5;
  --nav-method-color: #065f46;
}

.sidebar .nav-method-badge {
  border-radius: 0.25rem;
}
```

Pour masquer les badges tout en conservant les classes et attributs de méthode, utilisez `.sidebar .nav-method-badge { display: none; }`.

## Versions et langues

Pour des versions dans `docs/v1/` et `docs/v2/`, `spec = "openapi.json"` sélectionne le fichier de chaque version. Les versions Git lisent leurs propres spécifications et références, qui doivent rester dans l'instantané du dépôt. La configuration OpenAPI actuelle s'applique à toutes les versions activées : chacune doit contenir l'entrée configurée.

Pour ajouter une API sans exiger sa spécification dans les anciennes versions, ajoutez `versions = ["master"]` à son entrée `[[openapi]]`. Les noms doivent correspondre à des versions de documentation activées. Sans ce champ, l’API est générée pour toutes les versions activées.

La spécification est partagée entre les langues activées. Les pages utilisent les préfixes habituels de version et de langue, sans traduction des descriptions. Les spécifications propres à chaque langue ne sont pas encore prises en charge.

## Présentation des pages API

Les pages générées reprennent les couleurs et les polices du thème. L’en-tête regroupe méthode, chemin, description et tags. Des cartes compactes présentent l’authentification et les serveurs, et chaque réponse dispose de son panneau. Les pages de schémas et de présentation partagent les styles de tableaux et de sections. Le contenu reste lisible sans JavaScript et à l’impression.

Pour personnaliser l’en-tête des opérations :

```sh
docanvil component eject api-operation
```

Modifiez `theme/components/api-operation.html`. Le modèle reçoit `attrs.title`, `attrs.method`, `attrs.path`, `attrs.deprecated` (`"true"` ou `"false"`), `attrs.endpoint` (affichage du chemin sous le titre), le tableau `tags` et la description rendue dans `body`. Les valeurs simples sont échappées automatiquement ; conservez `{{ body | safe }}` pour la description. Cette personnalisation s’applique aux builds, à l’aperçu et au PDF. Les directives DocAnvil présentes dans une description de spécification restent du texte littéral.

Le composant peut aussi servir dans un guide rédigé :

````markdown
:::api-operation{title="Lister les documents" method="GET" path="/documents"}
Retourne les documents de votre espace de travail.
:::
````

Les pages de schémas utilisent le composant `api-schema` pour le nom, le type et la description, avec un panneau de propriétés et des résumés de contraintes. Pour personnaliser leur en-tête :

```sh
docanvil component eject api-schema
```

`theme/components/api-schema.html` reçoit `attrs.title`, `attrs.deprecated`, le type ou la référence rendus dans `type_html` et la description rendue dans `body`. Conservez `type_html | safe` et `body | safe` pour afficher ces fragments. Dans un guide rédigé, utilisez `:::api-schema{title="Document" type="object"}` avec la description dans le corps. Les classes propres aux schémas sont `.api-schema`, `.api-schema-kind`, `.api-schema-definition` et `.api-schema-constraints`.

Les en-têtes générés reçoivent aussi le menu dans `curl` : affichez-le avec `{{ curl | safe }}` pour le positionner. Les anciens modèles qui l’omettent reçoivent le menu après leur en-tête.

Les classes CSS disponibles comprennent `.api-curl`, `.api-reference`, `.api-operation`, `.api-context-card`, `.api-media` et `.api-response`. Les variables `--api-header-bg`, `--api-header-border` et `--api-accent` contrôlent les surfaces et l’accent de l’en-tête. Les autres pages de documentation conservent leur présentation.

## Prise en charge initiale

Les opérations affichent méthode, chemin, descriptions, authentification, serveurs, types et contraintes des paramètres, corps de requête, en-têtes de réponse et liens vers les schémas. Les exemples sont séparés des schémas. Des onglets permettent de choisir le type de contenu ; toutes les sections restent lisibles sans JavaScript et dans le PDF.

Les champs présentent les propriétés obligatoires, énumérations, valeurs par défaut, nullabilité, contraintes courantes et distinctions lecture/écriture. Les champs imbriqués et branches `oneOf`/`anyOf`/`allOf` peuvent être dépliés ; les références récursives renvoient aux pages de schémas. La profondeur d’affichage est limitée. **View source definition** conserve les définitions complètes pour les mots-clés sans présentation dédiée.

Les références JSON Pointer internes et entre fichiers locaux relatifs sont prises en charge, y compris les schémas récursifs. Les références distantes, ancres nommées, portées `$id`, ancres dynamiques et dialectes personnalisés produisent une erreur contextualisée. Les références des webhooks et callbacks sont vérifiées, mais leurs opérations ne sont pas rendues : un avertissement fait échouer les modes `--strict`.

### Matrice des schémas

Cette matrice décrit la documentation produite, sans validation des données de l’API. DocAnvil affiche les contraintes, sans les évaluer, vérifier les exemples ou fusionner les branches de composition.

| Fonctionnalité | OpenAPI 3.0 | OpenAPI 3.1 |
| --- | --- | --- |
| `type` et nullabilité | Un seul type ; `nullable: true` ajoute null si le même schéma déclare un type | Un type ou une liste de types uniques, dont `null` ; schémas booléens présentés comme « Any value » ou « No value allowed » |
| `properties`, `required`, `items`, `additionalProperties` | Tableaux de propriétés, listes des noms obligatoires, éléments de tableau et dictionnaires ouverts, fermés ou typés | Identique, avec sous-schémas booléens |
| `allOf`, `oneOf`, `anyOf` | Branches dépliables distinctes, sans fusion ni sélection par discriminateur | Identique |
| `enum`, `default`, `format`, `minimum`, `maximum`, `multipleOf`, `minLength`, `maxLength`, `pattern`, `minItems`, `maxItems`, `uniqueItems`, `minProperties`, `maxProperties` | Valeurs affichées comme contraintes ou annotations | Identique, avec affichage de `const` |
| `exclusiveMinimum`, `exclusiveMaximum` | Indicateurs booléens affichés avec `minimum`/`maximum` | Bornes exclusives numériques affichées directement |
| `description`, `readOnly`, `writeOnly`, `deprecated` | Descriptions et annotations ; les indicateurs lecture/écriture ne filtrent pas les champs des requêtes et réponses | Identique |
| `$ref` local | Liens vers les cibles ; champs voisins ignorés, y compris dans les tableaux de propriétés | Contraintes de la cible et des champs voisins conservées ; profondeur récursive limitée |
| `$defs`, `$schema` | Avertissement de dialecte ; source uniquement | Références locales parcourues dans `$defs` ; seuls les dialectes `$schema` acceptés sont autorisés, sans évaluateur de données |
| `not`, `discriminator`, `xml`, `externalDocs` | Source uniquement, avec avertissement | Identique |
| `prefixItems`, `contains`, `minContains`, `maxContains`, `patternProperties`, `propertyNames`, `dependentRequired`, `dependentSchemas`, `if`, `then`, `else`, `unevaluatedProperties`, `unevaluatedItems`, `contentEncoding`, `contentMediaType`, `contentSchema`, `$vocabulary` | Avertissement de dialecte ; source uniquement | Source uniquement, avec avertissement ; les références des sous-schémas reconnus sont chargées |
| Métadonnées et exemples | `title` et `example` conservés dans la source ; `example` d’un schéma directement attaché à un type de contenu est aussi affiché | Identique ; `examples` et `$comment` conservés dans la source |
| Extensions et autres mots-clés | Contenu `x-…` conservé sans interprétation ; les autres mots-clés inconnus produisent un avertissement de dialecte | Contenu `x-…` conservé sans interprétation ; les autres mots-clés inconnus ou anciens produisent un avertissement de présentation |

`openapi-schema-unrendered` signale les fonctionnalités sans présentation dédiée ; `openapi-schema-dialect` signale les mots-clés hors dialecte, notamment `nullable` en 3.1 et `const` en 3.0. Ces avertissements indiquent le fichier et le JSON Pointer et font échouer les modes stricts de build et doctor. Les métadonnées conservées en source ci-dessus et les extensions `x-…` ne produisent pas d’avertissement. Les portées URI, ancres et dialectes personnalisés non pris en charge restent des erreurs.

Les types incorrects et structures utilisées par le rendu produisent une erreur : `required` doit contenir des noms de propriétés uniques, les branches de composition doivent former des tableaux non vides, et les bornes exclusives doivent respecter le type de valeur du dialecte. Ces vérifications sont partielles : elles ne garantissent pas la conformité complète à OpenAPI ou JSON Schema et ne vérifient pas les relations entre contraintes.

Les descriptions sont rendues en Markdown. Le HTML brut est omis ; les inclusions et directives DocAnvil ne sont pas exécutées. Les requêtes interactives, la génération de SDK et la validation complète des schémas restent à venir.

## Groupes d’opérations

La vue d’ensemble regroupe les opérations par tag. Déclarez les `tags` à la racine de la spécification pour définir leur ordre et leurs descriptions Markdown. Les tags non déclarés suivent par ordre alphabétique, puis les opérations sans tag. Une opération portant plusieurs tags apparaît dans chaque groupe, une seule fois par groupe. Les tags déclarés sans opération restent visibles. Sans tag, la vue conserve un tableau unique. Les noms sont sensibles à la casse et les noms déclarés doivent être uniques. La navigation automatique et `autodiscover = "api"` (ou `"api/operations"`) regroupent les liens d’opérations par tag sous `Operations`. La navigation des schémas reste identique. Une opération portant plusieurs tags figure dans chaque groupe ; le fil d’Ariane, les liens précédent/suivant et les exports utilisent sa première occurrence. Les entrées explicites de pages et de groupes dans `nav.toml` conservent votre disposition. Les tags ne changent pas les URL des opérations.

## Exemples cURL générés

Le menu **Copy as cURL**, en haut à droite de l’en-tête, propose le serveur, l’option d’authentification, le type de contenu et l’exemple de corps. Choisissez les valeurs, puis **Copy cURL**. Les contrôles avec un seul choix restent masqués. **Show command** affiche la commande POSIX pour la vérifier ou la copier manuellement si le presse-papiers est indisponible.

Les choix utilisent les métadonnées effectives, y compris les serveurs et l’authentification hérités, ainsi que les exemples référencés localement. Les variables du serveur utilisent leurs valeurs par défaut sans champ de modification. **No authentication** est proposé uniquement lorsqu’une option vide est déclarée, ou lorsque l’opération ne demande aucune authentification. Les schémas d’une même option sont combinés ; les portées OAuth/OpenID sont indiquées. Changer d’authentification retire les anciens arguments, clés de requête et cookies.

Les identifiants restent des valeurs de remplacement. Aucune requête n’est exécutée, aucun jeton n’est obtenu ou stocké, et les choix ne sont pas conservés après rechargement. Les serveurs relatifs ou absents utilisent une origine fictive à remplacer. Les paramètres déjà présents dans l’URL du serveur sont préservés. Changer le type de contenu sélectionne un exemple valable pour ce type.

Les paramètres obligatoires sont inclus ; les paramètres facultatifs le sont lorsqu’un exemple ou une valeur par défaut est disponible. Les valeurs proviennent des exemples directs ou nommés (y compris les références locales), puis de l’exemple ou de la valeur par défaut du schéma résolu. Les valeurs manquantes utilisent `REPLACE_…`. Les arguments sont protégés pour le shell ; les valeurs d’URL sont encodées, même avec `allowReserved`.

La sérialisation couvre les valeurs simples et les tableaux/objets plats : `form`, `simple`, `label`, `matrix`, `deepObject` pour les objets avec `explode`, et les tableaux sans `explode` avec `spaceDelimited` ou `pipeDelimited`. Les valeurs imbriquées, les paramètres `content` et les autres styles demandent une adaptation manuelle indiquée sur la page.

Les corps proposent l’exemple direct et chaque exemple nommé du type de contenu, y compris les objets Example référencés localement : JSON, texte littéral, formulaires URL-encodés et champs texte multipart pour les objets plats sans `encoding` personnalisé. Sinon, `--data-binary '@request-body'` lit un fichier à préparer. Les fichiers multipart et encodages personnalisés nécessitent des parties préparées et une frontière correspondante. Les exemples externes ne sont pas téléchargés ; aucun corps n’est synthétisé à partir des schémas.

L’authentification Basic, bearer, OAuth 2, OpenID Connect et les clés API utilisent des valeurs à remplacer. Les schémas d’une même option sont combinés ; les autres mécanismes nécessitent une configuration manuelle. DocAnvil n’exécute aucune commande. Sans JavaScript, les commandes de chaque exemple utilisent le premier serveur et la première option d’authentification ; les autres serveurs et instructions d’authentification sont listés pour adaptation manuelle. Les exports PDF et IA reprennent cette présentation statique. La page conserve chaque choix une seule fois, sans produire toutes les combinaisons serveur/authentification/exemple.
