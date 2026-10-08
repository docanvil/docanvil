---
{
  "title": "Blocs de code depuis des fichiers",
  "slug": "code-blocks-from-files",
  "description": "Affichez de vrais fichiers sources, ou seulement les lignes utiles, avec numéros de ligne et légendes"
}
---

# Blocs de code depuis des fichiers

Les exemples de code copiés dans la documentation finissent par diverger du vrai code. Pointez plutôt un bloc de code vers le fichier source : il est relu à chaque compilation, et quand le code change, la documentation suit.

## Afficher un fichier

Ajoutez `file` à un bloc de code délimité vide :

````markdown
```toml file="/docanvil.toml"
```
````

Le bloc se remplit avec le contenu du fichier et reçoit une légende avec son chemin. Le langage vient de la clôture si vous en indiquez un, sinon de l'extension du fichier.

### Où pointent les chemins

:::include{file="../_shared/paths.md"}

## Afficher certaines lignes

Ajoutez `lines` pour n'afficher qu'une partie du fichier. Voici le `docanvil.toml` de ce site, réduit à deux sections :

````markdown
```toml file="/docanvil.toml" lines="1-3,7-9"
```
````

```toml file="/docanvil.toml" lines="1-3,7-9"
```

Les lignes sautées entre deux plages sont signalées par une rangée `⋯ N lines hidden`, et le bloc affiche les vrais numéros de ligne du fichier pour que le lecteur se repère dans la source.

| Valeur de `lines` | Affiche |
|---|---|
| `7` | la ligne 7 |
| `5-12` | les lignes 5 à 12 |
| `5-` | de la ligne 5 à la fin du fichier |
| `-12` | du début du fichier à la ligne 12 |
| `1-6,30-35` | plusieurs plages, dans l'ordre |

Les plages doivent être croissantes et ne pas se chevaucher. Une plage qui dépasse la fin du fichier est une erreur plutôt que d'être tronquée en silence : si le fichier source raccourcit, vous l'apprenez à la compilation au lieu de publier un exemple qui ne correspond plus.

L'indentation commune à toutes les lignes sélectionnées est retirée : une méthode tirée du milieu d'une classe commence au bord gauche.

## Numéros de ligne

Les numéros de ligne fonctionnent sur tous les blocs de code, pas seulement ceux qui viennent d'un fichier :

| Attribut | Effet |
|---|---|
| `numbers` | Numérote les lignes à partir de 1 (blocs de fichier : leurs vrais numéros) |
| `numbers="10"` | Commence à 10 |
| `numbers="false"` | Pas de numéros, même là où ils seraient affichés par défaut |

Les blocs de fichier avec `lines` sont numérotés par défaut. Pour numéroter tous les blocs de code du site, activez l'option dans `docanvil.toml` :

```toml
[syntax]
line_numbers = true
```

Les numéros sont dessinés par le thème au lieu d'être écrits dans le code : le bouton de copie ne copie que le code.

## Légendes

`title` ajoute une légende à n'importe quel bloc de code, ou remplace la légende par défaut d'un bloc de fichier. `title=""` la retire.

````markdown
```rust title="src/main.rs"
fn main() {}
```
````

## Dans les groupes de code et les onglets

Les blocs de fichier fonctionnent partout où un bloc de code fonctionne, y compris dans `:::code-group` et `:::tabs` :

````markdown
:::code-group
```rust file="/examples/hello.rs"
```
```python file="/examples/hello.py"
```
:::
````

## Quand quelque chose ne va pas

Un fichier introuvable, un fichier qui n'est pas en UTF-8, une plage `lines` qui dépasse la fin du fichier, ou un bloc de code qui a à la fois `file` et son propre code affiche un encadré d'erreur sur la page et un avertissement de compilation avec le fichier et la ligne. `docanvil build --strict` échoue dans tous ces cas, et `docanvil doctor` les vérifie tous à l'avance.

## Pages associées

- [[writing/includes|Inclusions]]
- [[writing/markdown|Markdown]]
- [[guides/configuration|Configuration]]
