---
{
  "title": "Guide de migration",
  "slug": "migration"
}
---
# Guide de migration

DocAnvil est en version pré-1.0, ce qui signifie que des changements incompatibles peuvent survenir entre les versions mineures. Cette page documentera les étapes de migration au fur et à mesure.

## Migration entre les versions

Rien à migrer pour l'instant ! DocAnvil n'a encore introduit aucun changement incompatible nécessitant une intervention manuelle.

Lorsque des changements incompatibles arriveront, vous trouverez ici des instructions de migration étape par étape — couvrant les changements de configuration, les options renommées, et tout ce qui pourrait affecter votre projet.

## Migrer depuis un autre outil

Si vous déplacez un site de documentation existant vers DocAnvil, le processus est simple :

1. Lancez `docanvil new mes-docs` pour créer un nouveau projet
2. Copiez vos fichiers Markdown dans le répertoire `docs/`
3. Configurez votre navigation dans `nav.toml` (ou laissez la découverte automatique s'en charger)
4. Adaptez la syntaxe spécifique à l'outil précédent (format du front matter, directives de composants, etc.)

DocAnvil utilise un front matter JSON (pas YAML), vous devrez peut-être convertir vos blocs de front matter. Consultez [[writing/front-matter|Front Matter]] pour le format.

:::note{title="Besoin d'aide pour migrer ?"}
Si vous rencontrez des problèmes en passant d'un autre outil de documentation, ouvrez une discussion sur [GitHub](https://github.com/docanvil/docanvil/discussions) — nous serions ravis d'entendre ce qui vous a posé problème pour rendre le processus plus simple.
:::
