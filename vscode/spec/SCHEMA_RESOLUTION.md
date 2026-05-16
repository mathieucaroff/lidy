# Lidy VS Code Schema Resolution Specification

## 1. Scope

This document specifies how the VS Code extension selects a Lidy schema for a YAML file.

The goals are:

- allow repository-wide schema association without using the command line
- allow per-file schema selection
- allow users to use their own custom schemas
- allow users to copy an official Lidy schema, modify it, and use the modified version
- keep the resolution rules deterministic and easy to explain

This specification only covers schema selection and retrieval. It does not define the full linting or completion behavior after a schema has been selected.

## 2. Resolution Sources

The extension resolves a schema for a YAML file from the following sources, ordered from highest priority to lowest priority:

1. `lidy.override.<n>.yaml`
2. in-file comment directive
3. `lidy.config.yaml`
4. autodetection

The first source that yields a schema wins.

No manual extension-only override state is defined in this version of the specification. All schema selection must be expressed in files.

## 3. Repository Configuration Files

### 3.1 Supported file names

The extension recognizes the following repository-level configuration files:

- `lidy.config.yaml`
- `lidy.override.<n>.yaml`

`<n>` must be a positive integer written in base 10 with no leading zero.
Furthermore, the extension `.yml` is unsupported

Valid examples:

- `lidy.override.0.yaml`
- `lidy.override.1.yaml`
- `lidy.override.2.yaml`
- `lidy.override.10.yaml`

Invalid examples due to unrecognized level:

- `lidy.override.01.yaml`
- `lidy.override.-1.yaml`
- `lidy.override.local.yaml`

Invalid examples due to unrecognized extension:

- `lidy.config.yml`
- `lidy.override.2.yml`
- `lidy.override.10.yml`

Any file matching `lidy.{config,override.*}.y{a,}ml` but being an invalid name
produce a warning in the `Problems` section of VSCode.

### 3.2 Override file ordering

If multiple `lidy.override.<n>.yaml` files are present, they are ordered by their numeric suffix.

- lower numbers have lower priority
- higher numbers have higher priority

Example:

- `lidy.override.2.yaml` has lower priority than `lidy.override.10.yaml`

### 3.3 Configuration file format

The content of `lidy.config.yaml` and `lidy.override.<n>.yaml` is the same.

Example:

```yaml
associations:
  - pattern: "deploy/**/*.yaml"
    schema: "schema/kubernetes.schema.yaml"
```

Format:

```yaml
associations:
  - pattern: <glob pattern>
    schema: <schema reference>
```

Rules:

- `associations` must be a sequence
- each entry in `associations` must be a map
- each entry must contain `pattern` and `schema`
- `pattern` must be a string
- `schema` must be a string

The lidy schema defining the configuration file format is:

```yaml
main:
  _map:
    associations: associationList
  _mapOf: { string: any }

associationList:
  _listOf: associationEntry

associationEntry:
  _map:
    pattern: string
    schema: string
```

Unknown top-level keys produce an error report line in the Problems section of VSCode.

### 3.4 Matching within one configuration file

If several associations in the same file match the same YAML document, the last matching association wins.

This allows users to define broad defaults first and then refine them later in the same file.

## 4. In-File Comment Directive

### 4.1 Syntax

A YAML file may declare its schema with a comment directive:

```yaml
# lidy-schema: ../custom.schema.yaml
```

The directive value may be either:

- a relative or absolute file path
- an HTTPS URL

Examples:

```yaml
# lidy-schema: ./schema/custom.schema.yaml
```

```yaml
# lidy-schema: https://raw.githubusercontent.com/mathieucaroff/lidy/5b40dffc/schema/gitlab.schema.yaml
```

### 4.2 Directive placement

The extension should only consider directive comments found near the start of the file.

Recommended first implementation:

