//! Leptos components that render Markdoc content.
//!
//! The pipeline is markdoc's (a port of upstream Markdoc 0.5.11) plus two of its extension
//! layers (`markdoc::ext`): the schema bundle markdoc ships, so that its tags render by default
//! ([`default_config`]), and the frontmatter binding, so that
//! `{% $markdoc.frontmatter.title %}` reads the document's own frontmatter
//! ([`transform_ast`]). Upstream does neither.

use leptos::prelude::*;
use markdoc::ast::Node;
use markdoc::ext::bundle::Bundle;
use markdoc::ext::variables::frontmatter_variables;
use markdoc::ext::{Frontmatter, ParseOptions};
use markdoc::renderers::html;
use markdoc::schema::Config;
use markdoc::schema::value::Value;
use markdoc::types::Result;

/// Component to render Markdoc content in Leptos
#[component]
pub fn Markdoc(
    /// The markdown content to parse and render
    content: String,
    /// The schema (tags, nodes, variables, functions, partials); [`default_config`] if absent
    #[prop(optional)]
    config: Option<Config>,
    /// How the content is parsed; [`default_options`] if absent
    #[prop(optional)]
    options: Option<ParseOptions>,
) -> impl IntoView {
    let config = config.unwrap_or_else(default_config);
    let options = options.unwrap_or_else(default_options);

    let html = move || render_page(&content, &options, &config);

    view! {
        <div inner_html=html()></div>
    }
}

/// Component to render Markdoc from a reactive signal
#[component]
pub fn MarkdocReactive(
    /// Reactive signal containing the markdown content
    content: ReadSignal<String>,
    /// The schema (tags, nodes, variables, functions, partials); [`default_config`] if absent
    #[prop(optional)]
    config: Option<Config>,
    /// How the content is parsed; [`default_options`] if absent
    #[prop(optional)]
    options: Option<ParseOptions>,
) -> impl IntoView {
    let config = config.unwrap_or_else(default_config);
    let options = options.unwrap_or_else(default_options);

    let html = move || render_page(&content.get(), &options, &config);

    view! {
        <div inner_html=html()></div>
    }
}

/// The config the components use when none is passed: the schema bundle markdoc ships
/// (`schema/tags.toml`), so that its tags (`callout` and its alias `notice`, `media` and its
/// aliases, ...) render under their own names, with their attribute defaults.
///
/// markdoc's own `Config::default()` is upstream's empty config, under which a tag renders only
/// its children. A corpus with tags of its own adds its overlay:
/// `let mut bundle = Bundle::builtin(); bundle.overlay(text)?; bundle.config()`.
pub fn default_config() -> Config {
    Bundle::builtin().config()
}

/// The parse options the components use when none are passed: upstream's, except that the
/// frontmatter is read as YAML, so that `$markdoc.frontmatter` is an object. Every other
/// option (indentation, comments, the typographer, a byte-order mark) is the caller's to set,
/// for instance from a corpus's `[parse]` table (`Bundle::parse_options`).
pub fn default_options() -> ParseOptions {
    ParseOptions {
        frontmatter: Frontmatter::Yaml,
        ..ParseOptions::default()
    }
}

/// Parse markdown to AST, with [`default_options`]
pub fn parse_markdown(content: &str) -> Result<Node> {
    parse_markdown_with(content, &default_options())
}

/// Parse markdown to AST with the given options
pub fn parse_markdown_with(content: &str, options: &ParseOptions) -> Result<Node> {
    markdoc::ext::parse(content, options)
}

/// Transform an AST into the value markdoc's HTML renderer takes (upstream's
/// `Markdoc.transform`), with the document's frontmatter bound as `$markdoc.frontmatter`.
/// A walker that wants a tree calls `value.to_render()`.
pub fn transform_ast(node: &Node, config: &Config) -> Result<Value> {
    if !node.attributes.contains_key("frontmatter") {
        return markdoc::transform(node, config);
    }
    let mut config = config.clone();
    frontmatter_variables(node).apply(&mut config);
    markdoc::transform(node, &config)
}

/// Render to HTML string (upstream's `Markdoc.renderers.html`)
pub fn render_to_html(value: &Value) -> Result<String> {
    html::render(value)
}

/// What both components show: `content` parsed, transformed and rendered, or the first
/// failure as a `<div class="markdoc-error">`.
fn render_page(content: &str, options: &ParseOptions, config: &Config) -> String {
    let node = match parse_markdown_with(content, options) {
        Ok(node) => node,
        Err(e) => return error_div("Parse error", &e.to_string()),
    };
    // Frontmatter that is no YAML mapping does not fail the parse: the document keeps the raw
    // text and carries a `frontmatter-invalid` error. Shown as the parse error it used to be,
    // rather than rendered with nothing bound.
    if let Some(e) = node.errors.iter().find(|e| e.id == "frontmatter-invalid") {
        return error_div("Parse error", &e.message);
    }
    let value = match transform_ast(&node, config) {
        Ok(value) => value,
        Err(e) => return error_div("Transform error", &e.to_string()),
    };
    match render_to_html(&value) {
        Ok(html) => html,
        Err(e) => error_div("Render error", &e.to_string()),
    }
}

