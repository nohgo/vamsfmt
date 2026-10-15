use crate::{Error, Options};
use std::collections::HashSet;
use std::ops::Range;
use tree_sitter::Node;

#[derive(Clone)]
struct Atom {
    range: Range<usize>,
    kind: &'static str,
    parent: &'static str,
    indent: usize,
    compact: bool,
    opaque: bool,
    frozen_line: bool,
}

struct Group {
    range: Range<usize>,
}

struct Plan<'a> {
    source: &'a str,
    atoms: Vec<Atom>,
    hard: HashSet<usize>,
    aligned: HashSet<usize>,
    groups: Vec<Group>,
    directives: Vec<Range<usize>>,
}

fn children(node: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).collect()
}

fn directive(kind: &str) -> bool {
    matches!(
        kind,
        "include_compiler_directive"
            | "text_macro_definition"
            | "id_directive"
            | "zero_directive"
            | "timescale_compiler_directive"
            | "default_nettype_compiler_directive"
            | "unconnected_drive"
            | "line_compiler_directive"
            | "begin_keywords"
    )
}

fn atomic(kind: &str) -> bool {
    kind.ends_with("identifier")
        || matches!(
            kind,
            "string_literal"
                | "double_quoted_string"
                | "integral_number"
                | "real_number"
                | "comment"
                | "attribute_instance"
                | "text_macro_usage"
                | "simple_text_macro_usage"
                | "unary_operator"
        )
}

// Adding a production here opts in to whitespace formatting of its children.
// Anything else stays one exact source atom, including its internal whitespace.
fn transparent(kind: &str) -> bool {
    matches!(
        kind,
        "module_header"
            | "module_nonansi_header"
            | "module_ansi_header"
            | "module_or_generate_item"
            | "port_declaration"
            | "input_declaration"
            | "output_declaration"
            | "inout_declaration"
            | "ansi_port_declaration"
            | "list_of_ports"
            | "list_of_port_declarations"
            | "port"
            | "port_reference"
            | "port_direction"
            | "net_type"
            | "list_of_port_identifiers"
            | "list_of_variable_port_identifiers"
            | "net_declaration"
            | "net_decl_assignment"
            | "list_of_net_decl_assignments"
            | "data_declaration"
            | "data_type"
            | "integer_atom_type"
            | "integer_vector_type"
            | "non_integer_type"
            | "implicit_data_type1"
            | "data_type_or_implicit1"
            | "variable_decl_assignment"
            | "list_of_variable_decl_assignments"
            | "unpacked_dimension"
            | "packed_dimension"
            | "parameter_declaration"
            | "local_parameter_declaration"
            | "parameter_type"
            | "parameter_port_list"
            | "parameter_port_declaration"
            | "param_assignment"
            | "list_of_param_assignments"
            | "parameter_value_range"
            | "range_bound"
            | "aliasparam_declaration"
            | "branch_declaration"
            | "branch_terminal"
            | "port_branch"
            | "nature_reference"
            | "nature_attribute"
            | "nature_binding"
            | "discipline_domain_binding"
            | "nature_attribute_override"
            | "connect_insertion"
            | "connect_resolution"
            | "analog_statement"
            | "analog_function_statement"
            | "analog_procedural_assignment"
            | "contribution_statement"
            | "branch_access"
            | "analog_event_control"
            | "analog_event_expression"
            | "analog_step_event"
            | "tf_port_declaration"
            | "tf_item_declaration"
            | "tf_port_list"
            | "tf_port_item1"
            | "tf_port_direction"
            | "list_of_tf_variable_identifiers"
            | "block_item_declaration"
            | "statement"
            | "statement_item"
            | "statement_or_null"
            | "function_statement"
            | "function_statement_or_null"
            | "blocking_assignment"
            | "nonblocking_assignment"
            | "variable_assignment"
            | "operator_assignment"
            | "assignment_operator"
            | "continuous_assign"
            | "net_assignment"
            | "list_of_net_assignments"
            | "function_declaration"
            | "task_declaration"
            | "variable_lvalue"
            | "net_lvalue"
            | "nonrange_variable_lvalue"
            | "expression"
            | "constant_expression"
            | "conditional_expression"
            | "mintypmax_expression"
            | "constant_mintypmax_expression"
            | "param_expression"
            | "constant_param_expression"
            | "primary"
            | "constant_primary"
            | "primary_literal"
            | "function_call"
            | "system_tf_call"
            | "argument_list"
            | "function_subroutine_call"
            | "constant_range"
            | "indexed_range"
            | "constant_indexed_range"
            | "select1"
            | "constant_select1"
            | "bit_select1"
            | "constant_bit_select1"
            | "nonrange_select1"
            | "concatenation"
            | "constant_concatenation"
            | "multiple_concatenation"
            | "constant_multiple_concatenation"
            | "array_initializer"
            | "module_instantiation"
            | "hierarchical_instance"
            | "name_of_instance"
            | "parameter_value_assignment"
            | "list_of_parameter_assignments"
            | "ordered_parameter_assignment"
            | "_ordered_parameter_assignment"
            | "named_parameter_assignment"
            | "list_of_port_connections"
            | "named_port_connection"
            | "ordered_port_connection"
            | "genvar_declaration"
            | "list_of_genvar_identifiers"
            | "genvar_initialization"
            | "genvar_iteration"
            | "event_control"
            | "event_expression"
            | "edge_identifier"
            | "delay_control"
            | "delay_value"
            | "delay2"
            | "delay3"
            | "delay_or_event_control"
            | "always_keyword"
            | "case_item"
            | "analog_case_item"
            | "case_generate_item"
            | "case_expression"
            | "case_item_expression"
    )
}