- scan comments from the start of the document
- stop scanning when the first non-comment, non-blank YAML content is reached
- if multiple `# lidy-schema:` directives are found before that point, the first one wins and
  an error is reported in the Problems section of VSCode.

This keeps the rule simple and avoids interpreting unrelated comments deeper in the file.

### 4.3 Path resolution

If the directive value is a relative file path, it is resolved relative to the YAML file containing the directive.

Examples:

- file: `services/api/deploy.yaml`
- directive: `# lidy-schema: ../schema/k8s.schema.yaml`
- resolved path: `services/schema/k8s.schema.yaml`

Absolute local paths may be supported, but relative paths are preferred.

## 5. Schema Reference Types

The `schema` value in config files and the value in `# lidy-schema:` may point to one of the following:

1. local file path
2. HTTPS URL

### 5.1 Local file path

Examples:

- `schema/kubernetes.schema.yaml`
- `./schema/custom.schema.yaml`
- `../schemas/company/openapi.schema.yaml`

In repository configuration files, relative paths are resolved relative to the configuration file containing the reference.

In in-file comment directives, relative paths are resolved relative to the YAML document containing the directive.

### 5.2 HTTPS URL

Examples:

- `https://raw.githubusercontent.com/mathieucaroff/lidy/5b40dffc/schema/gitlab.schema.yaml`
- `https://example.com/schemas/custom.schema.yaml`

Only `https` URLs should be accepted in the first implementation.

`http` URLs should be rejected.

## 6. Full Resolution Algorithm

For a given YAML file, the extension resolves the schema as follows:

1. Discover repository configuration files applicable to the YAML file.
   `lidy.config.yaml` is a configuration file with priority `-1`.
   Each validly named `lidy.override.<n>.yaml` is a configuration file with priority `n`.
2. Report a diagnostic for each file whose name matches `lidy.override.*.yaml` but whose suffix is not a valid positive integer without a leading zero.
   Invalid override file names do not participate in resolution.
3. Process all discovered configuration files independently.
   For each configuration file:

- load and parse the file
- ignore the file if the YAML document is unparseable or its top-level value is not a map
- otherwise, treat the file as having no usable `associations` if the `associations` key is missing or is not a sequence
- otherwise, iterate through `associations` in file order and ignore entries whose `pattern` or `schema` is missing or is not a string
- among the remaining entries, keep the last matching one
- report unparseable YAML, excess and missing keys and values of the wrong type as VSCode Problems.

4. If one or more `override` files produced a match, select the schema reference from the highest-priority matching configuration file.
   Resolve that reference.

- if the reference is syntactically invalid, report a diagnostic and stop with no selected schema
- if the referenced schema cannot be loaded, fetched, or parsed as a Lidy schema, report a diagnostic and stop with no selected schema
- otherwise use that schema and stop

5. Inspect the YAML file for a `# lidy-schema:` directive near the start of the file, according to the directive-placement rules defined above.
6. If a directive is found, use the first directive found before the first non-comment, non-blank YAML content and report a diagnostic if additional directives are present in that same prefix.
   Resolve the directive value.

- if the directive value is syntactically invalid, report a diagnostic and stop with no selected schema
- if the referenced schema cannot be loaded, fetched, or parsed as a Lidy schema, report a diagnostic and stop with no selected schema
- otherwise use that schema and stop

7. If `lidy.config.yaml` produced a match, resolve its schema reference.

- if the reference is syntactically invalid, report a diagnostic and stop with no selected schema
- if the referenced schema cannot be loaded, fetched, or parsed as a Lidy schema, report a diagnostic and stop with no selected schema
- otherwise use that schema and stop

8. If no schema has been selected yet, apply autodetection based on the rules defined in `schema-autodetection.json`, using only the file name and the name of the containing directory.
9. If autodetection yields a schema reference, resolve it.

- if resolution succeeds, use that schema and stop
- otherwise report a diagnostic and stop with no selected schema

10. Otherwise, no schema is selected.