fn error_div(stage: &str, message: &str) -> String {
    leptos::logging::error!("Failed to render markdown: {stage}: {message}");
    format!(
        "<div class=\"markdoc-error\">{stage}: {}</div>",
        html::escape_html(message)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_markdown() {
        let content = "# Hello World\n\nThis is a test.";
        let result = parse_markdown(content);
        assert!(result.is_ok());
    }

    fn html_for(source: &str, config: &Config) -> String {
        let node = parse_markdown(source).expect("parses");
        render_to_html(&transform_ast(&node, config).expect("transforms")).expect("renders")
    }

    /// This crate renders through markdoc's generic HTML renderer, which
    /// emits a tag under its schema's `render` name. So an unresolved alias
    /// would reach the page as `<notice>` while the stylesheet expects
    /// `<callout>` — the same disagreement that put 2 808 unstyled blocks
    /// into the PDFs, one layer down.
    ///
    /// markdoc's bundle gives an alias its canonical tag's `render`, so this
    /// crate needs no code of its own for it. These pin that it stays true.
    #[test]
    fn an_aliased_tag_renders_under_its_canonical_name() {
        let html = html_for(
            "{% notice type=\"warning\" %}\nBody.\n{% /notice %}\n",
            &default_config(),
        );
        assert!(html.contains("<callout"), "expected <callout>: {html}");
        assert!(
            !html.contains("<notice"),
            "alias leaked to the page: {html}"
        );
    }

    #[test]
    fn the_schema_supplies_attribute_defaults() {
        // Only declared tags get defaults. `{% notice %}` used to be in no
        // schema at all, so it got none.
        let html = html_for("{% notice %}\nBody.\n{% /notice %}\n", &default_config());
        assert!(html.contains("type=\"note\""), "no default applied: {html}");
    }

    #[test]
    fn a_corpus_overlay_reaches_the_rendered_page() {
        // `Config` is a prop on both components, which is how a corpus with
        // tags of its own passes them in. Without the overlay the tag is
        // undeclared: it renders only its children, with no defaults and no
        // validation, which is the quiet failure this whole arrangement
        // exists to make loud.
        let overlay = r#"
[tags.advisory]
description = "Alias of callout"
alias_of = "callout"
"#;
        let mut bundle = Bundle::builtin();
        assert!(bundle.config().tags.get("advisory").is_none());
        bundle.overlay(overlay).expect("overlay merges");
        let config = bundle.config();
        assert!(config.tags.get("advisory").is_some_and(Option::is_some));

        let html = html_for("{% advisory %}\nBody.\n{% /advisory %}\n", &config);
        assert!(
            html.contains("<callout"),
            "overlay alias not resolved: {html}"
        );
        assert!(
            html.contains("type=\"note\""),
            "overlay alias lost defaults: {html}"
        );
    }

    #[test]
    fn without_the_overlay_the_same_tag_renders_only_its_children() {
        // The contrast that makes the test above mean something. Upstream
        // renders a tag that has no schema as its children alone; the old
        // pipeline kept it under its own name (`<advisory>`). Nothing passes
        // undeclared tags through (MIGRATION-MAP D13).
        let html = html_for(
            "{% advisory %}\nBody.\n{% /advisory %}\n",
            &default_config(),
        );
        assert_eq!(html, "<article><p>Body.</p></article>");
    }

    #[test]
    fn the_frontmatter_is_bound_as_markdoc_frontmatter() {
        // Upstream binds no variables of its own; `transform_ast` binds the
        // document's frontmatter, as the old pipeline's transform did.
        let source = "---\ntitle: Hello\n---\n# {% $markdoc.frontmatter.title %}\n";
        assert_eq!(
            html_for(source, &default_config()),
            "<article><h1>Hello</h1></article>"
        );

        // Options are the caller's: with upstream's, the frontmatter stays
        // raw text, and `$markdoc.frontmatter.title` is undefined.
        let node = parse_markdown_with(source, &ParseOptions::default()).expect("parses");
        let value = transform_ast(&node, &default_config()).expect("transforms");
        assert_eq!(
            render_to_html(&value).expect("renders"),
            "<article><h1></h1></article>"
        );
    }

    #[test]
    fn frontmatter_that_is_no_mapping_shows_as_a_parse_error() {
        let page = render_page(
            "---\njust text\n---\n# Title\n",
            &default_options(),
            &default_config(),
        );
        assert!(
            page.starts_with("<div class=\"markdoc-error\">Parse error: "),
            "{page}"
        );
        assert!(!page.contains("<h1>"), "{page}");
    }

    #[test]
    fn an_error_message_is_escaped() {
        assert_eq!(
            error_div("Parse error", "<b>&</b>"),
            "<div class=\"markdoc-error\">Parse error: &lt;b&gt;&amp;&lt;/b&gt;</div>"
        );
    }
}
