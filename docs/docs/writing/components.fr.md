# Composants

DocAnvil propose des composants intégrés rendus via des directives délimitées. Ils sont traités avant le rendu Markdown, donc vous pouvez utiliser du Markdown à l'intérieur.

## Syntaxe des directives

Les composants utilisent une syntaxe de bloc délimité avec des triples deux-points :

```markdown
:::nom{attributs}
Le contenu va ici. **Markdown** est pris en charge.
:::
```

La clôture ouvrante est `:::` suivi du nom du composant et des attributs optionnels entre accolades. La clôture fermante est `:::` seul sur sa ligne.

### Attributs

Les attributs sont spécifiés à l'intérieur de `{...}` après le nom du composant :

| Syntaxe | Résultat |
|--------|--------|
| `cle="valeur"` | Attribut nommé |
| `.nomclasse` | Ajoute une classe CSS |
| `#nomid` | Définit l'ID de l'élément |

Plusieurs attributs peuvent être combinés : `:::note{title="Important" .classe-perso #ma-note}`

## Note

Affiche des blocs d'information avec un thème bleu/indigo.

:::note
Ceci est une note avec le titre par défaut.
:::

:::note{title="Titre personnalisé"}
Les notes acceptent un attribut `title`. Le titre par défaut est "Note".
:::

Syntaxe brute :

```markdown
:::note{title="Titre personnalisé"}
Votre contenu ici. Prend en charge le **Markdown**.
:::
```

## Warning

Affiche des messages d'avertissement avec un thème orange.

:::warning
Ceci est un avertissement avec le titre par défaut.
:::

:::warning{title="Zone de danger"}
Les avertissements acceptent un attribut `title`. Le titre par défaut est "Warning".
:::

Syntaxe brute :

```markdown
:::warning{title="Zone de danger"}
Votre contenu d'avertissement ici.
:::
```

## Lozenge

Affiche un statut visuel rapide avec un lozenge.

La syntaxe est la suivante : `:::lozenge{type="default",text="Défaut"}`

| Syntaxe | Résultat |
|--------|--------|
| :::lozenge{type="default",text="Default"} | Default |
| :::lozenge{type="warning",text="Warning"} | Warning |
| :::lozenge{type="in-progress",text="In Progress"} | In Progress |
| :::lozenge{type="error",text="Error"} | Error |
| :::lozenge{type="success",text="Success"} | Success |

## Onglets

Regroupez du contenu en onglets commutables. Chaque onglet est défini avec une directive `:::tab` imbriquée. L'extérieur `::::tabs` utilise quatre deux-points pour que les clôtures internes `:::tab` (trois deux-points) ne ferment pas prématurément le conteneur :

::::tabs
:::tab{title="JavaScript"}
```javascript
console.log("Bonjour !");
```
:::
:::tab{title="Python"}
```python
print("Bonjour !")
```
:::
:::tab{title="Rust"}
```rust
fn main() {
    println!("Bonjour !");
}
```
:::
::::

Syntaxe brute :

```markdown
::::tabs
:::tab{title="JavaScript"}
Contenu pour l'onglet JavaScript.
:::
:::tab{title="Python"}
Contenu pour l'onglet Python.
:::
::::
```

Si aucun `title` n'est fourni, les onglets sont étiquetés "Tab 1", "Tab 2", etc.

## Groupe de code

Un composant d'onglets spécialisé pour comparer des blocs de code entre langages. Chaque bloc de code délimité devient un onglet, avec le nom du langage comme libellé d'onglet :

:::code-group
```rust
fn saluer(nom: &str) {
    println!("Bonjour, {nom} !");
}
```

```python
def saluer(nom):
    print(f"Bonjour, {nom} !")
```

```javascript
function saluer(nom) {
  console.log(`Bonjour, ${nom} !`);
}
```
:::

Syntaxe brute :

````markdown
:::code-group
```rust
fn saluer(nom: &str) {
    println!("Bonjour, {nom} !");
}
```

```python
def saluer(nom):
    print(f"Bonjour, {nom} !")
```
:::
````

## Diagrammes Mermaid

Rendez des diagrammes et graphiques avec Mermaid.js. Le contenu à l'intérieur d'un bloc `:::mermaid` est passé directement à Mermaid — il n'est pas traité comme Markdown.

:::mermaid
graph TD
    A[Écrire du Markdown] --> B[Lancer docanvil build]
    B --> C[Site HTML statique]
    C --> D[Déployer partout]
:::

Syntaxe brute :

````markdown
:::mermaid
graph TD
    A[Écrire du Markdown] --> B[Lancer docanvil build]
    B --> C[Site HTML statique]
    C --> D[Déployer partout]
:::
````

Mermaid prend en charge de nombreux types de diagrammes incluant les organigrammes, les diagrammes de séquence, les diagrammes de classes, les diagrammes d'état, les diagrammes de Gantt, et plus encore. Consultez la [documentation Mermaid](https://mermaid.js.org/) pour la référence complète de la syntaxe.

:::note{title="Configuration"}
Mermaid est activé par défaut. Désactivez-le en définissant `enabled = false` sous `[charts]` dans `docanvil.toml`. Quand il est désactivé, les blocs `:::mermaid` sont rendus comme du texte préformaté. Consultez [[guides/configuration|Configuration]] pour les détails.
:::

## Imbrication des directives

Lors de l'imbrication de directives, utilisez plus de deux-points sur la clôture extérieure pour la distinguer des clôtures intérieures. Le motif `::::tabs` (quatre deux-points) et `:::tab` (trois deux-points) en est l'exemple principal :

```markdown
::::tabs
:::tab{title="Premier"}
Contenu ici.
:::
:::tab{title="Deuxième"}
Contenu ici.
:::
::::
```

La directive extérieure utilise 4 deux-points (`::::`) tandis que les intérieures utilisent 3 (`:::`). La clôture fermante doit correspondre au nombre exact de deux-points utilisé dans la clôture ouvrante.

## Directives inconnues

Si vous utilisez un nom de directive qui ne correspond à aucun composant intégré, le contenu est enveloppé dans un `<div>` avec le nom de la directive comme classe :

```markdown
:::bloc-personnalise{.extra}
Cela devient un `<div class="bloc-personnalise extra">`.
:::
```

Cela vous permet de créer des blocs stylisés personnalisés avec votre propre CSS.

## Résumé

| Composant | Directive | Attribut clé | Défaut |
|-----------|-----------|---------------|---------|
| Note | `:::note` | `title` | `"Note"` |
| Warning | `:::warning` | `title` | `"Warning"` |
| Onglets | `::::tabs` + `:::tab` | `title` (sur l'onglet) | `"Tab 1"`, `"Tab 2"`, ... |
| Groupe de code | `:::code-group` | *(aucun)* | Nom du langage depuis la clôture de code |
| Mermaid | `:::mermaid` | *(aucun)* | Rend le diagramme via Mermaid.js |

:::note
Les composants sont traités avant le rendu Markdown. Cela signifie que vous pouvez utiliser le gras, l'italique, les liens, le code, et d'autres mises en forme Markdown à l'intérieur de n'importe quel composant.
:::

## Pages associées

- [[writing/markdown|Markdown]] — mise en forme du texte, tableaux, et autres fonctionnalités Markdown
- [[writing/wiki-links|Wiki-links]] — liens à doubles crochets et popovers inline