## 7. Autodetection

Autodetection is intentionally limited in scope.

The autodetection rules are defined by the content of `schema-autodetection.json` packaged with the extension.
In particular, autodetection by path must be driven by that file's `autodetectionByPath` entries.

The current shape of `schema-autodetection.json` is:

```json
{
  "autodetectionByPath": [
    {
      "schema": "gitlab.schema.yaml",
      "pathList": [".gitlab-ci.y{a,}ml", "**/.gitlab/**/*.y{a,}ml"]
    }
  ]
}
```

`autodetectionByPath` is a sequence of autodetection rules.
Each rule is a map with:

- `schema`: the file name of a packaged schema in the extension's schema set
- `pathList`: a sequence of glob patterns

Each pattern in `pathList` is matched against the YAML file path.
If a pattern matches, the corresponding `schema` becomes an autodetection candidate.

This file is therefore the data source that maps path patterns such as `.github/workflows/*.y{a,}ml` or `.gitlab-ci.y{a,}ml` to packaged schema file names such as `github.schema.yaml` or `gitlab.schema.yaml`.

The first implementation only uses:

- the file name
- the name of the directory containing the file

It does not inspect document content.

Examples of possible heuristics:

- `.gitlab-ci.yml` -> GitLab CI schema
- files under `.github/workflows/` -> GitHub Actions schema
- `docker-compose.yml` or `compose.yaml` -> Docker Compose schema

Autodetection is a fallback only. It must never override a matching config or directive.

## 8. Using Official and Custom Schemas

This specification intentionally does not distinguish between official schemas and user schemas at the resolution level.

Any schema reference may point to:

- a schema authored by the user
- a copy of an official Lidy schema stored in the repository
- a remote schema published over HTTPS

The extension itself also packages the official Lidy schemas that belong to the same publication as the installed extension version.

These packaged schemas are authoritative local copies of the official schemas shipped with the extension.

For each packaged official schema, the extension must know one or more canonical `raw.githubusercontent.com` URLs that identify that schema.
These canonical URLs should preferably be tag-based URLs associated with the released extension version, rather than commit-based URLs.

If any schema reference appearing in `lidy.config.yaml`, `lidy.override.<n>.yaml`, or `# lidy-schema:` is equal to one of these known canonical official URLs, the extension must resolve that reference to the packaged local copy of the schema.

This substitution is part of schema resolution itself, not a cache optimization and not a best-effort fallback.
When such a URL is recognized, the extension must not attempt any network access for it.

If both tag-based and commit-based canonical URLs are known for the same packaged schema, both may be accepted, but the tag-based URL is the preferred published form.

This ensures that users can:

- use their own schema directly
- copy an official schema into their repository
- modify that copied schema
- point `lidy.config.yaml`, `lidy.override.<n>.yaml`, or `# lidy-schema:` to the modified copy
- refer to an official schema by its canonical URL while still using the packaged local copy

## 9. Remote Schema Retrieval

HTTPS URLs are allowed in order to support direct use of published schemas, including official Lidy schemas stored in a repository.

Known canonical URLs for packaged official schemas are not treated as ordinary remote schemas.
They are resolved directly to packaged local copies as described in Section 8 and therefore must bypass network retrieval entirely.

### 9.1 Intended usage

The preferred form for official schema URLs is a tag-pinned URL such as:

```text
https://raw.githubusercontent.com/mathieucaroff/lidy/refs/tags/v1.2.3/schema/gitlab.schema.yaml
```

This form is preferred because it is stable, readable, and can be aligned with the published extension version that already packages the same schema locally.

If tag-based URLs cannot be provided for some reason, a commit-pinned URL remains acceptable as a compatibility form:

```text
https://raw.githubusercontent.com/mathieucaroff/lidy/5b40dffc/schema/gitlab.schema.yaml
```

Commit-pinned URLs are also stable and reproducible, but they are less readable and less directly tied to the user-facing extension release.

