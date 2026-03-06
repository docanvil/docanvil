---
{
  "title": "Feuille de route",
  "slug": "roadmap"
}
---
# Feuille de route

DocAnvil suit le [versionnage sémantique](https://semver.org/lang/fr/). La version actuelle est **1.1.1**. Voici où en est le projet et vers quoi il se dirige.

## 1.1.x — Améliorations des fonctionnalités principales

- ✅ Support de la localisation (docs multilingues, navigation et recherche par locale, sélecteur de langue)
- ✅ Export PDF (`docanvil export pdf` — basé sur Chrome, pages de couverture, RTL, par locale, taille de papier personnalisée)
- ✅ Support du versionnage de la documentation
- ✅ Linting de style (`docanvil doctor` — vérifications de lisibilité, structure des titres, qualité des liens, et plus)

## 1.2.x — Extensibilité

Système de plugins WASM (v1), options CLI supplémentaires, plus de diagnostics, et améliorations des templates. Entièrement rétrocompatible.

## 1.3.x et plus — Croissance de l'écosystème

Hooks pour plugins, optimisations des performances, compilations incrémentales, mise en cache, et un crate SDK pour les plugins. Les changements incompatibles nécessitent une version majeure.

## 2.0 et au-delà

Une version 2.0 ne se justifierait que s'il y a un besoin clairement établi — refonte de la configuration, API de plugins v2, changements fondamentaux du modèle de sortie, ou évolutions architecturales majeures. Ce doit être une décision délibérée et bien justifiée.
