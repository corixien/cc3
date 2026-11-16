//! Serialization of nodes to HTML, and of inline `<svg>` subtrees to SVG source for usvg.

use html_escape::{encode_quoted_attribute_to_string, encode_text_to_string};
use markup5ever::{QualName, ns};

use super::{Node, NodeData};

#[derive(Clone, Copy)]
enum OutputStyle {
    Normal,
    Pretty,
}

#[derive(Clone, Copy)]
struct SerializeOpts {
    style: OutputStyle,
    /// Replace `currentColor` in attribute values with the element's computed `color`
    resolve_current_color: bool,
    /// Declare the `xlink` prefix on the root element so that the output is well-formed XML
    declare_xlink_ns: bool,
}

impl SerializeOpts {
    const fn html(style: OutputStyle) -> Self {
        Self {
            style,
            resolve_current_color: false,
            declare_xlink_ns: false,
        }
    }
}

const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

/// Writes an attribute's serialized name, per the HTML fragment serialization algorithm
fn write_attr_name(name: &QualName, writer: &mut String) {
    match name.ns {
        ns!() | ns!(html) => {}
        ns!(xml) => writer.push_str("xml:"),
        ns!(xmlns) if name.local.as_str() == "xmlns" => {}
        ns!(xmlns) => writer.push_str("xmlns:"),
        ns!(xlink) => writer.push_str("xlink:"),
        _ => {
            if let Some(prefix) = &name.prefix {
                writer.push_str(prefix);
                writer.push(':');
            }
        }
    }
    writer.push_str(&name.local);
}

fn is_xlink_ns_decl(name: &QualName) -> bool {
    (name.ns == ns!(xmlns) && name.local.as_str() == "xlink")
        || name.local.as_str() == "xmlns:xlink"
}

impl Node {
    /// Renders the HTML of this node and all its children as a `String` without extra whitespace.
    ///
    /// Example output:
    ///
    /// ```text
    /// <html><head /><body><main id="main"><div class="arbitrary-class" /></main></body></html>
    /// ```
    pub fn outer_html(&self) -> String {
        let mut output = String::new();
        self.write_outer_html(&mut output);
        output
    }

    /// Renders the HTML of this node and all its children as a `String` with whitespace for human
    /// readability.
    ///
    /// Example output:
    ///
    /// ```text
    /// <html>
    ///   <head />
    ///   <body>
    ///     <main id="main">
    ///       <div class="arbitrary-class" />
    ///     </main>
    ///   </body>
    /// </html>
    /// ```
    pub fn outer_html_pretty(&self) -> String {
        let mut output = String::new();
        self.write_outer_html_pretty(&mut output);
        output
    }

    pub fn write_outer_html(&self, writer: &mut String) {
        self.write_outer_html_in_style(writer, SerializeOpts::html(OutputStyle::Normal), 0);
    }

    pub fn write_outer_html_pretty(&self, writer: &mut String) {
        self.write_outer_html_in_style(writer, SerializeOpts::html(OutputStyle::Pretty), 0);
    }

    /// Serializes this `<svg>` subtree as a standalone SVG document for usvg, with
    /// `currentColor` resolved against each element's computed `color`.
    #[cfg(feature = "svg")]
    pub(crate) fn svg_source(&self) -> String {
        let mut output = String::new();
        let opts = SerializeOpts {
            style: OutputStyle::Normal,
            resolve_current_color: true,
            declare_xlink_ns: true,
        };
        self.write_outer_html_in_style(&mut output, opts, 0);
        if std::env::var("CC3_DEBUG_SVG").is_ok_and(|v| output.contains(&v)) {
            eprintln!("SVG SOURCE: {output}");
        }
        output
    }