Branch-based URLs may be supported, but they are less reproducible and may change unexpectedly.

### 9.2 Network and caching model

To make URL-based schemas workable in an editor, the extension should not fetch the remote document on every edit.

Before any remote fetch is attempted, the extension must first check whether the URL is one of the known canonical official schema URLs for the installed extension version.
If so, it must resolve that URL to the packaged local copy and skip network access entirely.

This check applies regardless of where the URL came from, including:

- `lidy.config.yaml`
- `lidy.override.<n>.yaml`
- `# lidy-schema:`

Recommended behavior:

1. if the URL matches a known canonical official URL, resolve it immediately to the packaged local copy and do not fetch it
2. otherwise, when a remote schema is first needed, download it over HTTPS
3. store it in a local cache managed by the extension
4. use the cached copy for validation and completion
5. if a cache entry is used while older than 1h and the device is online,
   schedule a refresh of the cache entry to be run asap

The first implementation should support one or more of these refresh triggers:

- explicit user command such as `Lidy: Refresh Remote Schemas`
- reload when the configured URL changes
- optional time-based refresh with a conservative cache lifetime

An explicit refresh command is the safest default.

### 9.3 Failure handling

If a remote schema cannot be fetched, the extension should:

- keep using the last successfully cached version if one exists
- report a clear diagnostic or status message
- never silently pretend that validation succeeded

This failure mode does not apply to known canonical official schema URLs that are resolved to packaged local copies, because those URLs must not trigger a fetch in the first place.

Example failure messages:

- `Failed to fetch Lidy schema from https://...`
- `Cached schema used because the remote source is unavailable`

### 9.4 Security constraints

The first implementation should apply the following constraints:

- only `https` URLs are accepted
- redirects should be limited to 3 levels
- users using the commands to add a schema must confirm they trust the source
  (the URL) they give
- remote schema loading should be disabled in restricted workspaces

If a workspace trust model is used, remote schema fetching should respect it.

## 10. Commands and Completion

To make this model usable, the extension should provide authoring assistance.

### 10.1 Commands

Recommended commands:

- `Lidy: Insert Schema Directive`
- `Lidy: Add Association To lidy.config.yaml`
- `Lidy: Add Association To Override File`
- `Lidy: Refresh Remote Schemas`

`Lidy: Insert Schema Directive` should help the user insert:

- a relative file path
- an HTTPS URL

### 10.2 Completion

When editing a line that starts with `# lidy-schema:`, the extension should offer completion for:

- relative file paths in the workspace
- existing local schema files
- optionally known remote URLs previously used or cached

Completion should prefer local relative paths when the schema file is inside the workspace.

## 11. Diagnostics

The extension should surface clear problems when schema selection fails.

Examples:

- invalid config file name: `lidy.override.01.yaml`
- malformed config content
- unresolved local schema path
- unsupported URL scheme
- failed remote fetch
- invalid Lidy schema content

These diagnostics should make it clear whether the problem is:

- selecting the schema
- retrieving the schema
- parsing the schema

## 12. Summary of Deterministic Rules

The following rules are normative:

1. schema resolution precedence is: override files, then in-file directive, then `lidy.config.yaml`, then autodetection
2. only one `lidy.config.yaml` is recognized
3. multiple `lidy.override.<n>.yaml` files are allowed
4. override file priority is numeric, and higher numbers win
5. within one config file, the last matching association wins
6. in-file directive relative paths resolve relative to the YAML file containing the directive
7. config-file relative paths resolve relative to the config file containing the association
8. only `https` remote URLs are accepted
9. autodetection uses only the file name and the name of the containing directory

## 13. Non-Goals for This Version

The following features are intentionally out of scope for this version:

- manual ephemeral extension-only schema overrides
- content-based autodetection
- multiple non-override config files
- non-HTTPS remote schemas
- a special resolution path for official schemas distinct from other schema references
