# Markdown

DocAnvil utilise comrak pour le rendu Markdown avec les extensions GitHub Flavored Markdown (GFM) activées. Tout ce que vous attendez du Markdown standard fonctionne, plus les tableaux, les listes de tâches, le texte barré, les notes de bas de page, et le front matter.

## Mise en forme du texte

Le **texte gras** s'entoure de doubles astérisques : `**gras**`

Le *texte italique* utilise des astérisques simples : `*italique*`

Le ~~texte barré~~ utilise des doubles tildes : `~~barré~~`

Vous pouvez les combiner : ***gras et italique***, ~~**gras barré**~~

Le ==texte mis en évidence== utilise des doubles signes égaux : `==mis en évidence==`

L'exposant utilise des carets : X^2^ s'écrit `X^2^`

L'indice utilise des tildes simples : H~2~O s'écrit `H~2~O`

Les codes emoji comme `:smile:` sont convertis en leurs équivalents Unicode : :smile:

## Titres

```markdown
# Titre 1
## Titre 2
### Titre 3
#### Titre 4
```

Le Titre 1 reçoit une bordure inférieure colorée. Le Titre 2 reçoit une ligne séparatrice subtile. Les Titres 3 et 4 sont des séparateurs sans style.

### IDs de titres personnalisés

Par défaut, les titres reçoivent des IDs générés automatiquement à partir de leur texte. Vous pouvez remplacer cela avec un ID personnalisé en utilisant `{#id}` à la fin de la ligne de titre :

```markdown
### Référence API {#api-ref}
```

Cela génère `<h3 id="api-ref">Référence API</h3>` au lieu de l'auto-généré `reference-api`.

## Liens et images

Liens Markdown standard : `[texte](url)`

Images : `![texte alternatif](url-image)`

Pour lier des pages de documentation, utilisez plutôt les [[writing/wiki-links|wiki-links]] — ils se résolvent automatiquement et avertissent des références cassées.

## Listes

Les listes non ordonnées et ordonnées prennent en charge l'imbrication et les cases à cocher de liste de tâches.

### Non ordonnée

- Premier élément
- Deuxième élément
  - Élément imbriqué
  - Autre élément imbriqué
- Troisième élément

### Ordonnée

1. Première étape
2. Deuxième étape
3. Troisième étape
   1. Sous-étape
   2. Autre sous-étape

### Listes de tâches

- [x] Écrire la documentation
- [x] Ajouter des exemples de code
- [ ] Relire et publier
- [ ] Célébrer

Les listes de tâches s'affichent comme des cases à cocher. Utilisez `- [x]` pour les éléments cochés et `- [ ]` pour les éléments non cochés.

## Tableaux

| Fonctionnalité | Syntaxe | Rendu |
|:--------|:------:|----------|
| Gras | `**texte**` | **texte** |
| Italique | `*texte*` | *texte* |
| Code | `` `code` `` | `code` |
| Barré | `~~texte~~` | ~~texte~~ |

Les tableaux prennent en charge l'alignement des colonnes avec des deux-points dans la ligne séparatrice :

```markdown
| Gauche | Centre | Droite |
|:-------|:------:|-------:|
| a      |   b    |      c |
```

## Blocs de code

Le code inline utilise des backticks simples : `let x = 42;`

Les blocs de code délimités utilisent des triples backticks avec un identifiant de langage optionnel :

```rust
fn main() {
    println!("Bonjour depuis DocAnvil !");
}
```

```javascript
const saluer = (nom) => {
  console.log(`Bonjour, ${nom} !`);
};
```

## Citations

> Les citations sont rendues avec une bordure gauche colorée et un arrière-plan subtil.
>
> Elles peuvent s'étendre sur plusieurs paragraphes.

Utilisez `>` au début de chaque ligne.

## Notes de bas de page

DocAnvil prend en charge les notes de bas de page[^1] avec la syntaxe standard. Référencez-les inline avec `[^nom]` et définissez-les n'importe où dans le document.

[^1]: Ceci est une note de bas de page. Elle apparaît en bas de la page dans une section dédiée.

Voici un autre exemple avec une note de bas de page plus longue[^détails].

[^détails]: Les notes de bas de page peuvent contenir plusieurs phrases. Elles sont collectées et rendues en bas de la page avec un séparateur horizontal et des rétro-références.

## Front Matter

Les pages peuvent inclure un front matter JSON entre des délimiteurs `---` en haut du fichier. Le front matter vous permet de définir des titres de pages personnalisés, des descriptions, des informations d'auteur, et des dates — que DocAnvil utilise pour les libellés de navigation, la recherche, et les balises meta SEO.

```markdown
---
{
  "title": "Titre de ma page",
  "description": "Un bref résumé pour les moteurs de recherche",
  "author": "Jane Doe",
  "date": "2024-01-15"
}
---

# Contenu de la page
```

Consultez [[writing/front-matter|Front Matter]] pour la liste complète des champs pris en charge et des exemples.

## Règles horizontales

Trois tirets ou plus, astérisques, ou underscores créent une règle horizontale :

```markdown
---
```

## Attributs inline

DocAnvil prend en charge un passage de post-traitement pour les attributs inline. Placez `{.classe #id}` sur la ligne immédiatement après un élément pour injecter des attributs HTML :

```markdown
## Ma section
{#id-personnalise .mis-en-evidence}
```

Cela génère `<h2 id="id-personnalise" class="mis-en-evidence">Ma section</h2>`.

Raccourcis pris en charge :

| Syntaxe | Résultat |
|--------|--------|
| `.nomclasse` | `class="nomclasse"` |
| `#nomid` | `id="nomid"` |
| `cle="valeur"` | `cle="valeur"` |

Plusieurs classes peuvent être combinées : `{.premiere .deuxieme #mon-id}`

## Pages associées

- [[writing/front-matter|Front Matter]] — métadonnées de pages, titres, et balises meta SEO
- [[writing/wiki-links|Wiki-links]] — liens à doubles crochets et popovers inline
- [[writing/components|Composants]] — notes, avertissements, onglets, et groupes de code

:::note{title="Propulsé par comrak"}
DocAnvil utilise comrak pour le rendu Markdown avec le mode `unsafe` activé, donc le HTML brut dans votre Markdown est transmis tel quel à la sortie.
:::
