//! Tests for declarative Shadow DOM parsing implementation

use boa_engine::{
    builtins::declarative_shadow_dom::{DeclarativeShadowDOMParser, DeclarativeShadowDOMHTMLProcessor},
    Context,
};

#[test]
fn test_shadow_root_mode_extraction() {
    // Test open mode
    let open_template = r#"<template shadowrootmode="open">content</template>"#;
    let templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(open_template);
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].shadow_mode, crate::builtins::shadow_root::ShadowRootMode::Open);

    // Test closed mode
    let closed_template = r#"<template shadowrootmode="closed">content</template>"#;
    let templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(closed_template);
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].shadow_mode, crate::builtins::shadow_root::ShadowRootMode::Closed);

    // Test with single quotes
    let single_quote_template = r#"<template shadowrootmode='open'>content</template>"#;
    let templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(single_quote_template);
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].shadow_mode, crate::builtins::shadow_root::ShadowRootMode::Open);

    // Test without quotes
    let no_quote_template = r#"<template shadowrootmode=open>content</template>"#;
    let templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(no_quote_template);
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].shadow_mode, crate::builtins::shadow_root::ShadowRootMode::Open);

    // Test with extra whitespace
    let whitespace_template = r#"<template   shadowrootmode = "open"  >content</template>"#;
    let templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(whitespace_template);
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].shadow_mode, crate::builtins::shadow_root::ShadowRootMode::Open);
}

#[test]
fn test_declarative_shadow_template_parsing() {
    let html = r#"
        <div>
            <template shadowrootmode="open">
                <style>
                    :host { display: block; }
                    .content { color: red; }
                </style>
                <div class="content">Shadow content</div>
                <slot></slot>
            </template>
            <p>Light DOM content</p>
        </div>
    "#;

    let templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(html);

    assert_eq!(templates.len(), 1);
    let template = &templates[0];

    assert_eq!(template.shadow_mode, crate::builtins::shadow_root::ShadowRootMode::Open);
    assert!(template.content.contains(":host { display: block; }"));
    assert!(template.content.contains("Shadow content"));
    assert!(template.content.contains("<slot></slot>"));
}

#[test]
fn test_multiple_declarative_shadow_roots() {
    let html = r#"
        <custom-element>
            <template shadowrootmode="open">
                <div>First shadow root</div>
            </template>
        </custom-element>

        <another-element>
            <template shadowrootmode="closed">
                <div>Second shadow root</div>
            </template>
        </another-element>
    "#;

    let templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(html);

    assert_eq!(templates.len(), 2);

    assert_eq!(templates[0].shadow_mode, crate::builtins::shadow_root::ShadowRootMode::Open);
    assert!(templates[0].content.contains("First shadow root"));

    assert_eq!(templates[1].shadow_mode, crate::builtins::shadow_root::ShadowRootMode::Closed);
    assert!(templates[1].content.contains("Second shadow root"));
}

#[test]
fn test_shadow_root_attributes_parsing() {
    let html = r#"
        <template shadowrootmode="open" shadowrootclonable shadowrootserializable>
            <div>Content</div>
        </template>
    "#;

    let templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(html);
    assert_eq!(templates.len(), 1);

    let template = &templates[0];
    assert!(template.attributes.contains_key("shadowrootclonable"));
    assert!(template.attributes.contains_key("shadowrootserializable"));
    assert_eq!(template.attributes.get("shadowrootclonable"), Some(&"true".to_string()));
}

#[test]
fn test_shadow_root_with_delegates_focus() {
    let html = r#"
        <template shadowrootmode="open" shadowrootdelegatesfocus="true">
            <input type="text" />
        </template>
    "#;

    let templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(html);
    assert_eq!(templates.len(), 1);

    let template = &templates[0];
    assert!(template.attributes.contains_key("shadowrootdelegatesfocus"));
    assert_eq!(template.attributes.get("shadowrootdelegatesfocus"), Some(&"true".to_string()));
}

