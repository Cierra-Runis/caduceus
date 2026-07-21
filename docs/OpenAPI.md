# API type sharing (OpenAPI pipeline)

The Rust server is the single source of truth for the REST wire format. Types
flow one way:

```
server/src (utoipa annotations)
  --> cargo run --bin gen_openapi
    --> docs/openapi.json                      (checked in)
      --> pnpm codegen:api  (in app/)
        --> app/src/lib/api/generated/schemas.ts   (checked in)
```

The WebSocket protocol under `/ws` is out of scope.

## Regenerating

After changing any payload/request struct or handler annotation in `server/`:

```sh
cd server && cargo run --bin gen_openapi   # rewrites docs/openapi.json
cd app && pnpm codegen:api                 # rewrites src/lib/api/generated/schemas.ts
```

Commit both generated files.

## Drift guards

- `server`: the `openapi::tests::checked_in_spec_is_up_to_date` test fails when
  `docs/openapi.json` no longer matches the annotations (runs in Rust CI).
- `app`: `pnpm test:api-contract` (runs in the api-contract workflow)
  1. regenerates the Zod schemas in memory and fails if the checked-in
     `generated/schemas.ts` is stale, and
  2. runs `scripts/check-api-drift.ts`: per endpoint, a wire-shaped fixture
     must be accepted by the generated schema (fixture matches the spec) *and*
     by the hand-written schema the app actually uses (app matches the server),
     and the hand-written date transforms must yield valid `Date`s.

## Adding an endpoint

1. Derive `utoipa::ToSchema` on any new request/payload struct. Fields that
   serde writes as strings but utoipa cannot infer need overrides:
   `ObjectId` -> `#[schema(value_type = String)]`, `semver::Version` ->
   `#[schema(value_type = Option<String>)]`. Always-serialized `Option` fields
   need `#[schema(required)]`.
2. Add `#[utoipa::path(...)]` on the handler; success bodies are
   `ApiSuccess<T>`, error/no-payload bodies are `ApiMessage`
   (`server/src/openapi.rs`).
3. List the handler in `ApiDoc` (`server/src/openapi.rs`) and regenerate.
4. Add a fixture case in `app/scripts/check-api-drift.ts`.

## Layering in the app

Generated schemas describe the raw wire shape only. The hand-written schemas in
`app/src/lib/types/` and `app/src/lib/api/` remain the app-facing layer: they
add `.trim()` and string->`Date` transforms that UI code relies on, and they
intentionally omit fields the app does not use (Zod strips unknown keys).
The drift check keeps the two layers structurally compatible. Migrating the
app-facing layer to compose the generated schemas directly is a follow-up.
