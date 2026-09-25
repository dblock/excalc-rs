//! Abstract syntax tree produced by the parser and consumed by the evaluator.

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    /// A bare identifier that isn't followed by `(` — a constant (`pi`, `e`)
    /// or a single-letter variable.
    Variable(String),
    /// A named function call with a variable number of arguments, e.g.
    /// `sin(x)`, `sum(1, 2, 3)`, `gcd(12, 18)`.
    Call(String, Vec<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
}

/// One statement in a program: either a variable assignment or a plain
/// expression to evaluate. A full input is a `;`/newline-separated sequence
/// of these (see [`Program`]); the value of the last statement is the
/// program's result.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `name := expr` — evaluates `expr` and binds it to `name`, visible to
    /// later statements in the same program. `pi`/`e` can't be assigned to.
    Assign(String, Expr),
    /// `name(param, ...) := body` — defines a user function, visible (like
    /// a variable) to later statements in the same session. `name` can't
    /// collide with a built-in function/constant, and parameter names must
    /// be distinct.
    DefineFunction(String, Vec<String>, Expr),
    Expr(Expr),
}

/// A full program: a sequence of statements evaluated in order.
pub type Program = Vec<Stmt>;

impl std::fmt::Display for Expr {
    /// Reconstructs calculator source text for this expression, adding
    /// parentheses only where required by precedence/associativity (see
    /// `parser.rs`'s module doc for the precedence table this mirrors).
    /// Used by the REPL's `vars` command to show a user-defined function's
    /// actual body instead of a placeholder.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", fmt_expr(self, 0))
    }
}

/// Binding power of an expression's outermost operator: higher binds
/// tighter. Matches the precedence levels in `parser.rs`'s module doc
/// (comparison loosest, postfix tightest); primaries (numbers, variables,
/// calls) are always tightest since they never need parenthesizing.
fn precedence(expr: &Expr) -> u8 {
    match expr {
        Expr::Number(_) | Expr::Variable(_) | Expr::Call(_, _) => 9,
        Expr::Unary(UnaryOp::Neg, _) => 6,
        Expr::Unary(UnaryOp::Factorial | UnaryOp::Percent, _) => 8,
        Expr::Binary(op, _, _) => match op {
            BinaryOp::Eq | BinaryOp::Gt | BinaryOp::Lt => 1,
            BinaryOp::Or | BinaryOp::Nor | BinaryOp::Xor | BinaryOp::Xnor => 2,
            BinaryOp::And | BinaryOp::Nand => 3,
            BinaryOp::Add | BinaryOp::Sub => 4,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 5,
            BinaryOp::Pow | BinaryOp::Root => 7,
        },
    }
}

fn binary_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "mod",
        BinaryOp::Pow => "^",
        BinaryOp::Root => "\\",
        BinaryOp::Eq => "=",
        BinaryOp::Gt => ">",
        BinaryOp::Lt => "<",
        BinaryOp::Or => "or",
        BinaryOp::Nor => "nor",
        BinaryOp::Xor => "xor",
        BinaryOp::Xnor => "xnor",
        BinaryOp::And => "and",
        BinaryOp::Nand => "nand",
    }
}

/// Formats `expr` as source text, wrapping it in parentheses if its own
/// precedence is lower than `min_prec` (the binding power required by
/// whatever context it's nested in).
fn fmt_expr(expr: &Expr, min_prec: u8) -> String {
    let prec = precedence(expr);
    let text = match expr {
        Expr::Number(n) => n.to_string(),
        Expr::Variable(name) => name.clone(),
        Expr::Call(name, args) => format!(
            "{name}({})",
            args.iter()
                .map(|a| fmt_expr(a, 0))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Unary(UnaryOp::Neg, inner) => format!("-{}", fmt_expr(inner, prec)),
        Expr::Unary(UnaryOp::Factorial, inner) => format!("{}!", fmt_expr(inner, prec)),
        Expr::Unary(UnaryOp::Percent, inner) => format!("{}%", fmt_expr(inner, prec)),
        Expr::Binary(op, lhs, rhs) => {
            // Right-associative (`^`) needs parens around an equal-precedence
            // left operand; every other (left-associative) operator needs
            // them around an equal-precedence right operand instead.
            let (left_prec, right_prec) = if *op == BinaryOp::Pow {
                (prec + 1, prec)
            } else {
                (prec, prec + 1)
            };
            format!(
                "{} {} {}",
                fmt_expr(lhs, left_prec),
                binary_symbol(*op),
                fmt_expr(rhs, right_prec)
            )
        }
    };
    if prec < min_prec {
        format!("({text})")
    } else {
        text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    /// Postfix factorial: `5!`
    Factorial,
    /// Postfix percent: `50%` == `0.5`
    Percent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    /// `n Root x` = the n-th root of x (`2 Root 9` = 3).
    Root,
    /// Comparison operators; evaluate to `1.0` (true) or `0.0` (false).
    Eq,
    Gt,
    Lt,
    /// Bitwise operators; operands are truncated to `i64` before applying
    /// the operation.
    Or,
    Nor,
    Xor,
    Xnor,
    And,
    Nand,
}

#[cfg(test)]
mod tests {
    use crate::parser;

    fn display(input: &str) -> String {
        parser::parse(input).unwrap().to_string()
    }

    #[test]
    fn display_preserves_semantically_required_parentheses_only() {
        assert_eq!(display("x^2 + 1"), "x ^ 2 + 1");
        assert_eq!(display("(x + y) * 2"), "(x + y) * 2");
        assert_eq!(display("-x + 3"), "-x + 3");
    }

    #[test]
    fn display_right_associative_power_needs_parens_on_the_left_only() {
        assert_eq!(display("x^2^3"), "x ^ 2 ^ 3");
        assert_eq!(display("(x^2)^3"), "(x ^ 2) ^ 3");
    }

    #[test]
    fn display_left_associative_subtraction_needs_parens_on_the_right_only() {
        assert_eq!(display("2 - (3 - x)"), "2 - (3 - x)");
        assert_eq!(display("(2 - 3) - x"), "2 - 3 - x");
    }

    #[test]
    fn display_postfix_and_calls_round_trip() {
        assert_eq!(display("x!"), "x!");
        assert_eq!(display("sqrt(x) + x^2"), "sqrt(x) + x ^ 2");
    }
}
