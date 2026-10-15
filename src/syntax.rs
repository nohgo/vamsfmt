use crate::Error;
use tree_sitter::{Node, Parser, Tree};

pub(crate) fn parser() -> Result<Parser, Error> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_verilog_ams::LANGUAGE.into())
        .map_err(|e| Error::usage(format!("cannot load grammar: {e}")))?;
    Ok(parser)
}

pub(crate) fn parse(parser: &mut Parser, source: &str) -> Result<Tree, Error> {
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| Error::usage("parser did not produce a tree"))?;
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.is_error() || node.is_missing() {
            let p = node.start_position();
            let problem = if node.is_missing() {
                format!("missing {:?}", node.kind())
            } else {
                "unrecognized syntax".to_owned()
            };
            return Err(Error::rejected(format!(
                "{}:{}: {problem}",
                p.row + 1,
                p.column + 1
            )));
        }
        if cursor.goto_first_child() {
            continue;
        }
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                drop(cursor);
                return Ok(tree);
            }
        }
    }
}

// Preorder kind/arity records preserve structure, not just a token bag. Leaf text
// includes punctuation, comments and literal contents. Positions are irrelevant.
pub(crate) fn signature<'a>(root: Node<'_>, source: &'a str) -> Vec<(u16, usize, &'a str)> {
    let mut result = Vec::new();
    let mut cursor = root.walk();
    loop {
        let node = cursor.node();
        let count = node.child_count();
        result.push((
            node.kind_id(),
            count,
            if count == 0 {
                &source[node.byte_range()]
            } else {
                ""
            },
        ));
        if cursor.goto_first_child() {
            continue;
        }
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                return result;
            }
        }
    }
}
