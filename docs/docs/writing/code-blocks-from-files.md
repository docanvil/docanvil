---
{
  "title": "Code Blocks from Files",
  "description": "Show real source files, or just the lines that matter, with line numbers and captions"
}
---

# Code Blocks from Files

Code samples copied into docs drift away from the real code. Point a code block at the source file instead and it's read fresh on every build — when the code changes, the docs change with it.

## Showing a file

Add `file` to an empty fenced code block:

````markdown
```toml file="/docanvil.toml"
```
````

The block fills with the file's contents and gets a caption with its path. The language comes from the fence if you give one, otherwise from the file's extension.

### Where paths point

:::include{file="../_shared/paths.md"}

## Showing some lines

Add `lines` to show only part of the file. Here's this site's own `docanvil.toml`, trimmed to two sections:

````markdown
```toml file="/docanvil.toml" lines="1-3,7-9"
```
````

```toml file="/docanvil.toml" lines="1-3,7-9"
```

Lines skipped between ranges are marked with a `⋯ N lines hidden` row, and the block shows the file's real line numbers so readers can find their place in the source.

| `lines` value | Shows |
|---|---|
| `7` | line 7 |
| `5-12` | lines 5 to 12 |
| `5-` | line 5 to the end of the file |
| `-12` | the start of the file to line 12 |
| `1-6,30-35` | several ranges, in order |

Ranges must go in ascending order and can't overlap. A range that runs past the end of the file is an error rather than being trimmed quietly — if the source file shrinks, you'll hear about it at build time instead of shipping a sample that no longer matches.

Indentation shared by all the selected lines is removed, so a method pulled from the middle of a class starts at the left edge.

## Line numbers

Line numbers work on any code block, not just file blocks:

| Attribute | Effect |
|---|---|
| `numbers` | Number the lines from 1 (file blocks: their real line numbers) |
| `numbers="10"` | Start counting at 10 |
| `numbers="false"` | No numbers, even where they'd be on by default |

File blocks with `lines` are numbered by default. To number every code block on the site, turn it on in `docanvil.toml`:

```toml
[syntax]
line_numbers = true
```

Numbers are drawn by the theme rather than written into the code, so the copy button copies just the code.

## Captions

`title` adds a caption to any code block, or replaces the default caption of a file block. `title=""` removes it.

````markdown
```rust title="src/main.rs"
fn main() {}
```
````

## Inside code groups and tabs

File blocks work anywhere a code block does, including `:::code-group` and `:::tabs`:

````markdown
:::code-group
```rust file="/examples/hello.rs"
```
```python file="/examples/hello.py"
```
:::
````

## When something's wrong

A missing file, a file that isn't UTF-8, a `lines` range past the end of the file, or a code block with both `file` and code of its own shows an error box on the page and a build warning with the file and line. `docanvil build --strict` fails on any of them, and `docanvil doctor` checks them all up front.

## Related Pages

- [[writing/includes|Includes]]
- [[writing/markdown|Markdown]]
- [[guides/configuration|Configuration]]