fn list(kind: &str) -> bool {
    matches!(
        kind,
        "argument_list"
            | "list_of_ports"
            | "list_of_port_declarations"
            | "parameter_port_list"
            | "parameter_value_assignment"
            | "hierarchical_instance"
            | "branch_access"
            | "concatenation"
            | "constant_concatenation"
            | "array_initializer"
    )
}

fn body(kind: &str) -> bool {
    matches!(
        kind,
        "analog_statement"
            | "analog_function_statement"
            | "statement"
            | "statement_or_null"
            | "function_statement"
            | "function_statement_or_null"
            | "generate_block"
    )
}

fn unwrap(mut node: Node<'_>) -> Node<'_> {
    loop {
        let named: Vec<_> = children(node)
            .into_iter()
            .filter(|n| n.is_named() && n.kind() != "comment")
            .collect();
        if matches!(
            node.kind(),
            "analog_statement"
                | "analog_function_statement"
                | "statement"
                | "statement_or_null"
                | "statement_item"
                | "function_statement"
                | "function_statement_or_null"
        ) && named.len() == 1
        {
            node = named[0];
        } else {
            return node;
        }
    }
}

fn block(node: Node<'_>) -> bool {
    let node = unwrap(node);
    matches!(
        node.kind(),
        "analog_seq_block" | "analog_function_seq_block" | "seq_block" | "par_block"
    ) || (node.kind() == "generate_block" && children(node).iter().any(|c| c.kind() == "begin"))
}

impl<'a> Plan<'a> {
    fn new(source: &'a str, root: Node<'_>) -> Self {
        let mut plan = Self {
            source,
            atoms: vec![],
            hard: HashSet::new(),
            aligned: HashSet::new(),
            groups: vec![],
            directives: vec![],
        };
        plan.visit(root, 0, "", false, 0);
        plan.freeze_directive_lines();
        plan
    }

    fn text(&self, node: Node<'_>) -> &'a str {
        &self.source[node.byte_range()]
    }
    fn mark(&mut self, node: Node<'_>) {
        self.hard.insert(node.start_byte());
    }

    fn push(
        &mut self,
        node: Node<'_>,
        indent: usize,
        parent: &'static str,
        compact: bool,
        opaque: bool,
    ) {
        if node.end_byte() > node.start_byte() {
            self.atoms.push(Atom {
                range: node.byte_range(),
                kind: node.kind(),
                parent,
                indent,
                compact,
                opaque,
                frozen_line: false,
            });
        }
    }

    fn visit(
        &mut self,
        node: Node<'_>,
        indent: usize,
        parent: &'static str,
        compact: bool,
        depth: usize,
    ) {
        let kind = node.kind();
        if directive(kind) {
            self.directives.push(node.byte_range());
            self.push(node, indent, parent, compact, true);
            return;
        }
        if atomic(kind) || node.child_count() == 0 || depth >= 128 {
            self.push(
                node,
                indent,
                parent,
                compact,
                depth >= 128 || kind == "attribute_instance" || kind.contains("macro_usage"),
            );
            return;
        }
        let compact = compact
            || matches!(
                kind,
                "packed_dimension"
                    | "unpacked_dimension"
                    | "constant_range"
                    | "indexed_range"
                    | "constant_indexed_range"
                    | "parameter_value_range"
                    | "port_branch"
            );
        let nodes = children(node);
        let next = depth + 1;
        match kind {
            "source_file" => {
                for child in nodes {
                    self.mark(child);
                    self.visit(child, indent, kind, compact, next);
                }
            }
            "module_declaration"
            | "analog_function_declaration"
            | "function_body_declaration"
            | "task_body_declaration" => {
                let header_end = nodes.iter().position(|n| n.kind() == ";");
                for (i, child) in nodes.iter().copied().enumerate() {
                    let end = matches!(child.kind(), "endmodule" | "endfunction" | "endtask");
                    let inside = header_end.is_some_and(|h| i > h) && !end;
                    if (inside && child.is_named()) || end {
                        self.mark(child);
                    }
                    self.visit(child, indent + usize::from(inside), kind, compact, next);
                }
            }
            "nature_declaration" | "discipline_declaration" | "connectrules_declaration" => {
                for child in nodes {
                    let item = matches!(
                        child.kind(),
                        "nature_attribute"
                            | "nature_binding"
                            | "discipline_domain_binding"
                            | "nature_attribute_override"
                            | "connect_insertion"
                            | "connect_resolution"
                    ) || directive(child.kind());
                    let end = matches!(
                        child.kind(),
                        "endnature" | "enddiscipline" | "endconnectrules"
                    );
                    if item || end {
                        self.mark(child);
                    }
                    self.visit(child, indent + usize::from(item), kind, compact, next);
                }
            }
            "analog_seq_block"
            | "analog_function_seq_block"
            | "seq_block"
            | "par_block"
            | "generate_region"
            | "generate_block" => {
                let opener = nodes
                    .iter()
                    .position(|n| matches!(n.kind(), "begin" | "fork" | "generate"));
                if let Some(open) = opener {
                    let mut header_end = open;
                    if let Some(colon) = nodes.get(open + 1).filter(|n| n.kind() == ":") {
                        let _ = colon;
                        header_end = (open + 2).min(nodes.len() - 1);
                    }
                    let closing = nodes
                        .iter()
                        .position(|n| matches!(n.kind(), "end" | "join_keyword" | "endgenerate"))
                        .unwrap_or(nodes.len());
                    for (i, child) in nodes.iter().copied().enumerate() {
                        let inside = i > header_end && i < closing;
                        if inside || i == closing {
                            self.mark(child);
                        }
                        self.visit(child, indent + usize::from(inside), kind, compact, next);
                    }
                } else {
                    for child in nodes {
                        self.visit(child, indent, kind, compact, next);
                    }
                }
            }
            "analog_case_statement"
            | "case_statement"
            | "case_generate_construct"
            | "analog_function_case_statement" => {
                let head = nodes.iter().position(|n| n.kind() == ")").unwrap_or(0);
                let mut label = true;
                for (i, child) in nodes.iter().copied().enumerate() {
                    let end = child.kind() == "endcase";
                    let inside = i > head && !end;
                    if end
                        || (inside
                            && (label
                                || matches!(
                                    child.kind(),
                                    "analog_case_item" | "case_item" | "case_generate_item"
                                )))
                    {
                        self.mark(child);
                    }
                    if inside && child.kind() != "comment" {
                        label = body(child.kind());
                    }
                    self.visit(child, indent + usize::from(inside), kind, compact, next);
                }
            }
            "analog_conditional_statement"
            | "analog_function_conditional_statement"
            | "conditional_statement"
            | "if_generate_construct"
            | "analog_loop_statement"
            | "analog_function_loop_statement"
            | "loop_statement"
            | "loop_generate_construct"
            | "analog_event_control_statement"
            | "procedural_timing_control_statement"
            | "wait_statement" => {
                let mut previous_block = false;
                let mut after_else = false;
                for child in nodes {
                    if child.kind() == "else" {
                        self.aligned.insert(child.start_byte());
                        if !previous_block {
                            self.mark(child);
                        }
                        after_else = true;
                        self.visit(child, indent, kind, compact, next);
                    } else if body(child.kind()) {
                        let nested_if = after_else
                            && matches!(
                                unwrap(child).kind(),
                                "analog_conditional_statement"
                                    | "analog_function_conditional_statement"
                                    | "conditional_statement"
                                    | "if_generate_construct"
                            );
                        let is_block = block(child);
                        let is_null = self.text(child).trim() == ";";
                        let extra = !is_block && !nested_if && !is_null;
                        if is_block || nested_if {
                            self.aligned.insert(child.start_byte());
                        }
                        if extra {
                            self.mark(child);
                        }
                        self.visit(child, indent + usize::from(extra), kind, compact, next);
                        previous_block = is_block;
                        after_else = false;
                    } else {
                        self.visit(child, indent, kind, compact, next);
                    }
                }
            }
            "analog_construct" | "initial_construct" | "always_construct" => {
                for child in nodes {
                    if block(child) {
                        self.aligned.insert(child.start_byte());
                    }
                    self.visit(child, indent, kind, compact, next);
                }
            }
            _ if transparent(kind) => {
                if list(kind) {
                    self.groups.push(Group {
                        range: node.byte_range(),
                    });
                }
                for child in nodes {
                    self.visit(child, indent, kind, compact, next);
                }
            }
            _ => self.push(node, indent, parent, compact, true),
        }
    }

    fn freeze_directive_lines(&mut self) {
        let mut spans: Vec<Range<usize>> = Vec::new();
        for range in &self.directives {
            let start = self.source[..range.start].rfind('\n').map_or(0, |i| i + 1);
            let last = range.end.saturating_sub(1);
            let end = self.source[last..]
                .find('\n')
                .map_or(self.source.len(), |i| last + i + 1);
            if let Some(previous) = spans.last_mut().filter(|r| r.end >= start) {
                previous.end = previous.end.max(end);
            } else {
                spans.push(start..end);
            }
        }
        if spans.is_empty() {
            return;
        }
        let mut output = Vec::new();
        let mut i = 0;
        for range in spans {
            while i < self.atoms.len() && self.atoms[i].range.end <= range.start {
                output.push(self.atoms[i].clone());
                i += 1;
            }
            // Normally directives own complete lines. If an opaque parent extends
            // past that line, preserve that entire parent too rather than split it.
            let start = if i < self.atoms.len() {
                range.start.min(self.atoms[i].range.start)
            } else {
                range.start
            };
            let mut end = range.end;
            while i < self.atoms.len() && self.atoms[i].range.start < end {
                end = end.max(self.atoms[i].range.end);
                i += 1;
            }
            output.push(Atom {
                range: start..end,
                kind: "preprocessor_lines",
                parent: "",
                indent: 0,
                compact: false,
                opaque: true,
                frozen_line: true,
            });
        }
        output.extend_from_slice(&self.atoms[i..]);
        self.atoms = output;
    }
}

pub(crate) fn atoms<'a>(source: &'a str, root: Node<'_>) -> Vec<(&'static str, &'a str)> {
    Plan::new(source, root)
        .atoms
        .into_iter()
        .map(|a| (a.kind, &source[a.range]))
        .collect()
}

struct Gap {
    flat: String,
    lines: String,
    hard: bool,
    soft: bool,
    continuation: bool,
}

fn is_operator(text: &str) -> bool {
    matches!(
        text,
        "+" | "-"
            | "*"
            | "/"
            | "%"
            | "**"
            | "<"
            | ">"
            | "<="
            | ">="
            | "=="
            | "!="
            | "==="
            | "!=="
            | "&&"
            | "||"
            | "&"
            | "|"
            | "^"
            | "~^"
            | "^~"
            | "<<"
            | ">>"
            | "<<<"
            | ">>>"
            | "?"
            | ":"
    )
}

fn space(left: &Atom, right: &Atom, source: &str) -> &'static str {
    let a = &source[left.range.clone()];
    let b = &source[right.range.clone()];
    if a.starts_with('\\') {
        return " ";
    }
    if left.kind == "comment" || right.kind == "comment" {
        return " ";
    }
    if a == "," && b == "," {
        return " ";
    }
    if matches!(b, ";" | "," | ")" | "]" | "}") || b == "." || a == "." {
        return "";
    }
    if b == "(" {
        return if left.kind != "unary_operator"
            && (is_operator(a)
                || matches!(
                    a,
                    "=" | "<+"
                        | ","
                        | ")"
                        | "if"
                        | "for"
                        | "while"
                        | "repeat"
                        | "case"
                        | "casez"
                        | "casex"
                        | "wait"
                        | "from"
                        | "exclude"
                ))
        {
            " "
        } else {
            ""
        };
    }
    if b == "[" {
        return if right.parent == "packed_dimension" {
            " "
        } else {
            ""
        };
    }
    if matches!(a, "(" | "[" | "{" | "'{" | "#" | "@") {
        return "";
    }
    if (a == ":" || b == ":") && (left.compact || right.compact) {
        return "";
    }
    if b == ":"
        && matches!(
            right.parent,
            "case_item"
                | "analog_case_item"
                | "case_generate_item"
                | "analog_function_case_statement"
        )
    {
        return "";
    }
    if right.parent == "port_branch" || left.parent == "port_branch" {
        return "";
    }
    if left.kind == "unary_operator" {
        return "";
    }
    " "
}

