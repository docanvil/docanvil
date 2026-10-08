---
{
  "title": "Feuille de route",
  "slug": "roadmap"
}
---
# Feuille de route

DocAnvil suit le [versionnage sémantique](https://semver.org/lang/fr/) et est stable en 1.x : les mises à jour au sein de 1.x ne casseront pas votre site. Voici où en est le projet et vers quoi il se dirige.

## 1.1.x — Améliorations des fonctionnalités principales

- ✅ Support de la localisation (docs multilingues, navigation et recherche par locale, sélecteur de langue)
- ✅ Export PDF (`docanvil export pdf` — basé sur Chrome, pages de couverture, RTL, par locale, taille de papier personnalisée)
- ✅ Support du versionnage de la documentation
- ✅ Linting de style (`docanvil doctor` — vérifications de lisibilité, structure des titres, qualité des liens, et plus)
- ✅ Scripts d'installation pour macOS, Linux et Windows, plus de plateformes précompilées, et `docanvil update`

## 1.2.x — L'essentiel de la rédaction

Les fonctionnalités du quotidien qu'on attend d'un outil de documentation, pour que les nouveaux projets aient tout ce qu'il leur faut dès le premier jour :

- Liens « Modifier cette page » vers la source d'une page sur GitHub, GitLab, Bitbucket ou tout autre hébergeur Git
- Ouverture du fichier source dans votre éditeur directement depuis `docanvil serve`
- Redirections, pour que les anciennes URL continuent de fonctionner quand des pages sont déplacées
- Inclusions, pour réutiliser du contenu partagé entre pages, locales et versions
- Blocs de code tirés de fichiers, éventuellement limités à certaines plages de lignes, et numéros de ligne pour tout bloc de code
- Pages brouillons, visibles dans `docanvil serve` mais exclues des compilations de production
- Dates de dernière mise à jour, issues de l'historique Git ou du front matter
- Génération de `llms.txt`, un index de votre documentation adapté aux IA
- Composants en templates : définissez vos propres composants sous forme de templates Tera dans votre thème, sans Rust

Entièrement rétrocompatible, et chaque fonctionnalité est optionnelle.

Plus tard dans la 1.2, une fois la première version des inclusions entre les mains des utilisateurs :

- Inclure une seule section d'une page (par exemple la section Installation de votre README) plutôt que le fichier entier
- Régions de code nommées (`#region` / `#endregion`), pour que les exemples ne cassent pas quand les numéros de ligne changent
- Mise en évidence de lignes et marqueurs de type diff dans les blocs de code
- Inclusions distantes depuis une URL

## 1.3.x — Extensibilité

Un système de plugins WASM (v1) pour les transformations Markdown et tout ce que les composants en templates ne permettent pas, options CLI supplémentaires, plus de diagnostics, et améliorations des templates. Entièrement rétrocompatible.

## 1.4.x et plus — Croissance de l'écosystème

Hooks pour plugins, optimisations des performances, compilations incrémentales, mise en cache, et un crate SDK pour les plugins. Les changements incompatibles nécessitent une version majeure.

## Idées à l'étude

Pas encore planifiées, mais dans notre viseur : un glossaire, la validation du schéma du front matter, la génération de références OpenAPI, des bundles de documentation hors ligne, des préréglages de compilation, des vérifications de contraste, le support des formules mathématiques (KaTeX), la vérification des liens externes et des cartes sociales.

## 2.0 et au-delà

Une version 2.0 ne se justifierait que s'il y a un besoin clairement établi — refonte de la configuration, API de plugins v2, changements fondamentaux du modèle de sortie, ou évolutions architecturales majeures. Ce doit être une décision délibérée et bien justifiée.
