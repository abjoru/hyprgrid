# hyprgrid

Grid-based application launcher for Hyprland. Reads a TOML config of apps grouped into categories, packs the chosen category into a diamond-shaped grid, and launches the selected app.

## Language

**Entry**:
A single launchable item — id, name, optional description/icon, and how to launch it. Its TOML wire form is an **EntryDef**.
_Avoid_: app, item, button.

**EntryDef**:
The TOML wire shape of an Entry (a flat `command` + `terminal` bool) that `From<EntryDef>` fuses into the launch intent. Deserialization-only.
_Avoid_: raw entry, config struct.

**Category**:
A named group of Entries selected at startup (e.g. `favorites`). Exactly one is displayed per run.
_Avoid_: section, page, tab.

**CategoryMap**:
The on-disk map of every Category to its EntryDefs; startup selects one.
_Avoid_: apps, config.

**GridLayout**:
The pure geometry of a launcher screen: the diamond packing of N Entries onto integer coordinates, the layer of each, the bounds, and the wrap-and-scan navigation between them. Knows only the count of Entries, never their contents — no GTK.
_Avoid_: grid, geometry, map.

**Layer**:
The diamond ring an Entry sits on, counted outward from the centre (layer 0). Drives the Accent assigned to a Cell.
_Avoid_: ring, level, row.

**Direction**:
A single navigation step — Left, Right, Up, or Down. The input vocabulary of `GridLayout::step`.
_Avoid_: dx/dy, delta, vector.

**Selection**:
The GTK-side adapter over a GridLayout: holds the Cells, tracks the selected index, and repaints on each Direction step.
_Avoid_: state, cursor.

**Cell**:
The GTK widget rendering one Entry at one coordinate, styled with an Accent.
_Avoid_: tile, box, widget.

**Accent**:
A background colour assigned to a Cell by its Layer, cycling through the theme's accent palette.
_Avoid_: colour, highlight.

**Invocation**:
A resolved, runnable command derived from an Entry's launch intent: the final tilde-expanded shell string plus the binary name to check on PATH. Pure and total — building one cannot fail; only running it can. Distinct from the launch *intent* (Terminal vs Command) and from the OS spawn mechanism that runs it.
_Avoid_: command, process, exec.

## Relationships

- A **Category** contains one or more **Entries**
- An **Entry**'s launch intent resolves to an **Invocation**, which is then run (the spawn can fail; resolving cannot)
- A **GridLayout** is built from a count of **Entries** and assigns each a coordinate and a **Layer**
- A **Layer** maps to an **Accent**
- A **Selection** wraps one **GridLayout** and renders one **Cell** per Entry
- A **Direction** drives `GridLayout::step`, which the **Selection** uses to repaint

## Example dialogue

> **Dev:** "When I press right from the rightmost **Cell**, does the **Selection** know where to go?"
> **Maintainer:** "No — the **Selection** asks the **GridLayout** to `step` Right. The **GridLayout** wraps to the opposite edge and scans for the next occupied coordinate, then hands back an index. The **Selection** just repaints that **Cell**."

## Flagged ambiguities

- "grid" was used for both the GTK `Grid` widget and the abstract packing — resolved: the pure packing/navigation is **GridLayout**; the GTK side is the **Selection** (holding **Cells**).
