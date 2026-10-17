# OpenAPI implementation

The first V1.3 implementation generates static API reference pages through normal build, serve, doctor and PDF workflows.

- [User guide and support boundaries](../../docs/docs/guides/openapi.md)
- [Loader, model and publishing design](foundation.md)
- [Documents API used by our own documentation](../../docs/docs/_examples/documents-api.json)
- [JSON/YAML and multi-file test fixtures](../../tests/fixtures/openapi)

Preview the example in the Master version of the documentation:

```sh
cargo run -- serve --path docs
```

Open `/master/en/example-api/operations/createdocument.html` to inspect the request/response presentation. The example also appears in the French documentation, with API descriptions kept in their original language.

Run focused checks with:

```sh
cargo test --test openapi_integration --test openapi_build
```
