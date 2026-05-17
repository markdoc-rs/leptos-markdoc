# leptos-markdoc

Leptos components for rendering [Markdoc](https://github.com/markdoc-rs/markdoc)
content in web and mobile UIs.

```
.mdoc source
     │
     ▼
markdoc::ext::parse  →  Node
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
   markdoc-pdf    leptos-markdoc   (other consumers)
        │              │
        ▼              ▼
       PDF       Leptos <Markdoc/> view
```

## What it does

Wraps markdoc's parse → transform → HTML render pipeline behind
two Leptos components so apps can drop Markdoc content into a view
without thinking about the AST. The pipeline is markdoc's (a port of
upstream Markdoc 0.5.11), plus two of markdoc's extension layers
(`markdoc::ext`): its bundled
tag schema, so that its tags render by default, and the frontmatter
binding, so that `{% $markdoc.frontmatter.title %}` reads the
document's own frontmatter. Upstream does neither.

- **`<Markdoc/>`** — static content, parsed once.
- **`<MarkdocReactive/>`** — bound to a `ReadSignal<String>`,
  re-renders whenever the signal changes (good for live editors and
  reactive previews).

Both render errors inline as `<div class="markdoc-error">…</div>`
(the message HTML-escaped) so the rest of the page survives a
malformed document — style this class to match your design system.
A syntax error inside a tag is not such an error: upstream Markdoc
renders the document around it. Frontmatter that is not a YAML
mapping is shown as a parse error, as before, although markdoc no
longer fails the parse over it (the document carries a
`frontmatter-invalid` error instead).

The lower-level helpers are public for apps that need to intervene
between stages:

| Helper | Does |
|--------|------|
| `default_config()` | the config the components use when given none: `markdoc::ext::bundle::Bundle::builtin().config()` |
| `default_options()` | the parse options they use when given none: upstream's, with YAML frontmatter |
| `parse_markdown(&str)`, `parse_markdown_with(&str, &ParseOptions)` | `markdoc::ext::parse` |
| `transform_ast(&Node, &Config) -> Result<Value>` | `markdoc::transform`, with the document's frontmatter bound as `$markdoc.frontmatter` |
| `render_to_html(&Value) -> Result<String>` | `markdoc::renderers::html::render` |

Variables, functions and partials go into the `Config`
(`config.variables`, `config.functions`, `config.partials`), as
upstream has them; markdoc's `ext::variables`, `ext::functions` and
`ext::partials` fill them. `{% if %}`/`{% else /%}` are evaluated at
transform time against `config.variables`.

## Usage

Static:

```rust
use leptos::*;
use leptos_markdoc::Markdoc;

#[component]
fn App() -> impl IntoView {
    let markdown = "# Hello\n\nThis is **Markdoc**.";
    view! { <Markdoc content=markdown.to_string() /> }
}
```

Reactive (live preview):

```rust
use leptos::*;
use leptos_markdoc::MarkdocReactive;

#[component]
fn Editor() -> impl IntoView {
    let (content, set_content) =
        create_signal(String::from("# Edit me!"));

    view! {
        <textarea
            on:input=move |ev| set_content(event_target_value(&ev))
            prop:value=content
        />
        <MarkdocReactive content=content />
    }
}
```

Passing a custom `Config` (for tag schemas, variables, functions,
partials) and `ParseOptions` is optional on both components:

```rust
use markdoc::ext::bundle::Bundle;
use markdoc::ext::variables::Bindings;

// markdoc's bundled tags, plus a corpus's own overlay.
let mut bundle = Bundle::builtin();
bundle.overlay(include_str!("tags.overlay.toml"))?;
let mut config = bundle.config();
Bindings::new().with_variable("product", &product).apply(&mut config);

view! { <Markdoc content=md config=config options=bundle.parse_options() /> }
```

The `Config` is `markdoc::schema::Config`, and markdoc's own
`Config::default()` is upstream's empty one: every tag would render
only its children. Start from `leptos_markdoc::default_config()` or a
`Bundle` instead. The default options read the frontmatter as YAML
and leave every other switch off (indentation, comments, the
typographer, byte-order-mark stripping): pass a corpus's options
(`bundle.parse_options()`, from its `[parse]` table) to read its
files as its other tools do.

### What the output looks like

Upstream Markdoc's HTML, which differs from the pipeline this crate
used before:

- the root is `<article>`, inside the component's `<div>`;
- a tag that no schema declares renders only its children, and only
  declared attributes (plus `class` and `id`) are rendered;
- attributes come in schema order; `<br>` rather than `<br />`; a
  soft line break is a space; `'` is not escaped;
- an interpolated number prints as JavaScript prints it, and an
  interpolated `true`, `false`, `null` or undefined value prints
  nothing (in an attribute, a value prints as `String(value)`).

A runnable sketch lives in [`examples/leptos-basic.rs`](examples/leptos-basic.rs).

## Why a separate crate

`markdoc` itself is framework-agnostic — it parses CommonMark plus
the `{% tag %}` extension and exposes an AST plus an HTML renderer.
Pinning a specific UI framework into that crate would force every
consumer (PDF renderer, CLI, server-side validators) to pull in
Leptos. Splitting the integration keeps:

- `markdoc` Leptos-free and reusable from any context.
- This crate small — it's <100 lines of glue.
- Framework choice as a downstream decision: a future
  `dioxus-markdoc` or `yew-markdoc` would slot in the same way.

## Dependencies

| Crate | Why |
|-------|-----|
| `markdoc` (path) | parse / transform / HTML render |
| `leptos` 0.8 | component framework |
| `leptos_meta`, `leptos_router` 0.8 | re-exported for app convenience |

The library name is pinned to `leptos_markdoc` in `Cargo.toml`
(`[lib] name = ...`) so downstream `use leptos_markdoc::…` works
despite the hyphenated package name.

## SSR

The components emit HTML via Leptos' `inner_html` attribute, which
works in both CSR and SSR modes. Parse / transform happen in pure
Rust with no DOM access, so server-side rendering produces the same
output as the client.

## Status

v0.1 — usable but minimal. Currently:

- Renders upstream Markdoc's HTML output (headings, lists, tables,
  fenced code, links, images, custom tags via `Config`).
- Surfaces parse, transform and render errors visually.
- Does not (yet) wire up the platform-specific bits — loading
  partials into `config.partials`, binding `$frontendType`, asset
  URI resolution — those happen upstream of the component for now.

## License

MIT.
