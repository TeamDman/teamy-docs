# Version JSON documents with Facet

Use [`facet-json-migration`](https://github.com/TeamDman/facet-json-migration/tree/dc5ab258957aa390cd74028afeb5d4153c89c028) as a standalone example for reading older JSON payloads after the schema changes. Start with the [migration tests](https://github.com/TeamDman/facet-json-migration/blob/dc5ab258957aa390cd74028afeb5d4153c89c028/tests/migration_demo.rs) and the [per-version types](https://github.com/TeamDman/facet-json-migration/tree/dc5ab258957aa390cd74028afeb5d4153c89c028/src/formats).

1. Read just the version field in a small preflight type before decoding the rest of the document.
2. Decode into the type for that schema, not the newest type. Keep version selection separate from the transformations. The demo maps compatible versions between schema changes to an explicit canonical schema and rejects versions outside its supported range.
3. Upgrade through explicit conversion steps. Return the resulting document with diagnostics for changes that matter, such as inserted defaults or structural rewrites.
4. Test historical inputs, unsupported versions and the migration path, not just the latest schema's round trip.

## See the pattern in SuperFactoryManager

The [SFM toolchain lockfile parser](https://github.com/TeamDman/SuperFactoryManager/blob/9c3c474ea988240ff41e632b58c3e7b86f1e9e11/platform/cli/sfm-propagate-changes/src/toolchain_lockfile_schema/api.rs) in `sfm-propagate-changes` uses the same design with integer `schema_version` values and typed V1–V4 documents. It uses `teamy-facet-json` directly; the standalone example is a design reference, not a dependency.

The SFM example also distinguishes parsing from migration and runtime support. The parser recognizes V3 and V4, but the legacy V2 engine does not consume them. A valid document is not necessarily usable by every reader.

For compatible crate pins, see [the Facet fork ecosystem](facet-ecosystem.md).