fn newlines(gap: &str) -> String {
    let mut result = String::new();
    let mut previous = 0u8;
    for byte in gap.bytes() {
        if byte == b'\n' {
            if previous == b'\r' {
                result.push('\r');
            }
            result.push('\n');
        }
        previous = byte;
    }
    result
}

fn indent(atom: &Atom, extra: usize, continuation: bool, options: Options) -> String {
    if atom.frozen_line {
        return String::new();
    }
    " ".repeat((atom.indent + extra + usize::from(continuation)) * options.indent_width)
}

fn advance(column: usize, text: &str) -> usize {
    match text.rsplit_once('\n') {
        Some((_, tail)) => tail.chars().count(),
        None => column + text.chars().count(),
    }
}

fn render_gap(
    i: usize,
    atoms: &[Atom],
    gaps: &[Gap],
    extras: &[usize],
    options: Options,
    source: &str,
) -> String {
    let gap = &gaps[i];
    let after_newline = i > 0 && source[atoms[i - 1].range.clone()].ends_with('\n');
    if !gap.lines.is_empty() {
        format!(
            "{}{}",
            gap.lines,
            indent(&atoms[i], extras[i], gap.continuation && !gap.hard, options)
        )
    } else if after_newline {
        indent(&atoms[i], extras[i], gap.continuation && !gap.hard, options)
    } else {
        gap.flat.clone()
    }
}

