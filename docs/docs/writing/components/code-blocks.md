---
{
  "title": "Code Blocks",
  "description": "Fenced code, syntax highlighting, code groups, source files, diffs and highlighted lines",
  "slug": "code-blocks",
  "redirect_from": [
    "writing/code-blocks-from-files"
  ]
}
---

# Code Blocks

Code blocks present examples with syntax highlighting, language labels and a Copy button. Add captions, line numbers, highlighted lines or diff backgrounds, or load code directly from a local or remote source.

## Writing a code block

Use a fenced block with the source language after the opening backticks:

````markdown
```rust
fn main() {
    println!("Hello from DocAnvil!");
}
```
````

```rust
fn main() {
    println!("Hello from DocAnvil!");
}
```

The language selects syntax highlighting and the header label. Without a language, the block uses plain text with the configured syntax theme's colours. Choose the token palette with `[syntax].theme`; see [[guides/configuration|Configuration]] for palette and theme styling options.

## Code Group

A specialized tab component for comparing code blocks across languages. Each fenced code block becomes a tab, with the language name as the tab label:

:::code-group
```rust
fn greet(name: &str) {
    println!("Hello, {name}!");
}
```

```python
def greet(name):
    print(f"Hello, {name}!")
```

```javascript
function greet(name) {
  console.log(`Hello, ${name}!`);
}
```
:::

Raw syntax:

````markdown
:::code-group
```rust
fn greet(name: &str) {
    println!("Hello, {name}!");
}
```

```python
def greet(name):
    print(f"Hello, {name}!")
```
:::
````

## Showing a file

Point an empty block at a source file to read it fresh on every build, keeping examples in sync with the code.

Add `file` to an empty fenced code block:

````markdown
```toml file="/docanvil.toml"
```
````

The block fills with the file's contents and gets a caption with its path. The language comes from the fence if you give one, otherwise from the file's extension.

### Where paths point

:::include{file="../../_shared/paths.md"}

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

`lines`, `region`, `numbers`, indentation removal and `title` work just as they do for local files. Without an explicit language, DocAnvil uses the URL path's file extension. The default caption is the URL; use `title` to replace or hide it.

Choose either `file` or `url` and leave the block empty. Remote URLs must use HTTPS and must not contain credentials or a fragment such as `#L10`; use `lines` to select lines instead. `:::include` continues to accept local Markdown files only.

Each distinct URL is fetched once per build, doctor run or PDF export. There is no persistent cache, so a new run fetches fresh content. Requests have a 15-second timeout, a 2 MiB response limit and at most five redirects. Remote changes are picked up on the next rebuild; they do not trigger the development server's file watcher.

Pin URLs to a commit rather than a moving branch when you need reproducible builds. Download failures, invalid UTF-8 and invalid line ranges use the same error boxes, warnings and strict-build checks as local includes.

## Diff blocks

Add `diff` after the source language (for example, `rust diff`) to keep syntax highlighting while showing additions on a green background and removals on a red background. The header shows the source language with a separate diff badge. File headers and `@@` hunk markers have separate styling; context lines keep their usual appearance. The original `+` and `-` markers remain in copied text.

````markdown
```rust diff title="Update the greeting"
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,3 @@
 fn main() {
-    println!("Hello world");
+    println!("Hello DocAnvil");
 }
```
````

```rust diff title="Update the greeting"
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,3 @@
 fn main() {
-    println!("Hello world");
+    println!("Hello DocAnvil");
 }
```

The Rust keywords, macro and strings retain their syntax colours on both changed and context lines. Only the line backgrounds indicate additions and removals.

Store an actual `.diff` or `.patch` file in your project and reference it with `file`, or use `url` for a remote raw file:

````markdown
```rust file="/examples/greeting.diff"
```

```rust url="https://raw.githubusercontent.com/your-org/your-repo/main/examples/greeting.diff"
```
````

Diff mode is inferred from `.diff` and `.patch` source extensions; specify the code language in the fence because a patch can contain any language. For other source extensions, add `diff` explicitly. Use `text diff` for plain text changes. Existing `diff` and `patch` fences still render plain text diffs. Captions, line selection, line numbers, code groups and tabs work as for other code blocks. Numbers count lines in the diff file, rather than old and new source line numbers. Diff colours also appear in PDF exports and when syntax highlighting is disabled.

Themes can customise the diff badge in their CSS using `--code-block-diff-badge-color`, `--code-block-diff-badge-bg` and `--code-block-diff-badge-border`. For example:

```css
:root {
  --code-block-diff-badge-color: #fff4cf;
  --code-block-diff-badge-bg: #674b16;
  --code-block-diff-badge-border: #c69a3c;
}
```

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

## Showing a named region

Use `region` instead of line ranges to keep a sample stable when lines move:

````markdown
```rust file="/examples/hello.rs" region="quickstart"
```
````

Place paired markers in the source file:

```rust
// {== region="quickstart" ==}
fn main() {
    println!("Hello!");
}
// {== endregion="quickstart" ==}
```

DocAnvil recognises the `{== region="name" ==}` and `{== endregion="name" ==}` wrappers regardless of the surrounding comment syntax. For example, Python can use `# {== region="quickstart" ==}` and HTML can use `<!-- {== region="quickstart" ==} -->`. The source file is never changed.

**Put each marker on a full, single-line comment containing no source code.** DocAnvil removes the entire marker line without checking the language's comment syntax. When using block comment delimiters, open and close the comment on that same line. Markers inside a multiline block comment may leave unmatched delimiters or change whether the copied code is commented out.

Names are case-sensitive and must be nonempty and unique within the file. Regions can nest, but must close in order with matching names. All marker lines inside the selected region are omitted, including nested markers. Source line numbers are preserved, shared indentation is removed, and numbers appear by default; `numbers="false"` hides them.

`region` works with both `file` and `url`. It cannot be combined with `lines` or a custom starting number such as `numbers="10"`. Missing, malformed, duplicate, unmatched or empty regions produce the existing build warnings and error boxes, and fail strict builds. The `{== … ==}` wrapper is reserved for region markers when selecting a region.

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

## Highlighting lines

Use `highlight` to draw attention to existing code without marking it as added or removed. Syntax colours remain visible beneath a subtle background and edge marker.

````markdown
```rust highlight="2-3,5" title="Configure the greeting"
fn main() {
    let name = "DocAnvil";
    let greeting = format!("Hello {name}");

    println!("{greeting}");
}
```
````

```rust highlight="2-3,5" title="Configure the greeting"
fn main() {
    let name = "DocAnvil";
    let greeting = format!("Hello {name}");

    println!("{greeting}");
}
```

The syntax matches `lines`: `7`, `5-12`, `5-`, `-12` and comma-separated selections such as `1-3,7,10-12`. Ranges must be ascending, non-overlapping and within the source. Invalid selections produce include diagnostics and fail strict builds.

On inline blocks, selections count from 1 independently of `numbers="N"`. On `file` or `url` blocks, they refer to original source line numbers, including when combined with `lines` or `region`; selected highlight lines outside the displayed snippet stay hidden. Highlighting does not enable line numbers or change copied text. It works in code groups, tabs, PDF exports and with syntax highlighting disabled. In diff blocks, the edge marker adds emphasis while the red/green backgrounds keep their change meaning.

Themes can set `--code-block-highlight-bg` and `--code-block-highlight-border` in their CSS.

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
