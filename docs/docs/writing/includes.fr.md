---
{
  "title": "Inclusions",
  "slug": "includes",
  "description": "Réutilisez du Markdown entre pages, langues et versions avec :::include"
}
---

# Inclusions

Écrivez un contenu une seule fois et utilisez-le sur autant de pages que nécessaire. Étapes d'installation, avertissement commun à plusieurs guides, tableau des options de la CLI : placez-le dans un fichier fragment et incluez-le là où il a sa place. Corrigez-le à un seul endroit et toutes les pages suivent.

## Fragments

Un fragment est un fichier Markdown dont le nom — ou celui d'un dossier de son chemin — commence par `_`. DocAnvil ne construit jamais les fragments comme des pages : ils restent hors de la barre latérale, de la recherche et du sitemap.

```text
docs/
  _shared/
    install.md        ← fragment
  guides/
    _flags.md         ← fragment
    setup.md          ← page
```

## Inclure un fragment

Placez `:::include` seul sur sa ligne. Les inclusions dans les citations (`> :::include{…}`) ne sont pas prises en charge : déplacez l’inclusion hors de la citation, ou placez la citation dans le fragment inclus.

Par exemple :

```markdown
:::include{file="_shared/install.md"}
```

Le Markdown du fragment est inséré à cet endroit avant tout le reste du rendu : ses titres apparaissent dans la table des matières, les wiki-links sont résolus, les composants s'affichent et la recherche indexe le texte — exactement comme si vous l'aviez écrit dans la page.

### Où pointent les chemins

:::include{file="../_shared/paths.md"}

Quelques règles de plus :

- Le **front matter** en tête d'un fragment est ignoré.
- **L'indentation est conservée** : indentez la ligne `:::include` dans un élément de liste et tout le fragment est indenté de la même façon.
- **Les fragments peuvent inclure des fragments.** Les chemins imbriqués sont relatifs au fragment qui les contient. Si des fichiers s'incluent les uns les autres en boucle, DocAnvil s'arrête et vous indique lesquels.
- **Tout fichier `.md` convient** : un fragment, une autre page, ou un README hors du dossier de contenu (`file="../README.md"`).
- `:::include` doit être seul sur sa ligne. Écrit au milieu d'une phrase, il s'affiche tel quel, et `docanvil doctor` le signale.

## Inclure une section

Incluez un titre et sa section depuis un fichier Markdown :

```markdown
:::include{file="../README.md" section="installation"}
```

`section` correspond à l'identifiant d'ancre du titre (un `#` initial est accepté). Le titre et ses sous-sections sont inclus jusqu'au prochain titre de niveau égal ou supérieur. Les titres soulignés sont également reconnus ; ceux dans les blocs de code, listes et citations ne délimitent pas de section.

Utilisez l'identifiant généré ou un identifiant stable comme `## Installation {#installation}`. Les titres répétés prennent les suffixes `-1`, `-2`, etc. Seules les inclusions de la section choisie sont développées, avec des chemins relatifs au fichier source et les numéros de ligne d'origine. La sélection de traduction reste identique ; utilisez le même identifiant personnalisé dans chaque langue. Une section absente ou vide produit une erreur d'inclusion.

## Traductions et versions

Quand la localisation est activée, DocAnvil cherche d'abord un fragment traduit. Une page française qui inclut `_shared/install.md` reçoit `_shared/install.fr.md` s'il existe, et le fichier non traduit sinon : traduisez les fragments qui en ont besoin et partagez les autres.

Les versions ne demandent rien de particulier : un fragment dans `docs/v2/` est trouvé relativement aux pages v2, et un chemin qui commence par `/` désigne le même fichier pour toutes les versions.

## Quand quelque chose ne va pas

Un fichier introuvable, une boucle ou un attribut inconnu s'affiche dans un encadré d'erreur à la place du contenu, avec un avertissement de compilation qui indique le fichier et la ligne. `docanvil build --strict` échoue dans ce cas, comme pour un lien cassé. `docanvil doctor` vérifie toutes les inclusions à l'avance, et liste aussi les fragments que rien n'inclut et les fragments traduits auxquels il manque une langue.

## Pages associées

- [[writing/components/code-blocks|Blocs de code]]
- [[reference/project-structure|Structure du projet]]
