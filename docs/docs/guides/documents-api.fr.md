---
{
  "title": "Parcours Documents API",
  "slug": "documents-api",
  "description": "Suivez un document fictif de sa création à sa consultation, avec les liens vers sa référence API.",
  "api_operations": [
    "example-api/operations/createdocument",
    "example-api/operations/getdocument",
    "example-api/operations/listdocuments"
  ]
}
---

# Parcours Documents API

Ce parcours fictif montre comment un guide et une référence API générée se complètent. Le serveur d’exemple n’accepte aucune requête réelle.

## Créer un document

Ouvrez [[example-api/operations/createdocument|Create a document]] et consultez le corps de la requête. Fournissez un titre, un contenu Markdown et, au besoin, un état de publication. Dans **Copy as cURL**, choisissez le serveur fictif de production ou de test, l’authentification bearer ou par clé API, et l’exemple JSON de brouillon ou de publication. **Show command** affiche le résultat ; **Copy cURL** le copie avec les identifiants à remplacer.

Une réponse réussie porte le statut `201`. Son en-tête `Location` désigne le nouveau document. Conservez son identifiant pour les requêtes suivantes ; le [[example-api/schemas/document|schéma Document]] décrit les champs renvoyés.

## Consulter le document

Utilisez [[example-api/operations/getdocument|Get a document]] avec l’identifiant comme paramètre de chemin. L’opération décrit la réponse réussie et le cas où le document est introuvable.

## Parcourir la bibliothèque

[[example-api/operations/listdocuments|List documents]] renvoie une page de résultats. Fournissez `limit` pour choisir la taille de la page, puis transmettez le `next_cursor` de la réponse comme `cursor` de la requête suivante.

Retrouvez ce parcours sous **Related guides** sur chacune de ces opérations. Le [[guides/openapi#guides-associes|guide OpenAPI]] explique comment ajouter ces liens à votre propre documentation.