// Only scan the current physical line. Re-rendering the entire file for every
// argument list would make ordinary files with many calls need quadratic work.
fn column_before(
    index: usize,
    atoms: &[Atom],
    gaps: &[Gap],
    extras: &[usize],
    options: Options,
    source: &str,
) -> usize {
    let mut column = 0;
    let mut i = index;
    loop {
        let gap = render_gap(i, atoms, gaps, extras, options, source);
        if gap.contains('\n') {
            return column + advance(0, &gap);
        }
        column += gap.chars().count();
        if i == 0 {
            return column;
        }
        i -= 1;
        let text = &source[atoms[i].range.clone()];
        if text.contains('\n') {
            return column + advance(0, text);
        }
        column += text.chars().count();
    }
}

fn line_indent(
    index: usize,
    atoms: &[Atom],
    gaps: &[Gap],
    extras: &[usize],
    source: &str,
) -> usize {
    let mut i = index;
    loop {
        if i == 0 || !gaps[i].lines.is_empty() || source[atoms[i - 1].range.clone()].ends_with('\n')
        {
            return atoms[i].indent
                + extras[i]
                + usize::from(gaps[i].continuation && !gaps[i].hard);
        }
        if source[atoms[i - 1].range.clone()].contains('\n') {
            return atoms[index].indent + extras[index];
        }
        i -= 1;
    }
}