#[test]
fn test_nested_declarative_shadow_roots() {
    let html = r#"
        <outer-element>
            <template shadowrootmode="open">
                <div>Outer shadow</div>
                <inner-element>
                    <template shadowrootmode="closed">
                        <div>Inner shadow</div>
                    </template>
                </inner-element>
            </template>
        </outer-element>
    "#;

    let outer_templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(html);
    assert_eq!(outer_templates.len(), 1);

    // The outer template should contain the inner template in its content
    let outer_template = &outer_templates[0];
    assert!(outer_template.content.contains("Outer shadow"));
    assert!(outer_template.content.contains("shadowrootmode=\"closed\""));

    // Parse inner templates from the outer template content
    let inner_templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(&outer_template.content);
    assert_eq!(inner_templates.len(), 1);
    assert!(inner_templates[0].content.contains("Inner shadow"));
}

#[test]
fn test_declarative_shadow_dom_with_slots() {
    let html = r#"
        <template shadowrootmode="open">
            <style>
                ::slotted(p) { color: blue; }
            </style>
            <div class="wrapper">
                <slot name="header"></slot>
                <slot></slot>
                <slot name="footer"></slot>
            </div>
        </template>
    "#;

    let templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(html);
    assert_eq!(templates.len(), 1);

    let template = &templates[0];
    assert!(template.content.contains("::slotted(p)"));
    assert!(template.content.contains(r#"<slot name="header"></slot>"#));
    assert!(template.content.contains(r#"<slot name="footer"></slot>"#));
    assert!(template.content.contains(r#"<slot></slot>"#));
}

#[test]
fn test_html_processing_workflow() {
    let html = r#"
        <!DOCTYPE html>
        <html>
            <body>
                <custom-element>
                    <template shadowrootmode="open">
                        <style>
                            :host { display: block; padding: 16px; }
                            .content { color: var(--text-color, black); }
                        </style>
                        <div class="content">
                            <slot></slot>
                        </div>
                    </template>
                    <p>This will be slotted</p>
                </custom-element>
            </body>
        </html>
    "#;

    // Test HTML contains declarative shadow DOM
    assert!(DeclarativeShadowDOMHTMLProcessor::contains_declarative_shadow_dom(html));

    // Test shadow template extraction
    let templates = DeclarativeShadowDOMHTMLProcessor::extract_shadow_templates(html);
    assert_eq!(templates.len(), 1);
    assert!(templates[0].contains(":host { display: block"));

    // Test full document processing
    let mut context = Context::default();
    let processed_html = DeclarativeShadowDOMHTMLProcessor::process_document_html(html, &mut context);
    assert!(processed_html.is_ok());

    let result = processed_html.unwrap();
    // The template should be replaced with a comment
    assert!(result.contains("<!-- declarative shadow root processed -->"));
    assert!(!result.contains("shadowrootmode"));
}

#[test]
fn test_element_tag_parsing() {
    let html = r#"
        <div class="container" id="main">
            <p>Text content</p>
            <img src="image.jpg" alt="Image" />
            <slot name="content"></slot>
        </div>
    "#;

    let tags = crate::builtins::declarative_shadow_dom::DeclarativeShadowDOMParser::extract_element_tags(html);

    // Should find div, p, img, and slot tags
    assert_eq!(tags.len(), 4);

    // Check div tag
    assert_eq!(tags[0].tag_name, "div");
    assert_eq!(tags[0].attributes.get("class"), Some(&"container".to_string()));
    assert_eq!(tags[0].attributes.get("id"), Some(&"main".to_string()));
    assert!(!tags[0].is_self_closing);

    // Check img tag
    let img_tag = tags.iter().find(|t| t.tag_name == "img").unwrap();
    assert_eq!(img_tag.attributes.get("src"), Some(&"image.jpg".to_string()));
    assert_eq!(img_tag.attributes.get("alt"), Some(&"Image".to_string()));
    assert!(img_tag.is_self_closing);

    // Check slot tag
    let slot_tag = tags.iter().find(|t| t.tag_name == "slot").unwrap();
    assert_eq!(slot_tag.attributes.get("name"), Some(&"content".to_string()));
}

#[test]
fn test_attribute_value_extraction() {
    use crate::builtins::declarative_shadow_dom::DeclarativeShadowDOMParser;

    // Test double-quoted value
    let attr1 = r#"="test value""#;
    let value1 = DeclarativeShadowDOMParser::extract_attribute_value(attr1);
    assert_eq!(value1, Some("test value".to_string()));

    // Test single-quoted value
    let attr2 = r#"='another value'"#;
    let value2 = DeclarativeShadowDOMParser::extract_attribute_value(attr2);
    assert_eq!(value2, Some("another value".to_string()));

    // Test unquoted value
    let attr3 = r#"=unquoted "#;
    let value3 = DeclarativeShadowDOMParser::extract_attribute_value(attr3);
    assert_eq!(value3, Some("unquoted".to_string()));

    // Test with whitespace
    let attr4 = r#" = "spaced value" "#;
    let value4 = DeclarativeShadowDOMParser::extract_attribute_value(attr4);
    assert_eq!(value4, Some("spaced value".to_string()));
}

#[test]
fn test_invalid_declarative_shadow_templates() {
    // Template without shadowrootmode
    let html1 = r#"<template>content</template>"#;
    let templates1 = DeclarativeShadowDOMParser::find_declarative_shadow_templates(html1);
    assert_eq!(templates1.len(), 0);

    // Template with invalid shadowrootmode value
    let html2 = r#"<template shadowrootmode="invalid">content</template>"#;
    let templates2 = DeclarativeShadowDOMParser::find_declarative_shadow_templates(html2);
    assert_eq!(templates2.len(), 0);

    // Malformed template (no closing tag)
    let html3 = r#"<template shadowrootmode="open">content"#;
    let templates3 = DeclarativeShadowDOMParser::find_declarative_shadow_templates(html3);
    assert_eq!(templates3.len(), 0);

    // Empty shadowrootmode attribute
    let html4 = r#"<template shadowrootmode="">content</template>"#;
    let templates4 = DeclarativeShadowDOMParser::find_declarative_shadow_templates(html4);
    assert_eq!(templates4.len(), 0);
}

#[test]
fn test_complex_declarative_shadow_dom() {
    let html = r#"
        <web-component>
            <template shadowrootmode="open" shadowrootclonable shadowrootdelegatesfocus>
                <style>
                    :host {
                        display: block;
                        border: 1px solid #ccc;
                        padding: 16px;
                    }

                    :host(.active) {
                        border-color: blue;
                    }

                    :host-context(.dark-theme) {
                        background: #222;
                        color: white;
                    }

                    ::slotted(h1) {
                        color: red;
                        margin-top: 0;
                    }

                    .header {
                        background: #f0f0f0;
                        padding: 8px;
                    }

                    .content {
                        margin: 16px 0;
                    }
                </style>

                <div class="header">
                    <slot name="title">Default Title</slot>
                </div>

                <div class="content">
                    <slot>Default content</slot>
                </div>

                <footer>
                    <slot name="footer">
                        <p>Default footer</p>
                    </slot>
                </footer>
            </template>

            <h1 slot="title">Custom Title</h1>
            <p>This content goes in the default slot</p>
            <div slot="footer">Custom footer content</div>
        </web-component>
    "#;

    let templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(html);
    assert_eq!(templates.len(), 1);

    let template = &templates[0];

    // Verify shadow root mode and attributes
    assert_eq!(template.shadow_mode, crate::builtins::shadow_root::ShadowRootMode::Open);
    assert!(template.attributes.contains_key("shadowrootclonable"));
    assert!(template.attributes.contains_key("shadowrootdelegatesfocus"));

    // Verify CSS content
    assert!(template.content.contains(":host {"));
    assert!(template.content.contains(":host(.active)"));
    assert!(template.content.contains(":host-context(.dark-theme)"));
    assert!(template.content.contains("::slotted(h1)"));

    // Verify slot elements
    assert!(template.content.contains(r#"<slot name="title">"#));
    assert!(template.content.contains(r#"<slot name="footer">"#));
    assert!(template.content.contains(r#"<slot>Default content</slot>"#));
}

#[test]
fn test_declarative_shadow_dom_processing() {
    let mut context = Context::default();

    let html = r#"
        <template shadowrootmode="open">
            <div>Shadow content</div>
            <slot></slot>
        </template>
    "#;

    let result = DeclarativeShadowDOMParser::parse_and_process(html, &mut context);
    assert!(result.is_ok());

    let elements = result.unwrap();
    assert_eq!(elements.len(), 1);

    // The element should be a host element with an attached shadow root
    // In a full implementation, we would verify the shadow root attachment
}