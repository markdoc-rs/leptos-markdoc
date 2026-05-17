// Example of using Markdoc with Leptos
// To run this example, you'll need to set up a Leptos project
// This is just a code sample showing the API

use leptos::prelude::*;
use leptos_markdoc::{Markdoc, MarkdocReactive};

#[component]
fn App() -> impl IntoView {
    let markdown = r#"
# Welcome to Markdoc + Leptos

This is a **reactive** markdown renderer built with:
- Markdoc (Rust port)
- Leptos framework

## Live Preview

Edit the content and see it update in real-time!
"#;

    view! {
        <div class="container">
            <h1>"Markdoc + Leptos Example"</h1>
            <Markdoc content=markdown.to_string() />
        </div>
    }
}

#[component]
fn ReactiveExample() -> impl IntoView {
    let (content, set_content) = signal(String::from("# Edit me!\n\nType some **markdown**..."));

    view! {
        <div class="editor-container">
            <div class="editor">
                <h2>"Editor"</h2>
                <textarea
                    on:input=move |ev| {
                        set_content.set(event_target_value(&ev));
                    }
                    prop:value=content
                />
            </div>
            <div class="preview">
                <h2>"Preview"</h2>
                <MarkdocReactive content=content />
            </div>
        </div>
    }
}

// Note: To actually run this, you would need:
// 1. Create a Leptos project with `cargo leptos new`
// 2. Add markdoc-leptos to dependencies
// 3. Set up the proper Leptos boilerplate
fn main() {
    println!("See the Leptos documentation for setting up a full application");
}
