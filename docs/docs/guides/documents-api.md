---
{
  "title": "Documents API walkthrough",
  "slug": "documents-api",
  "description": "Follow a fictional document from creation to retrieval, with links to its API reference.",
  "api_operations": [
    "example-api/operations/createdocument",
    "example-api/operations/getdocument",
    "example-api/operations/listdocuments"
  ]
}
---

# Documents API walkthrough

This fictional workflow demonstrates how a guide and generated API reference work together. The example server does not accept real requests.

## Create a document

Open [[example-api/operations/createdocument|Create a document]] and review its request body. Supply a title, Markdown body and optional publication status. The **Copy as cURL** menu provides a command with the supplied example and a token to replace.

A successful response has status `201`. Its `Location` header identifies the new document. Keep the document ID for subsequent requests; the [[example-api/schemas/document|Document schema]] describes the returned fields.

## Retrieve the document

Use [[example-api/operations/getdocument|Get a document]] with the document ID as its path parameter. The operation explains the successful response and what happens when a document cannot be found.

## Browse the library

[[example-api/operations/listdocuments|List documents]] returns a page of results. Supply a `limit` to control page size, then pass the response's `next_cursor` as the next request's `cursor` to continue browsing.

Return to any of these operation pages and find this walkthrough under **Related guides**. The [[guides/openapi#related-guides|OpenAPI guide]] explains how to add the same connections to your own documentation.
