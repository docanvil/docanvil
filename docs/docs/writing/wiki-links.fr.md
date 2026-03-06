---
{
  "title": "Liens & Popovers"
}
---

# Liens & Popovers

DocAnvil propose deux syntaxes inline spéciales au-delà du Markdown standard : les **wiki-links** pour lier des pages entre elles et les **popovers** pour des infobulles inline.

## Wiki-links

Les wiki-links sont la syntaxe de liens entre pages intégrée à DocAnvil — ils résolvent les slugs automatiquement et vous avertissent des références cassées lors de la compilation.

### Syntaxe de base

Créez un lien vers une autre page avec des doubles crochets :

<pre><code>Consultez &#91;[guides/getting-started]] pour les étapes d'installation.</code></pre>

Cela se résout en un lien HTML pointant vers la page cible. Le texte affiché est par défaut la cible du lien.

### Texte affiché personnalisé

Utilisez un pipe pour définir un texte de lien personnalisé :

<pre><code>Consultez le &#91;[guides/getting-started|guide d'installation]] pour commencer.</code></pre>

### Exemples en direct

Voici des wiki-links fonctionnels vers des pages de cette documentation :

- [[index]] — la page d'accueil
- [[guides/configuration|Guide de configuration]] — personnaliser votre projet
- [[reference/cli|Référence CLI]] — toutes les commandes et options
- [[writing/markdown|Fonctionnalités Markdown]] — syntaxe prise en charge

### Règles de résolution

DocAnvil résout les wiki-links en comparant à l'inventaire des pages en trois étapes :

1. **Correspondance exacte** — la cible est comparée directement aux slugs de pages (`guides/getting-started` correspond à `guides/getting-started`)
2. **Correspondance d'alias** — si le slug d'une page a été remplacé par le front matter (via `title` ou `slug`), l'ancien slug basé sur le nom de fichier pointe toujours vers la nouvelle page
3. **Correspondance de base** — si aucune correspondance exacte ou d'alias n'est trouvée, le dernier composant du chemin est essayé (`getting-started` correspond à `guides/getting-started`)

La correspondance de base vous permet d'utiliser des noms courts quand le nom de page est unique :

<pre><code>&#91;[getting-started]]     se résout vers → guides/getting-started
&#91;[configuration]]       se résout vers → guides/configuration</code></pre>

### Dérivation des slugs

Les slugs sont dérivés du chemin du fichier relatif au répertoire de contenu, avec l'extension `.md` supprimée. Quand une page a un `title` ou un `slug` dans son [[writing/front-matter|front matter]], le slug est remplacé en conséquence.

| Chemin du fichier | Front Matter | Slug |
|-----------|-------------|------|
| `docs/index.md` | — | `index` |
| `docs/guides/getting-started.md` | — | `guides/getting-started` |
| `docs/reference/cli.md` | — | `reference/cli` |
| `docs/01-setup.md` | `{"title": "Guide d'installation"}` | `guide-dinstallation` |
| `docs/faq-page.md` | `{"slug": "faq"}` | `faq` |

Quand un slug change, l'ancien et le nouveau slug se résolvent tous deux correctement dans les wiki-links.

### Résolution tenant compte de la locale

Quand la [[guides/localisation|localisation]] est activée, les wiki-links se résolvent **dans la même locale**. Un lien comme `[[getting-started]]` dans une page française pointe vers la version française de cette page — vous n'avez pas besoin de spécifier la locale dans vos liens. Cela signifie que votre contenu peut être traduit indépendamment sans modifier les wiki-links.

### Liens cassés

Quand la cible d'un wiki-link ne correspond à aucune page, il est rendu comme un span en rouge mis en évidence avec un popover d'erreur. Voici un exemple intentionnel :

<p>
    <span class="broken-link popover-trigger" tabindex="0">page-inexistante
        <span class="popover-content popover-error" role="tooltip">
        <strong>Page introuvable</strong><br>
        La page liée n'existe pas : <code>page-inexistante</code>
        </span>
    </span>
</p>

Le processus de compilation journalise également un avertissement pour chaque lien cassé, pour que vous puissiez les trouver et les corriger.

:::warning{title="Les liens cassés sont visibles"}
Les wiki-links cassés sont stylisés en rouge avec un soulignement en pointillés et une infobulle d'erreur. Ils sont faciles à repérer aussi bien dans le navigateur que dans la sortie de compilation.
:::

## Popovers

Les popovers ajoutent du contenu d'infobulle inline qui apparaît au survol ou au focus.

### Syntaxe

Utilisez `^[contenu]` pour créer un popover :

```markdown
DocAnvil utilise comrak^[Un parseur Markdown rapide et compatible GFM, écrit en Rust] pour le rendu.
```

### Exemples de popovers en direct

DocAnvil utilise comrak^[Un parseur Markdown rapide et compatible GFM, écrit en Rust] pour le rendu Markdown.

Le thème par défaut utilise une couleur d'accent indigo^[Plus précisément #6366f1, un bleu-violet équilibré].

Les popovers apparaissent au-dessus du texte déclencheur par défaut, mais se retournent en dessous^[Ce repositionnement automatique empêche les popovers d'être coupés en haut de la fenêtre] quand ils sont près du haut de la fenêtre.

### Comportement

- Les popovers apparaissent au **survol** et au **focus clavier**
- Ils se repositionnent automatiquement pour éviter de déborder des bords de la fenêtre
- Le contenu à l'intérieur des backticks (`` ` ``) et des blocs de code délimités n'est pas traité
- Le HTML dans le contenu des popovers est échappé pour la sécurité

:::note{title="Accessibilité"}
Chaque popover utilise `role="tooltip"` et `aria-describedby` pour connecter le déclencheur à son contenu, les rendant accessibles aux lecteurs d'écran. L'élément déclencheur a `tabindex="0"` pour la navigation au clavier.
:::

### Ignoré dans le code

La syntaxe de popover à l'intérieur du code inline (`^[comme ça]`) et des blocs de code délimités est laissée telle quelle :

```text
Cette syntaxe ^[popover] n'est pas traitée dans les blocs de code.
```

## Pages associées

- [[writing/markdown|Markdown]] — toutes les fonctionnalités Markdown et GFM prises en charge
- [[writing/components|Composants]] — blocs de directives pour du contenu plus riche
