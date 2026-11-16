use super::graph::Graph;
use crate::steps::parser::{Expr, Op, Statement, Type};

fn type_str(t: Type) -> &'static str {
    match t {
        Type::U8 => "u8",
        Type::U64 => "u64",
    }
}

fn op_str(op: Op) -> &'static str {
    match op {
        Op::Plus => "+",
        Op::Minus => "-",
        Op::Divide => "/",
        Op::Multiply => "*",
    }
}

fn add_expr(g: &mut Graph, expr: &Expr) -> String {
    match expr {
        Expr::Num(n) => g.add_node("Num", n),
        Expr::Ident(name) => g.add_node("Ident", name),
        Expr::Op(l, op, r) => {
            let id = g.add_node("Op", op_str(*op));
            let l = add_expr(g, l);
            let r = add_expr(g, r);
            g.connect_nodes(&id, &l);
            g.connect_nodes(&id, &r);
            id
        }
        Expr::Call { name, args } => {
            let id = g.add_node("Call", name);
            for arg in args {
                let a = add_expr(g, arg);
                g.connect_nodes(&id, &a);
            }
            id
        }
    }
}

fn add_statement(g: &mut Graph, stmt: &Statement) -> String {
    match stmt {
        Statement::FunctionDeclaration { name, args, body, return_type } => {
            let params = if args.is_empty() {
                "no params".to_string()
            } else {
                args.iter()
                    .map(|(t, n)| format!("{} {}", type_str(*t), n))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let ret = match return_type {
                Some(t) => format!("returns {}", type_str(*t)),
                None => "returns nothing".to_string(),
            };
            let id = g.add_node(&format!("fn {}", name), &format!("{}\n{}", params, ret));
            for s in body {
                let child = add_statement(g, s);
                g.connect_nodes(&id, &child);
            }
            id
        }
        Statement::LetStmt { type_, name, value } => {
            let id = g.add_node("let", &format!("{} {}", type_str(*type_), name));
            let v = add_expr(g, value);
            g.connect_nodes(&id, &v);
            id
        }
        Statement::ExprStmt(e) => {
            let id = g.add_node("expr", "statement");
            let v = add_expr(g, e);
            g.connect_nodes(&id, &v);
            id
        }
        Statement::ReturnStmt(e) => {
            let id = g.add_node("return", "value");
            let v = add_expr(g, e);
            g.connect_nodes(&id, &v);
            id
        }
    }
}

pub fn ast_to_dot(program: &[Statement]) -> String {
    let mut g = Graph::new();
    let root = g.add_node("Program", "top level");
    for stmt in program {
        let id = add_statement(&mut g, stmt);
        g.connect_nodes(&root, &id);
    }
    g.build()
}