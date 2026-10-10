---
{
  "description": "Toutes les commandes et options de docanvil, de new et serve jusqu'à export et update.",
  "title": "Commandes CLI",
  "slug": "cli-commands"
}
---
# Commandes CLI

DocAnvil propose huit sous-commandes : `new`, `theme`, `doctor`, `serve`, `build`, `component`, `export` et `update`.

## Options globales

| Option | Description |
|------|-------------|
| `--verbose` | Activer la sortie détaillée |
| `--quiet` | Supprimer les sorties non-erreur |

## `docanvil new`

Crée un nouveau projet de documentation.

```bash
docanvil new <nom>
```

| Argument | Requis | Description |
|----------|----------|-------------|
| `nom` | Oui | Nom du répertoire pour le nouveau projet |

Crée un répertoire de projet avec :

- `docanvil.toml` — configuration du projet
- `nav.toml` — structure de navigation
- `docs/` — répertoire de contenu avec des pages de démarrage
- `theme/custom.css` — feuille de style personnalisée vide

:::code-group
```bash
# Créer un projet de docs
docanvil new mes-docs
```

```bash
# Créer et immédiatement lancer le serveur
docanvil new mes-docs && cd mes-docs && docanvil serve
```
:::

## `docanvil theme`

Génère interactivement un thème de couleurs personnalisé pour votre projet.

```bash
docanvil theme [--overwrite] [--path <rép>]
```

| Option | Défaut | Description |
|--------|---------|-------------|
| `--overwrite` | `false` | Remplacer les personnalisations de thème existantes |
| `--path` | `.` | Chemin vers la racine du projet |

La commande guide à travers un prompt interactif :

1. **Mode de couleur** — choisir Clair uniquement, Sombre uniquement, ou Les deux (clair + sombre avec bascule)
2. **Couleur primaire** — code hex pour la couleur d'accent principale (demandé une fois par mode)
3. **Couleur secondaire** — code hex pour la couleur d'avertissement/secondaire (demandé une fois par mode)

Elle dérive ensuite toutes les variables CSS liées aux couleurs, les écrit dans `theme/custom.css`, et met à jour `docanvil.toml` avec `custom_css` et `color_mode`.

Si des personnalisations de thème existantes sont détectées (`custom_css` ou `[theme.variables]` dans la configuration), la commande se termine avec un message utile à moins que `--overwrite` ne soit passé.

:::code-group
```bash
# Générer un thème pour le projet courant
docanvil theme
```

```bash
# Générer un thème pour un projet dans un autre répertoire
docanvil theme --path ../mes-docs
```

```bash
# Remplacer un thème existant
docanvil theme --overwrite
```
:::

### Modes de couleur

| Mode | Comportement |
|------|----------|
| **Clair uniquement** | Palette claire unique dans `:root`. Pas de bouton de bascule. |
| **Sombre uniquement** | Palette sombre unique dans `:root` avec arrière-plans/texte sombres. Pas de bascule. |
| **Les deux** | Clair comme `:root` par défaut, sombre via `[data-theme="dark"]`, auto-détection `prefers-color-scheme`, et bouton soleil/lune dans l'en-tête. La préférence est sauvegardée dans localStorage. |

Quand "Les deux" est sélectionné, vous êtes invité à saisir des couleurs primaires et secondaires distinctes pour chaque mode, permettant des palettes claires et sombres indépendantes.

### Variables dérivées (clair)

À partir des deux couleurs saisies, les variables CSS suivantes sont générées pour le mode clair :

