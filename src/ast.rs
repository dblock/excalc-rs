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
    Expr(Expr),
}

/// A full program: a sequence of statements evaluated in order.
pub type Program = Vec<Stmt>;

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
