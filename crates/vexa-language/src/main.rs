use std::env::args;
use std::fs::read_to_string;
use std::process::ExitCode;

#[derive(Debug, Copy, Clone)]
pub enum Op { Plus, Minus, Divide, Multiply }

#[derive(Debug, Copy, Clone)]
pub enum Type { U8, U64 }

#[derive(Debug)]
pub enum Expr {
    Num(String),
    Ident(String),
    Op(Box<Expr>, Op, Box<Expr>),
    Call{name:String, args:Vec<Expr>},
}

#[derive(Debug)]
pub enum Statement {
    FunctionDeclaration {name:String, args:Vec<(Type, String)>, body: Vec<Statement>},
    LetStmt { type_: Type, name: String, value: Expr },
    ExprStmt(Expr),
}

fn fold(first: Expr, rest: Vec<(Op, Expr)>) -> Expr {
    rest.into_iter()
        .fold(first, |l, (op, r)| Expr::Op(Box::new(l), op, Box::new(r)))
}

peg::parser!{
    grammar parser() for str {
        rule _() = quiet!{ (" " / "\t" / "\r" / "\n" / Comment())* }
        rule Comment() = "//" (!"\n" [_])*

        rule SEMI() = ";" _
        rule LPAREN() = "(" _
        rule RPAREN() = ")" _
        rule LBRACE() = "{" _
        rule RBRACE() = "}" _
        rule COMMA() = "," _
        rule EQ() = "=" _
        rule PLUS() -> Op = "+" _ { Op::Plus }
        rule MINUS() -> Op = "-" _ { Op::Minus }
        rule DIVIDE() -> Op = "/" _ { Op::Divide }
        rule MULTIPLY() -> Op = "*" _ { Op::Multiply }

        rule Ident() -> String
            = !Keyword() fc:['a'..='z' | 'A'..='Z' | '_'] cc:(IdChar()*) _
                { std::iter::once(fc).chain(cc).collect() }

        rule TypedIdent() -> (Type, String)
            = t:Type() ident:Ident() { (t, ident) }

        rule IdChar() -> char
            = c:['a'..='z' | 'A'..='Z' | '0'..='9' | '_'] { c }

        rule U8() -> Type = "u8" !IdChar() _ { Type::U8 }
        rule U64() -> Type = "u64" !IdChar() _ { Type::U64 }
        rule LET() = "let" !IdChar() _
        rule FN() = "fn" !IdChar() _
        rule Type() -> Type = U8() / U64()

        rule Keyword() = Type() / LET() / FN()

        rule UnsignedNumber() -> Expr
            = n:$(['0'..='9']+) !IdChar() _ { Expr::Num(n.to_string()) }

        rule Factor() -> Expr
            = UnsignedNumber() / Call() / i:Ident() {Expr::Ident(i)} / LPAREN() e:Expr() RPAREN() {e}

        rule Term() -> Expr
            = first:Factor() rest:(op:(MULTIPLY() / DIVIDE()) f:Factor() {(op, f)})*
                { fold(first, rest) }

        rule Expr() -> Expr
            = first:Term() rest:(op:(PLUS() / MINUS()) f:Term() {(op, f)})*
                { fold(first, rest) }

        rule Args() -> Vec<Expr>
            = a:(Expr() ++ COMMA()) COMMA()? { a } / { Vec::new() }

        rule Call() -> Expr
            = name:Ident() LPAREN() args:Args() RPAREN()
                { Expr::Call {name, args} }

        rule FunctionDeclaration() -> Statement
           = FN() name:Ident() LPAREN() args:FunctionDeclarationArgs() RPAREN() LBRACE() body:(Statement()*) RBRACE()
                { Statement::FunctionDeclaration {name, args, body} }

        rule FunctionDeclarationArgs() -> Vec<(Type, String)>
            = a:(TypedIdent() ++ COMMA()) COMMA()? { a } / { Vec::new() }

        rule LetStmt() -> Statement
            = LET() i:TypedIdent() EQ() value:Expr() SEMI()
                { Statement::LetStmt {type_: i.0, name: i.1, value} }

        rule ExprStmt() -> Statement
            = e:Expr() SEMI()
                { Statement::ExprStmt(e) }

        rule Statement() -> Statement
            = FunctionDeclaration() / LetStmt() / ExprStmt()

        // No need for checking for EOF since peg does it automatically
        pub rule Program() -> Vec<Statement>
            = _ func:(FunctionDeclaration()*)
                {func}
    }
}

fn print_ast(ast: Statement, ident: usize) {
    let ident_str = "  ".repeat(ident);
    match ast {
        Statement::FunctionDeclaration { name, args, body } => {
            let args_str = args.iter().map(|(s, a)| format!("{:?} {}", s, a)).collect::<Vec<_>>().join(", ");
            if !body.is_empty() {
                println!("{}Function {}({}) (", ident_str, name, args_str);
                for stmt in body {
                    print_ast(stmt, ident + 1);
                }
                println!("{})", ident_str);
            } else {
                println!("{}Function {}({}) (  )  // Empty function", ident_str, name, args_str);
            }
        }
        Statement::LetStmt { type_, name, value } => {
            println!("{}Variable {} ({:?}) = (", ident_str, name, type_);
            print_expr(value, ident + 1);
            println!("{})", ident_str);
        }
        Statement::ExprStmt(e) => {
            print_expr(e, ident);
        }
    }
}

fn print_expr(expr: Expr, ident: usize) {
    let ident_str = "  ".repeat(ident);
    match expr {
        Expr::Num(num) => {println!("{}Number {}", ident_str, num);}
        Expr::Ident(label) => {println!("{}Ident {}", ident_str, label);}
        Expr::Op(l, op, r) => {
            println!("{}(", ident_str);
            print_expr(*l, ident + 1);
            println!("{}{:?}", ident_str, op);
            print_expr(*r, ident + 1);
            println!("{})", ident_str);
        }
        Expr::Call { name, args } => {
            if !args.is_empty() {
                println!("{}Call {} (", ident_str, name, );
                for arg in args {
                    print_expr(arg, ident + 1);
                }
                println!("{})", ident_str);
            } else {
                println!("{}Call {}()  // No argument call", ident_str, name);
            }
        }
    }
}

fn main() -> ExitCode {
    let args = args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: {} <file.vxn>", args[0]);
        return ExitCode::FAILURE;
    }
    let source = read_to_string(args[1].clone()).unwrap();
    match parser::Program(source.as_str()) {
        Ok(asts) => {
            for ast in asts {
                print_ast(ast, 0);
            }
            ExitCode::SUCCESS
        },
        Err(e) => {
            println!("Error: {}", e);
            ExitCode::FAILURE
        },
    }
}
