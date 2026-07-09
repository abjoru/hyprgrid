# CommandSource categories: dynamic menus from a command's JSON stdout

A Category may be sourced dynamically from a command instead of a static list of
EntryDefs. It is declared under a dedicated `[commands.<category>]` table with a
`command` field; at startup the command is run and its **stdout parsed as a JSON
array of EntryDef objects** — the exact same wire shape as static entries — which
become the Category's Entries. This lets users script menus (window lists, git
worktrees, playlists) without editing config.

## Considered Options

- **Output format** — JSON array of EntryDefs *(chosen)* vs. tab-separated lines
  vs. a TOML fragment. JSON reuses the existing `EntryDef` serde deserialization
  verbatim (one parsing path, all optional fields available), where line-based is
  positional and awkward for optionals and TOML is heavy for a script to emit.
- **Declaration** — a separate `[commands.<category>]` section *(chosen)* vs. an
  untagged variant inside the existing `apps` map. The separate section keeps
  static and dynamic sources in distinct namespaces and avoids a heterogeneous
  `apps` value type.
- **Failure handling** — fail-soft with an **inert error cell** *(chosen)* vs.
  fail-hard exit vs. silent empty grid. Since hyprgrid pops on a keybind, a
  visible error cell surfaces the problem without leaving a blank/absent window.

## Consequences

- The JSON-array-of-EntryDef contract and the `[commands.<category>]` schema are
  a **public config contract**: users' scripts and configs depend on them, so
  changing them later is a breaking change.
- `id` is **required** in emitted objects (EntryDef reused as-is); convenience
  such as auto-deriving `id` is deliberately out of scope for a later enhancement.
- A CommandSource whose name also exists as a static `[[apps.<category>]]` is
  shadowed — **static wins**.
- The command runs synchronously at startup with a bounded **timeout (~5s)**;
  spawn error, non-zero exit, invalid JSON, and timeout all render the inert
  error cell.