pub(crate) fn format(source: &str, root: Node<'_>, options: Options) -> Result<String, Error> {
    let plan = Plan::new(source, root);
    let atoms = &plan.atoms;
    if atoms.is_empty() {
        return Ok(source.to_owned());
    }
    let newline = if source
        .find('\n')
        .is_some_and(|i| i > 0 && source.as_bytes()[i - 1] == b'\r')
    {
        "\r\n"
    } else {
        "\n"
    };
    let mut gaps = Vec::with_capacity(atoms.len());
    let mut previous = 0;
    for (i, atom) in atoms.iter().enumerate() {
        if atom.range.start < previous {
            return Err(Error::rejected(
                "overlapping syntax spans; leaving source untouched",
            ));
        }
        let original = &source[previous..atom.range.start];
        if !original.bytes().all(|b| b.is_ascii_whitespace()) {
            return Err(Error::rejected(
                "unclassified source bytes; leaving source untouched",
            ));
        }
        let left = i.checked_sub(1).map(|j| &atoms[j]);
        let trailing_comment = atom.kind == "comment"
            && left.is_some()
            && !original.contains('\n')
            && !source[atoms[i - 1].range.clone()].ends_with('\n');
        let mut hard = plan.hard.contains(&atom.range.start) && !trailing_comment;
        let mut lines = newlines(original);
        if left.is_some_and(|a| a.kind == "comment" && source[a.range.clone()].starts_with("//")) {
            hard = true;
        }
        let after_newline = left.is_some_and(|a| source[a.range.clone()].ends_with('\n'));
        if hard && lines.is_empty() && !after_newline && i > 0 {
            lines.push_str(newline);
        }
        let flat = left.map_or("", |a| space(a, atom, source)).to_owned();
        let soft = left.is_some_and(|a| {
            let text = &source[a.range.clone()];
            text == "," || matches!(text, "=" | "<+")
        }) || (!atom.compact
            && is_operator(&source[atom.range.clone()])
            && matches!(
                atom.parent,
                "expression" | "constant_expression" | "conditional_expression"
            ));
        gaps.push(Gap {
            flat,
            lines,
            hard,
            soft: soft && !trailing_comment,
            continuation: i > 0 && !hard && !plan.aligned.contains(&atom.range.start),
        });
        previous = atom.range.end;
    }
    if !source[previous..].bytes().all(|b| b.is_ascii_whitespace()) {
        return Err(Error::rejected(
            "unclassified trailing source bytes; leaving source untouched",
        ));
    }
    // Lists and expression wraps settle together. Breaks only accumulate: if a
    // wrap makes an inner list multiline, expand it before returning, not on the
    // user's next format request.
    let mut groups: Vec<_> = plan.groups.iter().collect();
    groups.sort_by_key(|g| (g.range.start, std::cmp::Reverse(g.range.end)));
    let mut lists = Vec::new();
    for group in groups {
        let start = atoms.partition_point(|a| a.range.start < group.range.start);
        let end = atoms.partition_point(|a| a.range.end <= group.range.end);
        if start >= end {
            continue;
        }
        let Some(open) =
            (start..end).find(|i| matches!(&source[atoms[*i].range.clone()], "(" | "{" | "'{"))
        else {
            continue;
        };
        let Some(close) = (start..end)
            .rev()
            .find(|i| matches!(&source[atoms[*i].range.clone()], ")" | "}"))
        else {
            continue;
        };
        if close <= open + 1
            || atoms[open..=close]
                .iter()
                .any(|a| a.opaque || source[a.range.clone()].contains('\n'))
        {
            continue;
        }
        lists.push((open, close, None::<usize>));
    }
    let mut extras = vec![0usize; atoms.len()];
    let mut rendered = Vec::new();
    let mut settled = false;
    for _ in 0..2 * lists.len() + 4 {
        let mut changed = false;
        for (open, close, boost) in &mut lists {
            let (open, close) = (*open, *close);
            let column = column_before(open, atoms, &gaps, &extras, options, source);
            let mut width: usize = (open..=close)
                .map(|i| {
                    source[atoms[i].range.clone()].chars().count()
                        + if i > open { gaps[i].flat.len() } else { 0 }
                })
                .sum();
            for i in close + 1..atoms.len() {
                if gaps[i].hard
                    || !gaps[i].lines.is_empty()
                    || !matches!(&source[atoms[i].range.clone()], ")" | "]" | "}" | "," | ";")
                {
                    break;
                }
                width += gaps[i].flat.len() + source[atoms[i].range.clone()].chars().count();
            }
            let multiline = (open + 1..=close).any(|i| !gaps[i].lines.is_empty());
            if boost.is_none() && !multiline && column + width <= options.line_width {
                continue;
            }
            let anchor = line_indent(open, atoms, &gaps, &extras, source)
                .saturating_sub(atoms[open].indent + extras[open]);
            if *boost != Some(anchor) {
                let old = boost.map_or(0, |b| b + 1);
                for extra in &mut extras[open + 1..close] {
                    *extra = *extra - old + anchor + 1;
                }
                extras[close] = extras[close] - boost.unwrap_or(0) + anchor;
                *boost = Some(anchor);
                changed = true;
            }
            let mut breaks = vec![open + 1, close];
            let mut nesting = 0i32;
            for (i, atom) in atoms.iter().enumerate().take(close).skip(open + 1) {
                match &source[atom.range.clone()] {
                    "(" | "[" | "{" | "'{" => nesting += 1,
                    ")" | "]" | "}" => nesting -= 1,
                    "," if nesting == 0 => breaks.push(i + 1),
                    _ => (),
                }
            }
            for i in breaks {
                if atoms[i].kind == "comment" && gaps[i].lines.is_empty() {
                    continue;
                }
                gaps[i].hard = true;
                gaps[i].continuation = false;
                if gaps[i].lines.is_empty() {
                    gaps[i].lines = newline.to_owned();
                    changed = true;
                }
            }
        }
        rendered = (0..atoms.len())
            .map(|i| render_gap(i, atoms, &gaps, &extras, options, source))
            .collect();
        let mut column = 0;
        let mut last_break = None;
        let mut i = 0;
        while i < atoms.len() {
            if rendered[i].contains('\n')
                || (i > 0 && source[atoms[i - 1].range.clone()].ends_with('\n'))
            {
                column = advance(0, &rendered[i]);
                last_break = None;
            } else {
                if gaps[i].soft && i > 0 {
                    last_break = Some(i);
                }
                column = advance(column, &rendered[i]);
            }
            let text = &source[atoms[i].range.clone()];
            let length = text.split('\n').next().unwrap_or("").chars().count();
            if column + length > options.line_width
                && !atoms[i].opaque
                && atoms[i].kind != "comment"
            {
                if let Some(at) = last_break {
                    gaps[at].lines = newline.to_owned();
                    gaps[at].continuation = true;
                    rendered[at] = render_gap(at, atoms, &gaps, &extras, options, source);
                    changed = true;
                    i = at;
                    column = 0;
                    last_break = None;
                    continue;
                }
            }
            column = advance(column, text);
            if text.contains('\n') {
                last_break = None;
            }
            i += 1;
        }
        if !changed {
            settled = true;
            break;
        }
    }
    if !settled {
        return Err(Error::rejected(
            "layout did not settle; leaving source untouched",
        ));
    }
    let mut output = String::with_capacity(source.len());
    for (i, atom) in atoms.iter().enumerate() {
        output.push_str(&rendered[i]);
        output.push_str(&source[atom.range.clone()]);
    }
    output.push_str(&source[previous..]);
    Ok(output)
}
