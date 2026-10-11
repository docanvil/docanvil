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

## Showing a remote file

Use `url` instead of `file` for a public raw HTTPS file URL:

````markdown
```rust url="https://raw.githubusercontent.com/docanvil/docanvil/1703691ee41cddae60408c170bfeae1e696f3249/src/pipeline/markdown.rs" lines="27-31" title="src/pipeline/markdown.rs"
```
````

Here is that block rendered from DocAnvil’s own GitHub source: the function that turns Markdown into HTML. The URL is pinned to a commit, so the selected lines stay stable.

```rust url="https://raw.githubusercontent.com/docanvil/docanvil/1703691ee41cddae60408c170bfeae1e696f3249/src/pipeline/markdown.rs" lines="27-31" title="src/pipeline/markdown.rs"
```

Raw file URLs from GitHub, GitLab, Bitbucket and other hosts work. Copy the link to the **raw file**, rather than the repository page that displays it. URLs returning HTML are rejected. Private repositories requiring authentication are not supported.

`lines`, `numbers`, indentation removal and `title` work just as they do for local files. Without an explicit language, DocAnvil uses the URL path's file extension. The default caption is the URL; use `title` to replace or hide it.

Choose either `file` or `url` and leave the block empty. Remote URLs must use HTTPS and must not contain credentials or a fragment such as `#L10`; use `lines` to select lines instead. `:::include` continues to accept local Markdown files only.

Each distinct URL is fetched once per build, doctor run or PDF export. There is no persistent cache, so a new run fetches fresh content. Requests have a 15-second timeout, a 2 MiB response limit and at most five redirects. Remote changes are picked up on the next rebuild; they do not trigger the development server's file watcher.

Pin URLs to a commit rather than a moving branch when you need reproducible builds. Download failures, invalid UTF-8 and invalid line ranges use the same error boxes, warnings and strict-build checks as local includes.

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
