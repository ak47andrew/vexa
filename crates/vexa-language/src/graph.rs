#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Original look: square tables, left-aligned, horizontal, orthogonal edges.
    Boxy,
    /// Rounded boxes, centered text, vertical layout, curved edges.
    Rounded,
}

pub struct Node {
    name: String,
    code: String,
    id: String,
}

pub struct Graph {
    nodes: Vec<Node>,
    current_id: usize,
    connection: Vec<(String, String)>,
    style: Style,
}

impl Graph {
    /// Default is the rounded look. Use `with_style(Style::Boxy)` for the old one.
    pub fn new() -> Self {
        Self::with_style(Style::Rounded)
    }

    pub fn with_style(style: Style) -> Self {
        Self { nodes: Vec::new(), current_id: 0, connection: Vec::new(), style }
    }

    pub fn add_node(&mut self, name: &str, code: &str) -> String {
        let node = Node { name: name.to_string(), code: code.to_string(), id: Graph::node_id(self.current_id) };
        self.nodes.push(node);
        self.current_id += 1;
        self.nodes.last().unwrap().id.clone()
    }

    pub fn build(&self) -> String {
        let mut out = String::from(match self.style {
            Style::Boxy => {
                "digraph {\n\trankdir=LR\n\tsplines=ortho\n\tnode [shape=none margin=0 fontname=monospace]\n\n"
            }
            Style::Rounded => {
                "digraph {\n\trankdir=TB\n\tsplines=curved\n\tordering=out\n\tnodesep=0.4\n\tranksep=0.6\n\
                 \tnode [shape=box style=\"rounded,filled\" fillcolor=\"#2d2d2d\" color=\"#4a90d9\" penwidth=1.5 margin=\"0.15,0.1\" fontname=monospace]\n\
                 \tedge [color=\"#9aa0a6\" arrowhead=vee arrowsize=0.8 penwidth=1.2]\n\n"
            }
        });

        for node in &self.nodes {
            out.push_str(self.format_node(node).as_str());
        }

        for (from, to) in &self.connection {
            out.push_str(format!("\t{} -> {}\n", from, to).as_str());
        }

        out.push('}');
        out
    }

    pub fn connect_nodes(&mut self, a: &String, b: &String) {
        if !self.connection.contains(&(a.clone(), b.clone())) {
            self.connection.push((a.clone(), b.clone()));
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    // ------

    fn format_node(&self, node: &Node) -> String {
        match self.style {
            Style::Boxy => format!(
                "\t{} [label=<\n\
                 \t\t<table border=\"1\" cellborder=\"0\" cellspacing=\"0\" bgcolor=\"#2d2d2d\">\n\
                 \t\t\t<tr><td bgcolor=\"#4a90d9\" align=\"left\"><font color=\"white\"><b>{}</b></font></td></tr>\n\
                 \t\t\t<tr><td align=\"left\"><font color=\"white\">{}<br ALIGN=\"LEFT\"/></font></td></tr>\n\
                 \t\t</table>\n\t>]\n\n",
                node.id,
                html_text(&node.name, " "),
                html_text(&node.code, "<br ALIGN=\"LEFT\"/>"),
            ),
            Style::Rounded => {
                let mut s = format!(
                    "\t{} [label=<\n\
                     \t\t<table border=\"0\" cellborder=\"0\" cellspacing=\"0\" cellpadding=\"3\">\n\
                     \t\t\t<tr><td><font color=\"#8ab4f8\"><b>{}</b></font></td></tr>\n",
                    node.id,
                    html_text(&node.name, " "),
                );
                // An empty body would produce an empty <font>, so skip the separator and row.
                if !node.code.trim().is_empty() {
                    s.push_str(&format!(
                        "\t\t\t<hr/>\n\t\t\t<tr><td><font color=\"white\">{}</font></td></tr>\n",
                        html_text(&node.code, "<br/>"),
                    ));
                }
                s.push_str("\t\t</table>\n\t>]\n\n");
                s
            }
        }
    }

    pub fn node_id(n: usize) -> String {
        let mut n = n;
        let mut result = String::new();
        loop {
            result.push((b'A' + (n % 26) as u8) as char);
            n /= 26;
            if n == 0 { break; }
            n -= 1; // offset because there's no "zero" digit
        }
        result.chars().rev().collect()
    }

    pub fn id_to_node(s: &str) -> usize {
        s.chars().fold(0, |acc, c| {
            acc * 26 + (c as usize - 'A' as usize + 1)
        }) - 1
    }
}

/// Escapes text for Graphviz HTML-like labels and swaps newlines for `line_break`.
/// `<`, `>` and `&` are all escaped now, so operators like `<` are safe in labels.
fn html_text(text: &str, line_break: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\n', line_break)
        .replace('\t', "")
        .replace("  ", "")
        .replace('[', "&#91;")
        .replace(']', "&#93;")
}