| Variable | Dérivation |
|----------|-----------|
| `--color-primary` | Couleur primaire telle quelle |
| `--color-primary-light` | Primaire éclaircie de 10% |
| `--color-link` | Identique à la primaire |
| `--color-link-hover` | Primaire assombrie de 10% |
| `--color-sidebar-hover` | Primaire teintée à 95% de luminosité |
| `--color-sidebar-active-bg` | Primaire teintée à 95% de luminosité |
| `--color-sidebar-active-text` | Primaire assombrie de 10% |
| `--color-note-bg` | Primaire teintée à 95% de luminosité |
| `--color-note-border` | Primaire éclaircie de 10% |
| `--color-mark-bg` | Primaire à 12% d'opacité |
| `--nav-group-toggle-hover` | Primaire à 6% d'opacité |
| `--color-focus-ring` | Primaire à 40% d'opacité |
| `--color-warning-border` | Couleur secondaire telle quelle |
| `--color-warning-bg` | Secondaire teintée à 95% de luminosité |

### Variables dérivées (sombre)

Le mode sombre utilise une logique de dérivation inversée plus des couleurs d'arrière-plan et de texte explicites :

| Variable | Dérivation |
|----------|-----------|
| `--color-bg` | Arrière-plan sombre (`#0f172a`) |
| `--color-bg-secondary` | Légèrement plus clair (`#1e293b`) |
| `--color-text` | Texte clair (`#f1f5f9`) |
| `--color-text-muted` | Texte mi-clair (`#94a3b8`) |
| `--color-border` | Bordure sombre (`#334155`) |
| `--color-code-bg` | Arrière-plan de code sombre (`#1e293b`) |
| `--color-primary` | Couleur primaire telle quelle |
| `--color-primary-light` | Primaire éclaircie de 10% |
| `--color-link` | Identique à la primaire |
| `--color-link-hover` | Primaire éclaircie de 10% (plus clair sur fond sombre) |
| `--color-sidebar-hover` | Primaire teintée à 15% de luminosité |
| `--color-sidebar-active-bg` | Primaire teintée à 15% de luminosité |
| `--color-sidebar-active-text` | Primaire éclaircie de 10% |
| `--color-note-bg` | Primaire teintée à 15% de luminosité |
| `--color-note-border` | Primaire éclaircie de 10% |
| `--color-mark-bg` | Primaire à 20% d'opacité |
| `--nav-group-toggle-hover` | Primaire à 12% d'opacité |
| `--color-focus-ring` | Primaire à 40% d'opacité |
| `--color-warning-border` | Couleur secondaire telle quelle |
| `--color-warning-bg` | Secondaire teintée à 15% de luminosité |

:::note
Après avoir généré un thème, lancez `docanvil serve` pour prévisualiser le résultat. Vous pouvez modifier directement le fichier `theme/custom.css` généré pour des ajustements supplémentaires.
:::

## `docanvil doctor`

Diagnostique les problèmes de configuration et de contenu du projet avant la compilation.

```bash
docanvil doctor [--fix] [--strict] [--format <fmt>] [--path <rép>]
```

| Option | Défaut | Description |
|--------|---------|-------------|
| `--fix` | `false` | Appliquer automatiquement les corrections sûres (créer les répertoires et fichiers manquants) |
| `--strict` | `false` | Quitter avec le code `3` si des avertissements ou erreurs sont trouvés (pour CI) |
| `--format` | `human` | Format de sortie : `human`, `checkstyle`, ou `junit` |
| `--path` | `.` | Chemin vers la racine du projet |

:::code-group
```bash
# Vérifier le projet courant
docanvil doctor
```

```bash
# Vérifier et corriger automatiquement les problèmes sûrs
docanvil doctor --fix
```

```bash
# Utiliser en CI pour échouer sur tout problème
docanvil doctor --strict
```

```bash
# Vérifier un projet dans un autre répertoire
docanvil doctor --path ../mes-docs
```
:::

Si aucun `docanvil.toml` n'est trouvé, doctor affiche un message amical suggérant `docanvil new` et se termine proprement.

### Sortie lisible par les machines

L'option `--format` bascule de la sortie colorée par défaut vers du XML structuré, utile pour l'intégration avec des outils d'annotation CI ou des rapporteurs de tests. La sortie lisible par les machines est écrite sur **stdout** ; les messages de progression sont entièrement supprimés.

