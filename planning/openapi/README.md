# OpenAPI implementation

The first V1.3 implementation generates static API reference pages through normal build, serve, doctor and PDF workflows.

Merged in [PR #139](https://github.com/docanvil/docanvil/pull/139) on 17 October 2026. For current progress, remaining acceptance work and the next-session handoff, start with the [V1.3 plan](../v1.3-priorities.md).

- [User guide and support boundaries](../../docs/docs/guides/openapi.md)
- [Schema presentation matrix](../../docs/docs/guides/openapi.md#schema-support-matrix) and [regression inventory](foundation.md#schema-presentation-regression-coverage)
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
