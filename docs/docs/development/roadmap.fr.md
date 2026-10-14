---
{
  "description": "Où en est DocAnvil aujourd'hui et ce qui arrive dans les prochaines versions.",
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

- ✅ Liens « Modifier cette page » vers la source d'une page sur GitHub, GitLab, Bitbucket ou tout autre hébergeur Git
- ✅ Ouverture du fichier source dans votre éditeur directement depuis `docanvil serve`
- ✅ Redirections, pour que les anciennes URL continuent de fonctionner quand des pages sont déplacées
- ✅ Inclusions, pour réutiliser du contenu partagé entre pages, locales et versions
- ✅ Blocs de code tirés de fichiers, éventuellement limités à certaines plages de lignes, et numéros de ligne pour tout bloc de code
- ✅ Pages brouillons, visibles dans `docanvil serve` mais exclues des compilations de production
- ✅ Dates de dernière mise à jour, issues de l'historique Git ou du front matter
- ✅ Génération de `llms.txt`, un index de votre documentation adapté aux IA
- ✅ Composants en templates : définissez vos propres composants sous forme de templates Tera dans votre thème, sans Rust
- ✅ Fil d'Ariane, descriptions du front matter en sous-titre des pages, et titres de page tirés du premier titre
- ✅ Mise en évidence de lignes avec `highlight="2,5-8"`, y compris depuis des fichiers locaux ou distants
- ✅ Blocs de diff avec coloration syntaxique, fonds de ligne rouges et verts et badge diff, depuis des fichiers `.diff` / `.patch` locaux ou distants
- ✅ Personnalisation des fonds, en-têtes, polices et badges des blocs de code dans les thèmes
- ✅ Blocs de code distants depuis une URL HTTPS brute (`url="…"`)
- ✅ Régions de code nommées (`{== region="name" ==}` / `{== endregion="name" ==}`), pour que les exemples ne cassent pas quand les numéros de ligne changent
- ✅ Composants de mise en page, icônes intégrées et groupes de navigation dépliables
- ✅ Versions issues des tags Git, version de travail et alias stables `/latest/`
- ✅ Sélection de version et de langue pour les PDF, avec navigation et styles adaptés
- ✅ Inclusion d’une section Markdown avec `section="heading-id"`

Entièrement rétrocompatible, et chaque fonctionnalité est optionnelle.

## 1.3.x — Documentation API

La génération de références OpenAPI est la prochaine priorité. V1.2 a enrichi les guides et les exemples issus du code ; produire une référence à partir d’une spécification comble un nouveau besoin documentaire.

- Référence statique à partir de spécifications JSON ou YAML locales, visant OpenAPI 3.0 et 3.1 avec une matrice de compatibilité documentée
- Opérations, paramètres, corps de requête, réponses, authentification, exemples et schémas liés
- URL stables des opérations, navigation par tags, recherche et liens depuis les guides Markdown
- Validation des liens vers la référence et des ancres des opérations et schémas générés
- Intégration aux thèmes, versions de documentation, repli linguistique, PDF et sorties adaptées aux IA
- Diagnostics dans build et `docanvil doctor`, avec vérifications en mode strict

Commencer par les spécifications et références locales. Les requêtes interactives, les spécifications distantes et les autres formats API viendront éventuellement ensuite. La configuration et les limites détaillées seront définies par un prototype. Les fonctionnalités restent optionnelles et rétrocompatibles.

## 1.4.x et plus — Fiabilité et qualité documentaire

Sources de code distantes reproductibles avec actualisation explicite, cache persistant et compilation hors ligne ; validation plus complète des liens internes et des ancres ; schémas optionnels pour le front matter ; améliorations mesurées des compilations des grands sites versionnés. La référence API évoluera selon les retours d’usage. Cet ordre est une orientation, pas une promesse de livraison.

## Extensibilité — Reportée

Les composants en templates et les personnalisations couvrent déjà de nombreux besoins de présentation. Le système de plugins WASM, ses hooks et son SDK sont reportés jusqu’à ce que des cas concrets montrent ce que les templates et les générateurs intégrés ne peuvent pas couvrir. Aucune version précise n’est assignée.

## Idées à l’étude

Ces idées ne sont pas planifiées et n’élargissent pas la première version OpenAPI :

- Rapports de changements API entre versions, avec règles explicites de compatibilité
- Vérification de la couverture documentaire API : descriptions, exemples et réponses d’erreur manquants
- Détection des traductions à revoir après modification des pages sources ou des fragments partagés
- Bundles HTML autonomes avec recherche hors ligne, en complément des PDF
- Compilation ou tests optionnels des exemples de code en CI, avec paramètres d’exécution explicites
- Responsables, dates de révision, avis de dépréciation et vérifications des contenus anciens
- Audit des migrations d’URL pour signaler les routes supprimées et suggérer des redirections
- Alias, pondération et filtres de recherche pour guides, opérations et schémas
- Rapports de provenance avec révisions sources et sommes de contrôle des sources distantes
- Rendu de Mermaid à la compilation pour des diagrammes portables et des exports cohérents
- Variables de contenu réutilisables par version et langue, avec diagnostics des valeurs manquantes
- Exceptions ciblées aux règles de lint, avec justification et détection des exceptions inutilisées
- Vérification des pages absentes de la navigation et liens connexes ou entrants optionnels
- Exports PDF ou hors ligne limités à des chapitres ou groupes de navigation
- Résumés des changements documentaires entre versions, liés aux guides de migration
- Glossaire, inclusions Markdown distantes, préréglages de compilation, contraste, formules mathématiques (KaTeX), liens externes et cartes sociales

Les blocs de code distants sont déjà disponibles ; les inclusions Markdown distantes sont une idée distincte.

## 2.0 et au-delà

Une version 2.0 ne se justifierait que s'il y a un besoin clairement établi — refonte de la configuration, API de plugins v2, changements fondamentaux du modèle de sortie, ou évolutions architecturales majeures. Ce doit être une décision délibérée et bien justifiée.