    fn write_outer_html_in_style(&self, writer: &mut String, opts: SerializeOpts, nesting: usize) {
        const INDENT: &str = "  ";
        let style = opts.style;
        let has_children = !self.children.is_empty();
        let current_color = opts
            .resolve_current_color
            .then(|| self.primary_styles())
            .flatten()
            .map(|style| style.clone_color())
            // CC3: usvg only parses legacy colors; computed oklch()/color-mix() values must be
            // converted to sRGB first, otherwise the whole icon is dropped.
            .map(|color| {
                let c = color.into_srgb_legacy();
                let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                format!(
                    "rgba({},{},{},{})",
                    byte(c.components.0),
                    byte(c.components.1),
                    byte(c.components.2),
                    c.alpha
                )
            });

        match &self.data {
            NodeData::Document(_) => {}
            NodeData::Comment { .. } => {}
            NodeData::AnonymousBlock(_) => {}
            // NodeData::Doctype { name, .. } => write!(s, "DOCTYPE {name}"),
            NodeData::Text(data) => {
                if matches!(style, OutputStyle::Pretty) {
                    for _ in 0..nesting {
                        writer.push_str(INDENT);
                    }
                }
                let in_raw_text_element = self
                    .parent
                    .and_then(|id| self.tree()[id].data.downcast_element())
                    .is_some_and(|el| {
                        // Documents are parsed with scripting disabled, so `<noscript>` is not raw text
                        el.name.ns == ns!(html)
                            && matches!(
                                el.name.local.as_ref(),
                                "style"
                                    | "script"
                                    | "xmp"
                                    | "iframe"
                                    | "noembed"
                                    | "noframes"
                                    | "plaintext"
                            )
                    });
                if in_raw_text_element {
                    writer.push_str(data.content.as_str());
                } else {
                    encode_text_to_string(data.content.as_str(), writer);
                }
                if matches!(style, OutputStyle::Pretty) {
                    writer.push('\n');
                }
            }
            NodeData::Element(data) => {
                if matches!(style, OutputStyle::Pretty) {
                    for _ in 0..nesting {
                        writer.push_str(INDENT);
                    }
                }
                writer.push('<');
                writer.push_str(&data.name.local);

                if opts.declare_xlink_ns
                    && nesting == 0
                    && !data.attrs().iter().any(|attr| is_xlink_ns_decl(&attr.name))
                {
                    writer.push_str(" xmlns:xlink=\"");
                    writer.push_str(XLINK_NS);
                    writer.push('"');
                }

                // CC3: inside <svg> the page CSS (classes such as `fill-current`, `var()` stop
                // colors) is not visible to usvg, so computed paint values are written as a
                // `style` attribute after the original one.
                let svg_style = (opts.resolve_current_color && data.name.ns == ns!(svg))
                    .then(|| self.computed_svg_declarations(data))
                    .flatten();
                for attr in data.attrs() {
                    if svg_style.is_some() && &*attr.name.local == "style" {
                        continue;
                    }
                    writer.push(' ');
                    write_attr_name(&attr.name, writer);
                    writer.push_str("=\"");
                    #[allow(clippy::unnecessary_unwrap)] // Convert to if-let chain once stabilised
                    if current_color.is_some() && attr.value.contains("currentColor") {
                        let value = attr
                            .value
                            .replace("currentColor", current_color.as_ref().unwrap());
                        encode_quoted_attribute_to_string(&value, writer);
                    } else {
                        encode_quoted_attribute_to_string(&attr.value, writer);
                    }
                    writer.push('"');
                }
                if let Some(computed) = svg_style {
                    let original = data
                        .attrs()
                        .iter()
                        .find(|attr| &*attr.name.local == "style")
                        .map(|attr| {
                            // Declarations with var() are unusable for usvg; computed values follow.
                            attr.value
                                .split(';')
                                .filter(|decl| !decl.contains("var("))
                                .collect::<Vec<_>>()
                                .join(";")
                        })
                        .unwrap_or_default();
                    writer.push_str(" style=\"");
                    let original = original.trim_matches(|c: char| c == ';' || c.is_whitespace());
                    let merged = if original.is_empty() {
                        computed
                    } else {
                        format!("{original};{computed}")
                    };
                    encode_quoted_attribute_to_string(&merged, writer);
                    writer.push('"');
                }
                if !has_children {
                    writer.push_str(" /");
                }
                writer.push('>');
                if matches!(style, OutputStyle::Pretty) {
                    writer.push('\n');
                }

                if has_children {
                    for &child_id in &self.children {
                        self.tree()[child_id].write_outer_html_in_style(writer, opts, nesting + 1);
                    }

                    if matches!(style, OutputStyle::Pretty) {
                        for _ in 0..nesting {
                            writer.push_str(INDENT);
                        }
                    }
                    writer.push_str("</");
                    writer.push_str(&data.name.local);
                    writer.push('>');
                    if matches!(style, OutputStyle::Pretty) {
                        writer.push('\n');
                    }
                }
            }
        }
    }
}

fn css_rgba(color: &style::color::AbsoluteColor) -> String {
    let c = color.into_srgb_legacy();
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("rgba({},{},{},{})", byte(c.components.0), byte(c.components.1), byte(c.components.2), c.alpha)
}

impl Node {
    /// Computed SVG paint declarations (`fill:..;stroke:..`) that differ from the initial values
    /// and are not already given as an attribute on this element (CC3 patch).
    fn computed_svg_declarations(&self, data: &crate::node::ElementData) -> Option<String> {
        use style::values::generics::svg::SVGPaintKind;
        let styles = self.primary_styles()?;
        let current = styles.clone_color();
        let has_attr = |name: &str| data.attrs().iter().any(|a| &*a.name.local == name);
        let paint = |paint: &style::values::computed::SVGPaint| -> String {
            match &paint.kind {
                SVGPaintKind::None => "none".into(),
                SVGPaintKind::Color(c) => css_rgba(&c.resolve_to_absolute(&current)),
                SVGPaintKind::PaintServer(url) => {
                    use style_traits::ToCss;
                    format!("url({})", url.to_css_string().trim_start_matches("url(").trim_end_matches(')'))
                }
                SVGPaintKind::ContextFill => "context-fill".into(),
                SVGPaintKind::ContextStroke => "context-stroke".into(),
            }
        };
        let mut out = Vec::new();
        let svg = styles.get_inherited_svg();
        let fill = paint(&svg.fill);
        if !has_attr("fill") && fill != "rgba(0,0,0,1)" {
            out.push(format!("fill:{fill}"));
        }
        let stroke = paint(&svg.stroke);
        if !has_attr("stroke") && stroke != "none" {
            out.push(format!("stroke:{stroke}"));
        }
        if &*data.name.local == "stop" {
            let stop = styles.get_svg();
            if !has_attr("stop-color") {
                out.push(format!("stop-color:{}", css_rgba(&stop.stop_color.resolve_to_absolute(&current))));
            }
            if !has_attr("stop-opacity") {
                out.push(format!("stop-opacity:{}", stop.stop_opacity));
            }
        }
        (!out.is_empty()).then(|| out.join(";"))
    }
}
