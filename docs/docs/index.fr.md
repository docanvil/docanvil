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
:::feature{icon="⚡" title="Builds statiques rapides"}
Du HTML simple à déployer n'importe où, avec rechargement à chaud pendant l'écriture.
:::
:::feature{icon="🔍" title="Recherche plein texte"}
Un index de recherche côté client, généré pour chaque page sans configuration.
:::
:::feature{icon="🧩" title="Composants"}
Notes, onglets, groupes de code, diagrammes — ou [[writing/components|vos propres templates]].
:::
:::feature{icon="🎨" title="Thèmes personnalisés"}
Variables CSS, un [[guides/theming|générateur de thèmes]] et la surcharge complète des templates.
:::
:::feature{icon="🌍" title="Localisation"}
[[guides/localisation|Une documentation multilingue]] avec navigation et recherche par langue.
:::
:::feature{icon="🗂️" title="Versions"}
Gardez [[guides/versioning|la documentation de chaque version]] côte à côte, avec un sélecteur de version.
:::
::::

## Explorer la documentation

| Section | Ce que vous apprendrez |
|---------|------------------------|
| [[guides/getting-started\|Installation]] | Installer DocAnvil et créer votre premier projet |
| [[guides/configuration\|Configuration]] | Personnaliser `docanvil.toml` et `nav.toml` |
| [[guides/theming\|Thèmes]] | Variables CSS, styles personnalisés et surcharge des templates |
| [[guides/localisation\|Localisation]] | Documentation multilingue avec suffixes de langue et sélecteur de langue |
| [[writing/markdown\|Markdown]] | Toutes les fonctionnalités Markdown et GFM prises en charge |
| [[writing/wiki-links\|Liens et popovers]] | Syntaxe des wiki-links et popovers en ligne |
| [[writing/components\|Composants]] | Notes, onglets, groupes de code, pages d'accueil et vos propres templates |
| [[reference/cli\|Commandes CLI]] | Chaque commande, option et paramètre |
| [[reference/project-structure\|Structure du projet]] | Organisation des dossiers et découverte des pages |
| [[reference/css-variables\|Variables CSS]] | Référence complète des variables et de leurs valeurs par défaut |
