//! Markdown for chat messages, producing the same element structure as `ChatMarkdown.tsx`
//! (`p dir=ltr`, `ul dir=ltr`, inline `code[data-inline-code]`, punctuation after inline code kept
//! in a no-break span). Block code with syntax highlighting is still to be ported.
use dioxus::prelude::*;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Text(String),
    Element { name: &'static str, attrs: Vec<(&'static str, String)>, children: Vec<Node> },
}

fn element(name: &'static str, children: Vec<Node>) -> Node {
    Node::Element { name, attrs: vec![], children }
}

fn block(name: &'static str, children: Vec<Node>) -> Node {
    Node::Element { name, attrs: vec![("dir", "ltr".into())], children }
}

/// Leading punctuation that stays attached to inline code so a line never breaks between them.
fn leading_punctuation(text: &str) -> usize {
    text.chars().take_while(|c| matches!(c, ',' | '.' | ';' | ':' | '!' | '?' | ')' | ']' | '}')).map(char::len_utf8).sum()
}

pub fn parse(text: &str) -> Vec<Node> {
    let mut stack: Vec<(Tag<'_>, Vec<Node>)> = Vec::new();
    let mut root: Vec<Node> = Vec::new();
    for event in Parser::new_ext(text, Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS) {
        let target = |stack: &mut Vec<(Tag<'_>, Vec<Node>)>, root: &mut Vec<Node>| -> *mut Vec<Node> {
            match stack.last_mut() {
                Some((_, children)) => children as *mut _,
                None => root as *mut _,
            }
        };
        match event {
            Event::Start(tag) => stack.push((tag, Vec::new())),
            Event::End(end) => {
                let Some((tag, children)) = stack.pop() else { continue };
                let node = match (tag, end) {
                    (Tag::Paragraph, TagEnd::Paragraph) => block("p", children),
                    (Tag::List(None), _) => block("ul", children),
                    (Tag::List(Some(_)), _) => block("ol", children),
                    (Tag::Item, _) => element("li", children),
                    (Tag::Strong, _) => element("strong", children),
                    (Tag::Emphasis, _) => element("em", children),
                    (Tag::Strikethrough, _) => element("del", children),
                    (Tag::BlockQuote(_), _) => element("blockquote", children),
                    (Tag::Link { dest_url, .. }, _) => Node::Element { name: "a", attrs: vec![("href", dest_url.to_string())], children },
                    (Tag::Heading { level, .. }, _) => block(
                        match level {
                            pulldown_cmark::HeadingLevel::H1 => "h1",
                            pulldown_cmark::HeadingLevel::H2 => "h2",
                            pulldown_cmark::HeadingLevel::H3 => "h3",
                            pulldown_cmark::HeadingLevel::H4 => "h4",
                            pulldown_cmark::HeadingLevel::H5 => "h5",
                            _ => "h6",
                        },
                        children,
                    ),
                    (Tag::Table(_), _) => block("table", children),
                    (Tag::TableHead, _) => element("tr", children),
                    (Tag::TableRow, _) => element("tr", children),
                    (Tag::TableCell, _) => {
                        let in_head = matches!(stack.last(), Some((Tag::TableHead, _)));
                        element(if in_head { "th" } else { "td" }, children)
                    }
                    (Tag::CodeBlock(_), _) => Node::Element { name: "pre", attrs: vec![], children: vec![element("code", children)] },
                    (_, _) => element("span", children),
                };
                // SAFETY-free re-borrow: push into the new top.
                match stack.last_mut() {
                    Some((_, parent)) => parent.push(node),
                    None => root.push(node),
                }
            }
            Event::Text(t) => {
                let out = unsafe { &mut *target(&mut stack, &mut root) };
                // Punctuation right after inline code joins the code in a no-break span.
                if let Some(Node::Element { name: "code", attrs, .. }) = out.last() {
                    let n = leading_punctuation(&t);
                    if n > 0 && attrs.iter().any(|(k, _)| *k == "data-inline-code") {
                        let code = out.pop().unwrap();
                        out.push(Node::Element { name: "span", attrs: vec![("data-nobreak", String::new())], children: vec![code, Node::Text(t[..n].to_string())] });
                        if n < t.len() {
                            out.push(Node::Text(t[n..].to_string()));
                        }
                        continue;
                    }
                }
                out.push(Node::Text(t.to_string()));
            }
            Event::Code(code) => {
                let out = unsafe { &mut *target(&mut stack, &mut root) };
                let inline = Node::Element { name: "code", attrs: vec![("data-inline-code", String::new()), ("dir", "ltr".into())], children: vec![Node::Text(code.to_string())] };
                out.push(inline);
            }
            Event::SoftBreak => {
                let out = unsafe { &mut *target(&mut stack, &mut root) };
                out.push(Node::Text("\n".into()));
            }
            Event::Rule => {
                let out = unsafe { &mut *target(&mut stack, &mut root) };
                out.push(element("hr", vec![]));
            }
            Event::TaskListMarker(done) => {
                let out = unsafe { &mut *target(&mut stack, &mut root) };
                out.push(Node::Text(if done { "☑ " } else { "☐ " }.into()));
            }
            Event::HardBreak => {
                let out = unsafe { &mut *target(&mut stack, &mut root) };
                out.push(element("br", vec![]));
            }
            _ => {}
        }
    }
    root
}

fn view(node: &Node) -> Element {
    let attr = |attrs: &Vec<(&'static str, String)>, key: &str| attrs.iter().find(|(k, _)| *k == key).map(|(_, v)| v.clone());
    match node {
        Node::Text(t) => rsx! { "{t}" },
        Node::Element { name, attrs, children } => {
            let kids = children.iter().map(view);
            let dir = attr(attrs, "dir");
            match *name {
                "p" => rsx! { p { dir, {kids} } },
                "ul" => rsx! { ul { dir, {kids} } },
                "ol" => rsx! { ol { dir, {kids} } },
                "li" => rsx! { li { {kids} } },
                "strong" => rsx! { strong { {kids} } },
                "em" => rsx! { em { {kids} } },
                "del" => rsx! { del { {kids} } },
                "blockquote" => rsx! { blockquote { {kids} } },
                "pre" => rsx! { pre { {kids} } },
                "br" => rsx! { br {} },
                "hr" => rsx! { hr {} },
                "table" => rsx! { div { class: "overflow-x-auto", table { {kids} } } },
                "tr" => rsx! { tr { {kids} } },
                "th" => rsx! { th { {kids} } },
                "td" => rsx! { td { {kids} } },
                "a" => {
                    let href = attr(attrs, "href");
                    rsx! { a { href, {kids} } }
                }
                "h1" => rsx! { h1 { dir, {kids} } },
                "h2" => rsx! { h2 { dir, {kids} } },
                "h3" => rsx! { h3 { dir, {kids} } },
                "h4" => rsx! { h4 { dir, {kids} } },
                "h5" => rsx! { h5 { dir, {kids} } },
                "h6" => rsx! { h6 { dir, {kids} } },
                // Inline code: the no-break span exists only to keep trailing punctuation attached.
                "span" if attr(attrs, "data-nobreak").is_some() => {
                    rsx! { span { class: "has-[[data-slot=context-chip-label]]:whitespace-nowrap", {kids} } }
                }
                "code" if attr(attrs, "data-inline-code").is_some() => {
                    rsx! { code { "data-inline-code": "", dir: "ltr", {kids} } }
                }
                "code" => rsx! { code { {kids} } },
                _ => rsx! { span { {kids} } },
            }
        }
    }
}

/// Renders message markdown into the children of a `.chat-markdown` element.
pub fn render(text: &str) -> Element {
    let nodes = parse(text);
    rsx! { for node in nodes.iter() { {view(node)} } }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(nodes: &[Node]) -> String {
        nodes
            .iter()
            .map(|n| match n {
                Node::Text(t) => t.clone(),
                Node::Element { name, children, .. } => format!("<{name}>{}</{name}>", flat(children)),
            })
            .collect()
    }

    #[test]
    fn tables_and_rules() {
        let nodes = parse("| a | b |\n|---|---|\n| 1 | 2 |\n\n---\n");
        assert_eq!(flat(&nodes), "<table><tr><th>a</th><th>b</th></tr><tr><td>1</td><td>2</td></tr></table><hr></hr>");
    }

    #[test]
    fn paragraphs_lists_and_inline_code() {
        let nodes = parse("Answer 0.\n\n- checked `a`\n- ran\n\nThe width is `x`, so.");
        assert_eq!(
            flat(&nodes),
            "<p>Answer 0.</p><ul><li>checked <code>a</code></li><li>ran</li></ul><p>The width is <span><code>x</code>,</span> so.</p>"
        );
    }
}