| Format | Description |
|--------|-------------|
| `human` | Sortie colorée et lisible sur stderr (par défaut) |
| `checkstyle` | [XML Checkstyle](https://checkstyle.org/) — compatible avec reviewdog, les problem matchers GitHub Actions, et la plupart des outils CI Java/linting |
| `junit` | XML JUnit — compatible avec le résumé de tests GitHub Actions, GitLab CI, CircleCI, et la plupart des rapporteurs de résultats de tests |

:::code-group
```bash
# Sortie Checkstyle XML (ex. pipe vers reviewdog)
docanvil doctor --format checkstyle
```

```bash
# Sortie JUnit XML (ex. pour le résumé de tests GitHub Actions)
docanvil doctor --format junit
```

```bash
# Échouer en CI et émettre du Checkstyle XML
docanvil doctor --format checkstyle --strict
```

```bash
# Sauvegarder les résultats JUnit dans un fichier
docanvil doctor --format junit > test-results/doctor.xml
```
:::

**Checkstyle XML** regroupe les diagnostics par fichier. Chaque élément `<error>` porte :

- `severity` — `info`, `warning`, ou `error`
- `message` — le message de diagnostic
- `line` — numéro de ligne source (`0` quand non applicable)
- `source` — `docanvil.{catégorie}.{vérification}` (ex. `docanvil.readability.long-paragraph`)

**JUnit XML** mappe chaque catégorie de vérification à un `<testsuite>`. Les huit catégories sont toujours émises — les catégories sans problème produisent un seul `<testcase name="all-checks-passed"/>` réussi. Les avertissements et erreurs apparaissent comme éléments `<failure>` ; les diagnostics info apparaissent comme `<skipped/>`.

:::note
`--quiet` supprime le résumé lisible par les humains mais ne supprime pas la sortie XML — les formats machine écrivent toujours sur stdout indépendamment de `--quiet`.
:::

### Catégories de vérifications

Doctor exécute six catégories de vérifications, jusqu'à huit quand le versionnement et/ou l'i18n sont activés :

1. **Structure du projet** — fichier de configuration, répertoire de contenu, page index
2. **Configuration** — analyse TOML, références de fichiers (logo, favicon), validation nav.toml
3. **Thème** — existence du fichier CSS personnalisé, syntaxe Tera du template de mise en page
4. **Contenu** — wiki-links cassés, directives non fermées, erreurs JSON dans le front matter, slugs dupliqués
5. **Lisibilité** — vérifications de qualité du contenu sur tous les fichiers Markdown sources (voir ci-dessous)
6. **Versions** *(versionnement uniquement)* — configuration des versions et intégrité des répertoires
7. **Traductions** *(i18n uniquement)* — couverture des traductions dans les locales activées
8. **Sortie** — accessibilité en écriture du répertoire de sortie

### Vérifications de lisibilité

Ces vérifications s'exécutent sur les sources Markdown brutes et détectent les problèmes de qualité du contenu avant qu'ils n'atteignent les lecteurs. Chaque diagnostic inclut le chemin du fichier et le numéro de ligne.

#### Structure des titres

| Vérification | Sévérité | Ce qu'elle détecte |
|-------|----------|-----------------|
| `multiple-h1` | ⚠️ Avertissement | Plus d'un titre H1 sur la même page |
| `skipped-heading-level` | ⚠️ Avertissement | Saut de niveau de titre supérieur à un (ex. H1 → H3) |
| `consecutive-headings` | ⚠️ Avertissement | Deux titres avec seulement des lignes vides entre eux |
| `empty-heading` | ✗ Erreur | Un titre sans texte (`## `) |
| `heading-punctuation` | ℹ Info | Titre se terminant par `.` ou `!` — les points d'interrogation sont acceptables |
| `duplicate-heading-text` | ⚠️ Avertissement | Deux titres avec le même texte ; produit des collisions d'ID d'ancre |
| `emphasis-used-as-heading` | ⚠️ Avertissement | Une ligne entièrement en `**gras**` — utilisez `## Titre` à la place |
| `no-document-title` | ⚠️ Avertissement | La page n'a pas de H1 et pas de `"title"` dans le front matter |
| `heading-adjacent-separator` | ⚠️ Avertissement | Un titre est immédiatement adjacent à une règle horizontale — les titres créent déjà des ruptures visuelles, le séparateur est donc redondant |

#### Liens et images

| Vérification | Sévérité | Ce qu'elle détecte |
|-------|----------|-----------------|
| `missing-alt-text` | ⚠️ Avertissement | Image sans texte alternatif : `![](photo.jpg)` |
| `reversed-link-syntax` | ✗ Erreur | `(texte)[url]` au lieu de `[texte](url)` — le lien ne sera pas rendu |
| `empty-link` | ✗ Erreur | `[texte]()` (pas de destination) ou `[](url)` (pas de texte visible) |
| `non-descriptive-link-text` | ⚠️ Avertissement | Le texte du lien n'offre aucun contexte de navigation : "ici", "lire plus", "en savoir plus", etc. |
| `bare-url` | ⚠️ Avertissement | URL brute dans le texte — enveloppez-la avec `<url>` ou `[texte](url)` |

#### Blocs de code

| Vérification | Sévérité | Ce qu'elle détecte |
|-------|----------|-----------------|
| `missing-fenced-code-language` | ℹ Info | Bloc de code sans indicateur de langage — la coloration syntaxique ne s'appliquera pas |

#### Qualité du texte

| Vérification | Sévérité | Ce qu'elle détecte |
|-------|----------|-----------------|
| `long-paragraph` | ℹ Info | Paragraphe dépassant le seuil de nombre de mots (défaut : 150 mots) |
| `repeated-word` | ⚠️ Avertissement | Mots consécutifs en double : `le le`, `est est` |
| `todo-comment` | ⚠️ Avertissement | `TODO`, `FIXME`, `HACK`, `XXX`, ou `PLACEHOLDER` dans le texte |
| `placeholder-text` | ⚠️ Avertissement | `Lorem ipsum`, `TBD`, ou `[Insert … here]` dans le texte |

Toutes les vérifications ignorent le contenu à l'intérieur des blocs de code délimités. La plupart suppriment aussi les spans de code inline avant l'analyse pour éviter les faux positifs.

### Vérifications des versions

Ces vérifications s'exécutent quand `version.enabled` est non vide dans `docanvil.toml`.

| Vérification | Sévérité | Ce qu'elle détecte |
|-------|----------|-----------------|
| `current-not-in-enabled` | ✗ Erreur | `version.current` spécifie une version absente de la liste `enabled` |
| `version-dir-missing` | ✗ Erreur | Une version activée n'a pas de sous-répertoire correspondant dans le répertoire de contenu — `--fix` le crée |
| `empty-version` | ⚠️ Avertissement | Un répertoire de version existe mais ne contient aucun fichier `.md` |

### Correction automatique

L'option `--fix` applique des corrections sûres et non destructives :

| Problème | Correction appliquée |
|-------|-------------|
| Répertoire de contenu manquant | Crée le répertoire |
| Pas de `index.md` à la racine du contenu | Crée une page index minimale |
| Fichier CSS personnalisé introuvable | Crée un fichier CSS vide au chemin configuré |
| Répertoire de version manquant | Crée le sous-répertoire de version manquant dans le répertoire de contenu |

Les problèmes de lisibilité ne sont jamais corrigés automatiquement — ils nécessitent un jugement humain.

:::note
Relancez `docanvil doctor` après `--fix` pour vérifier que tous les problèmes sont résolus. Certaines corrections (comme la création du répertoire de contenu) peuvent révéler des problèmes supplémentaires à la prochaine exécution.
:::

## `docanvil serve`

Démarre un serveur de développement avec rechargement en direct.

```bash
docanvil serve [--host <adresse>] [--port <port>] [--path <rép>] [--editor <nom>]
```

| Option | Défaut | Description |
|--------|---------|-------------|
| `--host` | `127.0.0.1` | Adresse à laquelle lier le serveur |
| `--port` | `3000` | Numéro de port |
| `--path` | `.` | Chemin vers la racine du projet |
| `--editor` | `vscode` | Éditeur du lien « Open in editor » : `vscode`, `cursor`, `zed`, `idea`, `none`, ou un modèle d'URL contenant `{path}`. Également lu depuis `DOCANVIL_EDITOR` |

Le serveur :

- Compile le site au démarrage dans un répertoire temporaire — votre dossier `dist/` n'est jamais modifié, donc une compilation de développement ne peut pas être déployée par erreur
- Affiche aussi les [[writing/front-matter|pages brouillon]], avec un bandeau **Draft**
- Surveille votre répertoire de contenu (`content_dir`), `theme/`, `assets/`, `static/`, `docanvil.toml` et chaque `nav*.toml` (y compris les fichiers de navigation par langue et par version)
- Recompile les pages affectées lors d'un changement de fichier
- Notifie le navigateur via WebSocket à `/__docanvil_ws`
- Le navigateur recharge automatiquement — pas besoin de rafraîchissement manuel
- Les pages manquantes affichent le `404.html` de votre site, comme la plupart des hébergeurs statiques en production
- Affiche une ligne d'information quand une nouvelle version de DocAnvil est disponible (voir [`docanvil update`](#docanvil-update))

### Ouvrir dans votre éditeur

Pendant que `docanvil serve` tourne, chaque page affiche en bas un lien **Open in editor** qui ouvre sa source Markdown directement dans votre éditeur : vous repérez une coquille, vous la corrigez en un clic. Le lien est là même si vous n'avez pas configuré les [[guides/configuration|liens `[edit]`]], et si vous l'avez fait, il les remplace jusqu'à votre prochain `docanvil build`.

DocAnvil choisit votre éditeur dans cet ordre :

1. `--editor <nom>`
2. La variable d'environnement `DOCANVIL_EDITOR`
3. `$VISUAL` ou `$EDITOR`, quand elles désignent VS Code (`code`), Cursor, Zed ou IntelliJ IDEA (`idea`)
4. VS Code

| Nom | Ouvre |
|------|-------|
| `vscode` | VS Code |
| `cursor` | Cursor |
| `zed` | Zed |
| `idea` | IntelliJ IDEA et les autres IDE JetBrains qui gèrent les liens `idea://` |
| `none` | Désactive le lien (les pages affichent alors leur lien `[edit]`, s'il existe) |

Pour tout autre éditeur, donnez un modèle d'URL avec `{path}` à l'endroit où va le chemin absolu du fichier :

```bash
docanvil serve --editor "subl://open?url=file://{path}"
```

:::note{title="Réglez-le une fois"}
Le choix de l'éditeur est personnel : il se règle dans votre environnement plutôt que dans `docanvil.toml`. Ajoutez `export DOCANVIL_EDITOR=cursor` (ou votre éditeur) à votre profil shell et tous vos projets l'utiliseront.
:::

Pour masquer le lien sur une seule page, définissez `"edit_link": false` dans son [[writing/front-matter|front matter]].

### Exemples

:::code-group
```bash
# Par défaut : localhost:3000
docanvil serve
```

```bash
# Hôte et port personnalisés
docanvil serve --host 0.0.0.0 --port 8080
```

```bash
# Sortie détaillée pour voir les événements de recompilation
docanvil serve --verbose
```

```bash
# Servir un projet depuis un autre répertoire
docanvil serve --path ../mes-docs
```

```bash
# Ouvrir les pages dans Zed
docanvil serve --editor zed
```
:::

## `docanvil build`

Génère le site HTML statique pour le déploiement.

```bash
docanvil build [--out <chemin>] [--clean] [--strict] [--drafts] [--path <rép>]
```

| Option | Défaut | Description |
|--------|---------|-------------|
| `--out` | `[build] output_dir` (`dist`) | Répertoire de sortie pour le site généré. Remplace `output_dir` de `docanvil.toml` |
| `--clean` | `false` | Supprimer le répertoire de sortie avant la compilation. Refuse si le répertoire contient la racine du projet, le contenu, le thème ou les assets |
| `--strict` | `false` | Émettre les avertissements comme erreurs et quitter avec le code `3` |
| `--drafts` | `false` | Inclure les [[writing/front-matter|pages brouillon]] (pour les déploiements de prévisualisation). Elles gardent leur bandeau et sont marquées `noindex` |
| `--path` | `.` | Chemin vers la racine du projet |

Le pipeline de compilation traite chaque page en passant par :

1. Expansion des directives (composants)
2. Conversion des popovers
3. Rendu Markdown (comrak avec GFM)
4. Résolution des wiki-links
5. Injection des attributs inline
6. Encapsulation de template (mise en page Tera)

Les ressources statiques (CSS personnalisé, `assets/` et tout le contenu de `static/`) sont copiées dans le répertoire de sortie. Les fichiers restants des compilations précédentes — comme les pages renommées ou supprimées — sont supprimés ; les entrées cachées comme `.git` sont conservées.

:::code-group
```bash
# Compilation par défaut vers dist/
docanvil build
```

```bash
# Compilation propre vers un répertoire personnalisé
docanvil build --out public --clean
```

```bash
# Compiler un projet depuis un autre répertoire
docanvil build --path ../mes-docs
```

```bash
# Compilation de prévisualisation qui inclut les brouillons
docanvil build --drafts --out preview
```
:::

:::note
Les wiki-links cassés sont signalés comme avertissements pendant la compilation. Vérifiez la sortie pour les messages "broken link" pour trouver les références vers des pages inexistantes.
:::

## `docanvil component`

Consultez et personnalisez les [[writing/components|composants]] — les intégrés comme les vôtres.

### `docanvil component list`

Liste les composants intégrés et indique ceux que vous avez surchargés ou ajoutés.

```bash
docanvil component list [--path <rép>]
```

| Option | Défaut | Description |
|--------|---------|-------------|
| `--path` | `.` | Chemin vers la racine du projet |

```bash
# Voir ce qui est intégré, surchargé, et personnalisé
docanvil component list
```

### `docanvil component eject`

Copie le template d'un composant intégré dans `theme/components/` pour pouvoir le restyler.

```bash
docanvil component eject <nom>... [--all] [--force] [--path <rép>]
```

| Argument / Option | Défaut | Description |
|-------|---------|-------------|
| `<nom>...` | — | Un ou plusieurs composants intégrés à éjecter, ex. `note tabs` |
| `--all` | `false` | Éjecte tous les composants intégrés |
| `--force` | `false` | Remplace un template déjà présent à la destination |
| `--path` | `.` | Chemin vers la racine du projet |

```bash
# Éjecter le composant note pour le restyler
docanvil component eject note
```

## `docanvil export`

Exporte votre documentation dans d'autres formats.

### `docanvil export pdf`

Exporte les docs en un seul PDF en utilisant Chrome ou Chromium.

```bash
docanvil export pdf --out <chemin> [--path <rép>] [--locale <code>]
```

| Option | Requis | Défaut | Description |
|--------|----------|---------|-------------|
| `--out` | Oui | — | Chemin de sortie pour le fichier PDF |
| `--path` | Non | `.` | Chemin vers la racine du projet |
| `--locale` | Non | défaut du projet | Locale à exporter. Passez `all` pour générer un PDF par locale activée — ex. `guide.pdf` → `guide.en.pdf`, `guide.fr.pdf`. |

Nécessite Chrome ou Chromium. DocAnvil recherche dans les emplacements d'installation courants sur macOS, Windows et Linux avant de chercher dans le PATH.

:::code-group
```bash
# Exporter le projet courant
docanvil export pdf --out guide.pdf
```

```bash
# Exporter une seule locale
docanvil export pdf --out guide-fr.pdf --locale fr
```

```bash
# Exporter toutes les locales activées (projets i18n)
docanvil export pdf --out guide.pdf --locale all
```

```bash
# Exporter un projet dans un autre répertoire
docanvil export pdf --out guide.pdf --path ../mes-docs
```
:::

La sortie PDF est configurée via la section `[pdf]` dans `docanvil.toml`. Consultez [[guides/pdf-export|Export PDF]] pour le guide complet, y compris les pages de couverture, les formats de papier, le support RTL, et le CSS personnalisé.

## `docanvil update`

Vérifie sur GitHub si une nouvelle version de DocAnvil existe et met à jour sur place.

```bash
docanvil update [--check] [--yes] [--version <x.y.z>]
```

| Option | Défaut | Description |
|--------|---------|-------------|
| `--check` | `false` | Indique seulement si une nouvelle version est disponible, sans rien modifier |
| `--yes`, `-y` | `false` | Met à jour sans demander de confirmation (nécessaire sans terminal, par exemple dans un script) |
| `--version` | dernière | Installe une version précise, par exemple `1.2.0`. Permet aussi de revenir en arrière |

:::code-group
```bash
# Vérifier si une nouvelle version existe
docanvil update --check
```

```bash
# Mettre à jour, avec confirmation
docanvil update
```

```bash
# Mettre à jour sans confirmation
docanvil update --yes
```

```bash
# Revenir à une version précise
docanvil update --version 1.1.3
```
:::

Chaque téléchargement est vérifié avec la somme de contrôle SHA-256 publiée avec la version avant tout remplacement, et en cas de problème votre binaire actuel reste intact.

Si DocAnvil a été installé avec `cargo install`, `update` ne le remplace pas (cela perturberait cargo) et vous invite à lancer `cargo install docanvil --force`.

`docanvil serve` affiche une ligne d'information quand une nouvelle version est disponible. La vérification a lieu au plus une fois par jour, jamais en CI (quand `CI` est défini) ni avec `--quiet`, et vous pouvez la désactiver avec `DOCANVIL_NO_UPDATE_CHECK=1`.

## Codes de sortie

Toutes les commandes retournent des codes de sortie structurés pour que les pipelines CI puissent distinguer les différents types d'échec :

| Code | Signification | Causes examples |
|------|---------|----------------|
| `0` | Succès | Compilation terminée, doctor réussi |
| `1` | Échec général | Erreur IO, répertoire déjà existant, échec de configuration de l'exécution, erreur réseau ou de somme de contrôle dans `update` |
| `2` | Erreur de configuration | `docanvil.toml` manquant, syntaxe TOML invalide |
| `3` | Erreur de validation du contenu | Répertoire de contenu manquant, avertissements `--strict`, échecs `doctor --strict` |
| `4` | Erreur de rendu | Erreur de syntaxe de template, échec du rendu Markdown |
| `5` | Erreur interne | Panique (bug) — inclut un message vous demandant de soumettre un ticket |

Vous pouvez utiliser ces codes dans les scripts CI pour gérer différents types d'échec de manière appropriée :

```bash
docanvil build --strict
code=$?
case $code in
  0) echo "Compilation réussie" ;;
  2) echo "Corrigez votre docanvil.toml" ;;
  3) echo "Problèmes de contenu trouvés — vérifiez les avertissements ci-dessus" ;;
  *) echo "Compilation échouée avec le code $code" ;;
esac
```

:::note
Le code de sortie `5` ne devrait jamais apparaître en utilisation normale. Si vous le voyez, veuillez [signaler le problème](https://github.com/docanvil/docanvil/issues) — cela signifie que DocAnvil a rencontré une panique inattendue.
:::

## Pages associées

- [[guides/getting-started|Premiers pas]] — installer et créer votre premier projet
- [[guides/configuration|Configuration]] — référence `docanvil.toml` et `nav.toml`
- [[guides/pdf-export|Export PDF]] — pages de couverture, formats de papier, support RTL, et export par locale
