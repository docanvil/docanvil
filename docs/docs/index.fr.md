---
{
  "description": "Transformez du Markdown en sites de documentation rapides et consultables, avec un seul binaire Rust.",
  "toc": false
}
---

::::hero{eyebrow="Documentation propulsée par Markdown" title="De belles docs. Vite construites."}
Transformez du Markdown en sites de documentation rapides, consultables et soignés. Un seul binaire, sans outillage compliqué.

:::buttons
[[guides/getting-started|Commencer →]] [Voir sur GitHub ↗](https://github.com/docanvil/docanvil)
:::
::::

## Démarrage rapide

```bash
curl -fsSL https://github.com/docanvil/docanvil/releases/latest/download/install.sh | sh
docanvil new my-docs
cd my-docs && docanvil serve
```

Ouvrez [http://localhost:3000](http://localhost:3000) et commencez à écrire — chaque enregistrement apparaît aussitôt dans votre navigateur. Sous Windows, ou vous préférez Cargo ? Consultez [[guides/getting-started|Premiers pas]].

## Ce que vous obtenez

::::features
:::feature{icon="lucide:zap" title="Builds statiques rapides"}
Du HTML simple à déployer n'importe où, avec rechargement à chaud pendant l'écriture.
:::
:::feature{icon="lucide:search" title="Recherche plein texte"}
Un index de recherche côté client, généré pour chaque page sans configuration.
:::
:::feature{icon="lucide:blocks" title="Composants"}
Notes, onglets, groupes de code, diagrammes — ou [[writing/components|vos propres templates]].
:::
:::feature{icon="lucide:palette" title="Thèmes personnalisés"}
Variables CSS, un [[guides/theming|générateur de thèmes]] et la surcharge complète des templates.
:::
:::feature{icon="lucide:globe" title="Localisation"}
[[guides/localisation|Une documentation multilingue]] avec navigation et recherche par langue.
:::
:::feature{icon="lucide:layers" title="Versions"}
Gardez [[guides/versioning|la documentation de chaque version]] côte à côte, avec un sélecteur de version.
:::
::::

## Explorer la documentation

::::cards
:::link-card{title="Premiers pas" href="guides/getting-started.html" icon="lucide:rocket"}
Installez DocAnvil et créez votre premier site.
:::
:::link-card{title="Configuration" href="guides/configuration.html" icon="lucide:settings"}
Configurez votre projet, la navigation et les options de build.
:::
:::link-card{title="Thèmes" href="guides/theming.html" icon="lucide:palette"}
Personnalisez les couleurs et les templates. Consultez la [[reference/css-variables|référence des variables CSS]].
:::
:::link-card{title="Localisation" href="guides/localisation.html" icon="lucide:globe"}
Publiez dans plusieurs langues avec navigation et recherche par locale.
:::
:::link-card{title="Écrire du contenu" href="writing/components.html" icon="lucide:blocks"}
Ajoutez des composants à votre [[writing/markdown|Markdown]] et reliez vos pages avec les [[writing/wiki-links|wiki-links et popovers]].
:::
:::link-card{title="Référence CLI" href="reference/cli-commands.html" icon="lucide:terminal"}
Retrouvez chaque commande et option, ainsi que la [[reference/project-structure|structure du projet]].
:::
::::
