//! Evaluates an `Expr` AST into a final `f64` value.

use std::collections::HashMap;

use crate::ast::{BinaryOp, Expr, Stmt, UnaryOp};
use crate::error::{CalcError, CalcResult};
use crate::functions::{
    advanced, base, combinatorics, financial, general, geometry,
    integration::{self, Rule},
    logic, numbertheory, rootfinding, stats, trig,
};

/// The result of evaluating a program or expression: almost always a plain
/// number, except for the base-conversion functions (`hex`/`oct`/`bin`),
/// which produce a formatted string instead.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Text(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::Text(s) => write!(f, "{s}"),
        }
    }
}

/// Evaluation context: currently just variable bindings. Constants (`pi`,
/// `e`) are always available and can't be shadowed.
#[derive(Default)]
pub struct Context {
    variables: HashMap<String, f64>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, name: impl Into<String>, value: f64) {
        self.variables.insert(name.into(), value);
    }
}

/// Evaluates a full program: a sequence of statements executed in order,
/// with variable assignments visible to later statements. Returns the value
/// of the last statement (an assignment's value is the value assigned, so a
/// program that ends in `x := 5` evaluates to `5`). Variables are always
/// numbers, so an assignment's right-hand side must evaluate to a number
/// (not the text result of `hex`/`oct`/`bin`).
pub fn eval_program(stmts: &[Stmt], ctx: &mut Context) -> CalcResult<Value> {
    let mut last = Value::Number(0.0);
    for stmt in stmts {
        last = match stmt {
            Stmt::Assign(name, expr) => {
                if is_reserved_constant(name) {
                    return Err(CalcError::ReservedIdentifier(name.clone()));
                }
                let value = eval_numeric(expr, ctx)?;
                ctx.set(name.clone(), value);
                Value::Number(value)
            }
            Stmt::Expr(expr) => eval(expr, ctx)?,
        };
    }
    Ok(last)
}

fn is_reserved_constant(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(), "pi" | "e")
}

/// Evaluates `expr` and requires the result to be a number, erroring with
/// `NotANumber` if it's the text result of `hex`/`oct`/`bin` (which can only
/// appear as a whole top-level expression, not nested inside arithmetic).
fn eval_numeric(expr: &Expr, ctx: &Context) -> CalcResult<f64> {
    match eval(expr, ctx)? {
        Value::Number(n) => Ok(n),
        Value::Text(_) => Err(CalcError::NotANumber),
    }
}

pub fn eval(expr: &Expr, ctx: &Context) -> CalcResult<Value> {
    match expr {
        Expr::Number(n) => Ok(Value::Number(*n)),

        Expr::Variable(name) => Ok(Value::Number(resolve_variable(name, ctx)?)),

        Expr::Unary(op, inner) => {
            let v = eval_numeric(inner, ctx)?;
            Ok(Value::Number(eval_unary(*op, v)?))
        }

        Expr::Binary(op, lhs, rhs) => {
            let l = eval_numeric(lhs, ctx)?;
            let r = eval_numeric(rhs, ctx)?;
            Ok(Value::Number(eval_binary(*op, l, r)?))
        }

        Expr::Call(name, arg_exprs) => {
            let lower = name.to_ascii_lowercase();
            if let Some(rule) = Rule::from_name(&lower) {
                return Ok(Value::Number(eval_composite_integration(
                    &lower, rule, arg_exprs, ctx,
                )?));
            }
            if lower == "int" || lower == "gauss" {
                return Ok(Value::Number(eval_adaptive_integration(
                    &lower, arg_exprs, ctx,
                )?));
            }
            if matches!(lower.as_str(), "bisect" | "secant") {
                return Ok(Value::Number(eval_root_finding(&lower, arg_exprs, ctx)?));
            }
            if matches!(lower.as_str(), "hex" | "oct" | "bin") {
                return eval_base_conversion(&lower, arg_exprs, ctx);
            }
            let mut args = Vec::with_capacity(arg_exprs.len());
            for a in arg_exprs {
                args.push(eval_numeric(a, ctx)?);
            }
            Ok(Value::Number(call_function(name, &args)?))
        }
    }
}

/// `hex(n)`, `oct(n)`, `bin(n)`: format a non-negative integer as a string
/// in the given base. Only meaningful as a whole expression (its result
/// can't be combined into further arithmetic), so it's handled here rather
/// than in `call_function`, which only ever returns numbers.
fn eval_base_conversion(name: &str, arg_exprs: &[Expr], ctx: &Context) -> CalcResult<Value> {
    if arg_exprs.len() != 1 {
        return Err(CalcError::WrongArgCount {
            name: name.to_string(),
            expected: "1".to_string(),
            got: arg_exprs.len(),
        });
    }
    let n = eval_numeric(&arg_exprs[0], ctx)?;
    let text = match name {
        "hex" => base::hex(n)?,
        "oct" => base::oct(n)?,
        "bin" => base::bin(n)?,
        _ => unreachable!("eval_base_conversion only called for hex/oct/bin"),
    };
    Ok(Value::Text(text))
}

/// Shared setup for the numeric-integration functions (`int`, `gauss`, and
/// the named composite rules): validates the fixed 5-argument arity,
/// extracts and validates the bare integration variable (2nd argument, per
/// the original's `fSum`/`Tegral` requiring a plain, non-reserved variable
/// node), and evaluates the bounds and rule-specific 5th parameter (`n` for
/// the composite rules, `tolerance` for `int`/`gauss`).
fn integration_setup<'a>(
    name: &str,
    arg_exprs: &'a [Expr],
    ctx: &Context,
) -> CalcResult<(&'a Expr, String, f64, f64, f64)> {
    if arg_exprs.len() != 5 {
        return Err(CalcError::WrongArgCount {
            name: name.to_string(),
            expected: "5".to_string(),
            got: arg_exprs.len(),
        });
    }
    let var_name = match &arg_exprs[1] {
        Expr::Variable(v) if !matches!(v.to_ascii_lowercase().as_str(), "pi" | "e") => v.clone(),
        _ => return Err(CalcError::DomainError(name.to_string())),
    };
    let a = eval_numeric(&arg_exprs[2], ctx)?;
    let b = eval_numeric(&arg_exprs[3], ctx)?;
    let param = eval_numeric(&arg_exprs[4], ctx)?;
    Ok((&arg_exprs[0], var_name, a, b, param))
}

fn eval_composite_integration(
    name: &str,
    rule: Rule,
    arg_exprs: &[Expr],
    ctx: &Context,
) -> CalcResult<f64> {
    let (body, var_name, a, b, n) = integration_setup(name, arg_exprs, ctx)?;
    let mut work_ctx = Context {
        variables: ctx.variables.clone(),
    };
    let f = |x: f64| -> CalcResult<f64> {
        work_ctx.set(var_name.clone(), x);
        eval_numeric(body, &work_ctx)
    };
    integration::composite(name, rule, f, a, b, n)
}

fn eval_adaptive_integration(name: &str, arg_exprs: &[Expr], ctx: &Context) -> CalcResult<f64> {
    let (body, var_name, a, b, tolerance) = integration_setup(name, arg_exprs, ctx)?;
    let mut work_ctx = Context {
        variables: ctx.variables.clone(),
    };
    let f = |x: f64| -> CalcResult<f64> {
        work_ctx.set(var_name.clone(), x);
        eval_numeric(body, &work_ctx)
    };
    integration::adaptive(name, f, a, b, tolerance)
}

/// `bisect(expr, var, a, b, tolerance)` / `secant(expr, var, x0, x1,
/// tolerance)`: root-finding functions sharing the same 5-argument shape
/// (expression, bare variable, two numeric bracket/seed values, tolerance)
/// as the numeric-integration functions above, so they reuse
/// `integration_setup` for arity/variable/argument validation.
fn eval_root_finding(name: &str, arg_exprs: &[Expr], ctx: &Context) -> CalcResult<f64> {
    let (body, var_name, a, b, tolerance) = integration_setup(name, arg_exprs, ctx)?;
    let mut work_ctx = Context {
        variables: ctx.variables.clone(),
    };
    let f = |x: f64| -> CalcResult<f64> {
        work_ctx.set(var_name.clone(), x);
        eval_numeric(body, &work_ctx)
    };
    match name {
        "bisect" => rootfinding::bisect(name, f, a, b, tolerance),
        "secant" => rootfinding::secant(name, f, a, b, tolerance),
        _ => unreachable!("eval_root_finding only called for bisect/secant"),
    }
}

fn resolve_variable(name: &str, ctx: &Context) -> CalcResult<f64> {
    match name.to_ascii_lowercase().as_str() {
        "pi" => Ok(std::f64::consts::PI),
        "e" => Ok(std::f64::consts::E),
        _ => ctx
            .variables
            .get(name)
            .copied()
            .ok_or_else(|| CalcError::UnknownVariable(name.to_string())),
    }
}

fn eval_unary(op: UnaryOp, v: f64) -> CalcResult<f64> {
    match op {
        UnaryOp::Neg => Ok(-v),
        UnaryOp::Percent => Ok(v / 100.0),
        UnaryOp::Factorial => factorial(v),
    }
}

fn factorial(v: f64) -> CalcResult<f64> {
    if v < 0.0 || v.fract() != 0.0 {
        return Err(CalcError::DomainError("factorial".to_string()));
    }
    let n = v as u64;
    let mut result: f64 = 1.0;
    for i in 2..=n {
        result *= i as f64;
        if result.is_infinite() {
            return Err(CalcError::Overflow);
        }
    }
    Ok(result)
}

fn eval_binary(op: BinaryOp, l: f64, r: f64) -> CalcResult<f64> {
    match op {
        BinaryOp::Add => Ok(l + r),
        BinaryOp::Sub => Ok(l - r),
        BinaryOp::Mul => Ok(l * r),
        BinaryOp::Div => {
            if r == 0.0 {
                Err(CalcError::DivisionByZero)
            } else {
                Ok(l / r)
            }
        }
        BinaryOp::Mod => {
            if r == 0.0 {
                Err(CalcError::DivisionByZero)
            } else {
                Ok(l % r)
            }
        }
        BinaryOp::Pow => Ok(l.powf(r)),
        BinaryOp::Root => {
            // `n Root x` = the n-th root of x = x^(1/n).
            if l == 0.0 {
                Err(CalcError::DivisionByZero)
            } else if r < 0.0 && (1.0 / l).fract() != 0.0 {
                Err(CalcError::DomainError("root".to_string()))
            } else {
                Ok(r.powf(1.0 / l))
            }
        }
        BinaryOp::Eq => Ok(if l == r { 1.0 } else { 0.0 }),
        BinaryOp::Gt => Ok(if l > r { 1.0 } else { 0.0 }),
        BinaryOp::Lt => Ok(if l < r { 1.0 } else { 0.0 }),
        BinaryOp::Or => bitwise(l, r, "or", |a, b| a | b),
        BinaryOp::Nor => bitwise(l, r, "nor", |a, b| !(a | b)),
        BinaryOp::Xor => bitwise(l, r, "xor", |a, b| a ^ b),
        BinaryOp::Xnor => bitwise(l, r, "xnor", |a, b| !(a ^ b)),
        BinaryOp::And => bitwise(l, r, "and", |a, b| a & b),
        BinaryOp::Nand => bitwise(l, r, "nand", |a, b| !(a & b)),
    }
}

/// Truncates both operands to `i64`, applies `f`, and converts the result
/// back to `f64`. Errors with `DomainError(name)` if either operand doesn't
/// fit in `i64` once truncated.
fn bitwise(l: f64, r: f64, name: &str, f: impl Fn(i64, i64) -> i64) -> CalcResult<f64> {
    let a = logic::to_i64(l, name)?;
    let b = logic::to_i64(r, name)?;
    Ok(f(a, b) as f64)
}

fn call_function(name: &str, args: &[f64]) -> CalcResult<f64> {
    let lower = name.to_ascii_lowercase();

    // Variadic statistics functions.
    match lower.as_str() {
        "sum" => return stats::sum(args),
        "average" | "avg" => return stats::average(args),
        "product" | "prod" => return stats::product(args),
        "min" => return stats::min(args),
        "max" => return stats::max(args),
        "median" => return stats::median(args),
        "mode" => return stats::mode(args),
        "variance" => return stats::variance(args),
        "stddev" => return stats::stddev(args),
        "percentile" => return stats::percentile(args),
        "covariance" => return stats::covariance(args),
        "correlation" => return stats::correlation(args),
        "gcd" => return numbertheory::gcd(args),
        "lcm" => return numbertheory::lcm(args),
        "multinomial" => return combinatorics::multinomial(args),
        "distance" => return geometry::distance(args),
        "manhattan" => return geometry::manhattan(args),
        "dot" => return geometry::dot(args),
        "norm" => return geometry::norm(args),
        _ => {}
    }

    // Fixed-arity functions below.
    let one_arg = |f: fn(f64) -> CalcResult<f64>| -> CalcResult<f64> {
        expect_args(&lower, args, 1)?;
        f(args[0])
    };

    match lower.as_str() {
        "sqrt" => {
            expect_args(&lower, args, 1)?;
            if args[0] < 0.0 {
                return Err(CalcError::DomainError("sqrt".to_string()));
            }
            Ok(args[0].sqrt())
        }
        "ln" => {
            expect_args(&lower, args, 1)?;
            if args[0] <= 0.0 {
                return Err(CalcError::DomainError("ln".to_string()));
            }
            Ok(args[0].ln())
        }
        "log" => {
            expect_args(&lower, args, 1)?;
            if args[0] <= 0.0 {
                return Err(CalcError::DomainError("log".to_string()));
            }
            Ok(args[0].log10())
        }
        "logn" => {
            expect_args(&lower, args, 2)?;
            if args[0] <= 0.0 || args[1] <= 0.0 || args[1] == 1.0 {
                return Err(CalcError::DomainError("logn".to_string()));
            }
            Ok(args[0].log(args[1]))
        }

        "sin" => one_arg(trig::sin),
        "cos" => one_arg(trig::cos),
        "tan" => one_arg(trig::tan),
        "asin" => one_arg(trig::asin),
        "acos" => one_arg(trig::acos),
        "atan" => one_arg(trig::atan),
        "atan2" => {
            expect_args(&lower, args, 2)?;
            trig::atan2(args[0], args[1])
        }
        "deg" => one_arg(trig::deg),
        "rad" => one_arg(trig::rad),
        "sind" => one_arg(trig::sind),
        "cosd" => one_arg(trig::cosd),
        "tand" => one_arg(trig::tand),
        "asind" => one_arg(trig::asind),
        "acosd" => one_arg(trig::acosd),
        "atand" => one_arg(trig::atand),
        "sinh" => one_arg(trig::sinh),
        "cosh" => one_arg(trig::cosh),
        "tanh" => one_arg(trig::tanh),
        "asinh" => one_arg(trig::asinh),
        "acosh" => one_arg(trig::acosh),
        "atanh" => one_arg(trig::atanh),
        "sec" => one_arg(trig::sec),
        "csc" => one_arg(trig::csc),
        "cot" => one_arg(trig::cot),
        "asec" => one_arg(trig::asec),
        "acsc" => one_arg(trig::acsc),
        "acot" => one_arg(trig::acot),
        "sech" => one_arg(trig::sech),
        "csch" => one_arg(trig::csch),
        "coth" => one_arg(trig::coth),
        "asech" => one_arg(trig::asech),
        "acsch" => one_arg(trig::acsch),
        "acoth" => one_arg(trig::acoth),

        "harmonic" => {
            expect_args(&lower, args, 1)?;
            stats::harmonic(args[0])
        }
        "binom" => {
            expect_args(&lower, args, 2)?;
            stats::binom(args[0], args[1])
        }
        "factorial" => one_arg(combinatorics::factorial),
        "perm" => {
            expect_args(&lower, args, 2)?;
            combinatorics::perm(args[0], args[1])
        }
        "catalan" => one_arg(combinatorics::catalan),

        "abs" => one_arg(general::abs),
        "trunc" | "intg" => one_arg(general::trunc),
        "frac" => one_arg(general::frac),
        "round" => one_arg(general::round),
        "ceil" => one_arg(general::ceil),
        "floor" => one_arg(general::floor),
        "random" => one_arg(general::random),
        "log2" => one_arg(general::log2),
        "cbrt" => one_arg(general::cbrt),
        "sign" => one_arg(general::sign),
        "hypot" => {
            expect_args(&lower, args, 2)?;
            general::hypot(args[0], args[1])
        }
        "clamp" => {
            expect_args(&lower, args, 3)?;
            general::clamp(args[0], args[1], args[2])
        }
        "lerp" => {
            expect_args(&lower, args, 3)?;
            general::lerp(args[0], args[1], args[2])
        }
        "roundto" => {
            expect_args(&lower, args, 2)?;
            general::roundto(args[0], args[1])
        }
        "floordiv" => {
            expect_args(&lower, args, 2)?;
            general::floordiv(args[0], args[1])
        }
        "mod2" => {
            expect_args(&lower, args, 2)?;
            general::mod2(args[0], args[1])
        }

        "fib" | "fibonacci" => one_arg(numbertheory::fib),
        "lucas" => one_arg(numbertheory::lucas),
        "prime?" => one_arg(numbertheory::isprime),
        "moebius" => one_arg(numbertheory::moebius),
        "mersenne" => one_arg(numbertheory::mersenne),
        "perfect" => one_arg(numbertheory::perfect),
        "fermat" => one_arg(numbertheory::fermat),
        "safeprime" => one_arg(numbertheory::safeprime),
        "primec" => one_arg(numbertheory::primec),
        "primen" => one_arg(numbertheory::primen),
        "mersennegen" => one_arg(numbertheory::mersennegen),
        "mersgen" => one_arg(numbertheory::mersgen),
        "genmers" => one_arg(numbertheory::genmers),
        "tau" => one_arg(numbertheory::tau),
        "phi" | "eind" => one_arg(numbertheory::phi),
        "sigma" => {
            expect_args(&lower, args, 2)?;
            numbertheory::sigma(args[0], args[1])
        }
        "primorial" => one_arg(numbertheory::primorial),
        "digitsum" => one_arg(numbertheory::digitsum),
        "digitalroot" => one_arg(numbertheory::digitalroot),
        "palindrome?" => one_arg(numbertheory::ispalindrome),
        "nextprime" => one_arg(numbertheory::nextprime),
        "triangular" => one_arg(numbertheory::triangular),
        "pentagonal" => one_arg(numbertheory::pentagonal),
        "hexagonal" => one_arg(numbertheory::hexagonal),
        "carmichael" => one_arg(numbertheory::carmichael),
        "aliquot" => one_arg(numbertheory::aliquot),
        "amicable?" => one_arg(numbertheory::isamicable),
        "coprime?" => {
            expect_args(&lower, args, 2)?;
            numbertheory::iscoprime(args[0], args[1])
        }
        "order" => {
            expect_args(&lower, args, 2)?;
            numbertheory::order(args[0], args[1])
        }
        "jacobi" => {
            expect_args(&lower, args, 2)?;
            numbertheory::jacobi(args[0], args[1])
        }

        "triarea" => {
            expect_args(&lower, args, 3)?;
            geometry::triarea(args[0], args[1], args[2])
        }
        "circlearea" => one_arg(geometry::circlearea),
        "circumference" => one_arg(geometry::circumference),
        "spherevol" => one_arg(geometry::spherevol),
        "spherearea" => one_arg(geometry::spherearea),

        "not" => one_arg(logic::not),
        "shl" => {
            expect_args(&lower, args, 2)?;
            logic::shl(args[0], args[1])
        }
        "shr" => {
            expect_args(&lower, args, 2)?;
            logic::shr(args[0], args[1])
        }
        "popcount" => one_arg(logic::popcount),
        "bitlen" => one_arg(logic::bitlen),
        "bitreverse" => {
            expect_args(&lower, args, 2)?;
            logic::bitreverse(args[0], args[1])
        }

        "gamma" => one_arg(advanced::gamma),
        "beta" => {
            expect_args(&lower, args, 2)?;
            advanced::beta(args[0], args[1])
        }
        "pochhammer" => {
            expect_args(&lower, args, 2)?;
            advanced::pochhammer(args[0], args[1])
        }
        "bth" => {
            expect_args(&lower, args, 3)?;
            advanced::bth(args[0], args[1], args[2])
        }
        "bman" => {
            expect_args(&lower, args, 3)?;
            advanced::bman(args[0], args[1], args[2])
        }
        "elliptice" => match args.len() {
            1 => advanced::elliptic_e(args[0], 1.0),
            2 => advanced::elliptic_e(args[0], args[1]),
            got => Err(CalcError::WrongArgCount {
                name: lower.clone(),
                expected: "1 or 2".to_string(),
                got,
            }),
        },
        "ellipticf" | "elliptick" => match args.len() {
            1 => advanced::elliptic_f(args[0], 1.0),
            2 => advanced::elliptic_f(args[0], args[1]),
            got => Err(CalcError::WrongArgCount {
                name: lower.clone(),
                expected: "1 or 2".to_string(),
                got,
            }),
        },
        "ellipticce" => one_arg(advanced::elliptic_ce),
        "ellipticck" => one_arg(advanced::elliptic_ck),
        "dilog" => one_arg(advanced::dilog),
        "dawson" => one_arg(advanced::dawson),
        "erf" => one_arg(advanced::erf),
        "erfc" => one_arg(advanced::erfc),
        "si" => one_arg(advanced::si),
        "ssi" => one_arg(advanced::ssi),
        "ci" => one_arg(advanced::ci),
        "chi" => one_arg(advanced::chi),
        "fresnelc" => one_arg(advanced::fresnel_c),
        "fresnels" => one_arg(advanced::fresnel_s),
        "fresnelf" => one_arg(advanced::fresnel_f),
        "fresnelg" => one_arg(advanced::fresnel_g),

        "sln" => {
            expect_args(&lower, args, 3)?;
            financial::sln(args[0], args[1], args[2])
        }
        "syd" => {
            expect_args(&lower, args, 4)?;
            financial::syd(args[0], args[1], args[2], args[3])
        }
        "cterm" => {
            expect_args(&lower, args, 3)?;
            financial::cterm(args[0], args[1], args[2])
        }
        "term" => {
            expect_args(&lower, args, 3)?;
            financial::term(args[0], args[1], args[2])
        }
        "pmt" => {
            expect_args(&lower, args, 3)?;
            financial::pmt(args[0], args[1], args[2])
        }
        "rate" => {
            expect_args(&lower, args, 3)?;
            financial::rate(args[0], args[1], args[2])
        }
        "pv" => {
            expect_args(&lower, args, 3)?;
            financial::pv(args[0], args[1], args[2])
        }
        "npv" => financial::npv(args),
        "fv" => {
            expect_args(&lower, args, 3)?;
            financial::fv(args[0], args[1], args[2])
        }
        "ddb" => {
            expect_args(&lower, args, 4)?;
            financial::ddb(args[0], args[1], args[2], args[3])
        }
        "db" => match args.len() {
            4 => financial::db(args[0], args[1], args[2], args[3], 12.0),
            5 => financial::db(args[0], args[1], args[2], args[3], args[4]),
            got => Err(CalcError::WrongArgCount {
                name: lower.clone(),
                expected: "4 or 5".to_string(),
                got,
            }),
        },
        "irate" => {
            expect_args(&lower, args, 5)?;
            financial::irate(args[0], args[1], args[2], args[3], args[4])
        }
        "nper" => {
            expect_args(&lower, args, 5)?;
            financial::nper(args[0], args[1], args[2], args[3], args[4])
        }
        "paymt" => {
            expect_args(&lower, args, 5)?;
            financial::paymt(args[0], args[1], args[2], args[3], args[4])
        }
        "fval" => {
            expect_args(&lower, args, 5)?;
            financial::fval(args[0], args[1], args[2], args[3], args[4])
        }
        "pval" => {
            expect_args(&lower, args, 5)?;
            financial::pval(args[0], args[1], args[2], args[3], args[4])
        }
        "ipaymt" => {
            expect_args(&lower, args, 6)?;
            financial::ipaymt(args[0], args[1], args[2], args[3], args[4], args[5])
        }
        "ppaymt" => {
            expect_args(&lower, args, 6)?;
            financial::ppaymt(args[0], args[1], args[2], args[3], args[4], args[5])
        }

        _ => Err(CalcError::UnknownFunction(name.to_string())),
    }
}

fn expect_args(name: &str, args: &[f64], n: usize) -> CalcResult<()> {
    if args.len() == n {
        Ok(())
    } else {
        Err(CalcError::WrongArgCount {
            name: name.to_string(),
            expected: n.to_string(),
            got: args.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;

    #[test]
    fn context_set_and_resolve_variable() {
        let mut ctx = Context::new();
        ctx.set("x", 5.0);
        assert_eq!(resolve_variable("x", &ctx).unwrap(), 5.0);
    }

    #[test]
    fn resolve_unknown_variable_errors() {
        let ctx = Context::new();
        assert_eq!(
            resolve_variable("nope", &ctx),
            Err(CalcError::UnknownVariable("nope".to_string()))
        );
    }

    #[test]
    fn eval_variable_expr_through_context() {
        let mut ctx = Context::new();
        ctx.set("x", 42.0);
        assert_eq!(
            eval(&Expr::Variable("x".to_string()), &ctx).unwrap(),
            Value::Number(42.0)
        );
    }

    #[test]
    fn eval_program_assignment_visible_to_later_statements() {
        let stmts = crate::parser::parse_program("x := 5; y := x^2 + 1; y").unwrap();
        let mut ctx = Context::new();
        assert_eq!(eval_program(&stmts, &mut ctx).unwrap(), Value::Number(26.0));
        assert_eq!(ctx.variables.get("x"), Some(&5.0));
        assert_eq!(ctx.variables.get("y"), Some(&26.0));
    }

    #[test]
    fn eval_program_trailing_assignment_returns_assigned_value() {
        let stmts = crate::parser::parse_program("x := 5").unwrap();
        let mut ctx = Context::new();
        assert_eq!(eval_program(&stmts, &mut ctx).unwrap(), Value::Number(5.0));
    }

    #[test]
    fn eval_program_rejects_assigning_to_reserved_constants() {
        for name in ["pi", "PI", "e", "E"] {
            let stmts = vec![Stmt::Assign(name.to_string(), Expr::Number(1.0))];
            let mut ctx = Context::new();
            assert_eq!(
                eval_program(&stmts, &mut ctx),
                Err(CalcError::ReservedIdentifier(name.to_string()))
            );
        }
    }

    #[test]
    fn eval_program_single_expression_still_works() {
        let stmts = crate::parser::parse_program("2 + 2").unwrap();
        let mut ctx = Context::new();
        assert_eq!(eval_program(&stmts, &mut ctx).unwrap(), Value::Number(4.0));
    }

    #[test]
    fn factorial_domain_and_overflow() {
        assert_eq!(factorial(5.0).unwrap(), 120.0);
        assert_eq!(
            factorial(-1.0),
            Err(CalcError::DomainError("factorial".to_string()))
        );
        assert_eq!(
            factorial(2.5),
            Err(CalcError::DomainError("factorial".to_string()))
        );
        assert_eq!(factorial(2000.0), Err(CalcError::Overflow));
    }

    #[test]
    fn binary_division_and_mod_by_zero() {
        assert_eq!(
            eval_binary(BinaryOp::Div, 1.0, 0.0),
            Err(CalcError::DivisionByZero)
        );
        assert_eq!(
            eval_binary(BinaryOp::Mod, 1.0, 0.0),
            Err(CalcError::DivisionByZero)
        );
        assert_eq!(eval_binary(BinaryOp::Mod, 7.0, 2.0).unwrap(), 1.0);
        assert_eq!(eval_binary(BinaryOp::Pow, 2.0, 3.0).unwrap(), 8.0);
    }

    #[test]
    fn binary_root_errors_and_ok() {
        assert_eq!(
            eval_binary(BinaryOp::Root, 0.0, 8.0),
            Err(CalcError::DivisionByZero)
        );
        assert_eq!(
            eval_binary(BinaryOp::Root, 2.0, -8.0),
            Err(CalcError::DomainError("root".to_string()))
        );
        assert_eq!(eval_binary(BinaryOp::Root, 3.0, 8.0).unwrap(), 2.0);
        assert_eq!(eval_binary(BinaryOp::Root, 1.0, -8.0).unwrap(), -8.0);
    }

    #[test]
    fn log_family_domain_errors() {
        assert_eq!(
            call_function("ln", &[0.0]),
            Err(CalcError::DomainError("ln".to_string()))
        );
        assert_eq!(
            call_function("log", &[-1.0]),
            Err(CalcError::DomainError("log".to_string()))
        );
        assert_eq!(
            call_function("logn", &[0.0, 2.0]),
            Err(CalcError::DomainError("logn".to_string()))
        );
        assert_eq!(
            call_function("logn", &[2.0, 0.0]),
            Err(CalcError::DomainError("logn".to_string()))
        );
        assert_eq!(
            call_function("logn", &[2.0, 1.0]),
            Err(CalcError::DomainError("logn".to_string()))
        );
        assert_eq!(call_function("logn", &[8.0, 2.0]).unwrap(), 3.0);
    }

    #[test]
    fn wrong_arg_count_error() {
        assert_eq!(
            call_function("sqrt", &[1.0, 2.0]),
            Err(CalcError::WrongArgCount {
                name: "sqrt".to_string(),
                expected: "1".to_string(),
                got: 2,
            })
        );
    }

    #[test]
    fn comparison_operators() {
        assert_eq!(eval_binary(BinaryOp::Eq, 2.0, 2.0).unwrap(), 1.0);
        assert_eq!(eval_binary(BinaryOp::Eq, 2.0, 3.0).unwrap(), 0.0);
        assert_eq!(eval_binary(BinaryOp::Gt, 3.0, 2.0).unwrap(), 1.0);
        assert_eq!(eval_binary(BinaryOp::Gt, 2.0, 3.0).unwrap(), 0.0);
        assert_eq!(eval_binary(BinaryOp::Lt, 2.0, 3.0).unwrap(), 1.0);
        assert_eq!(eval_binary(BinaryOp::Lt, 3.0, 2.0).unwrap(), 0.0);
    }

    #[test]
    fn bitwise_logical_operators_match_manual_examples() {
        assert_eq!(eval_binary(BinaryOp::Xor, 2.0, 4.0).unwrap(), 6.0);
        assert_eq!(eval_binary(BinaryOp::Xnor, 2.0, 4.0).unwrap(), -7.0);
        assert_eq!(eval_binary(BinaryOp::And, 3.0, 9.0).unwrap(), 1.0);
        assert_eq!(eval_binary(BinaryOp::Nand, 3.0, 9.0).unwrap(), -2.0);
        assert_eq!(eval_binary(BinaryOp::Or, 2.0, 4.0).unwrap(), 6.0);
        assert_eq!(eval_binary(BinaryOp::Nor, 2.0, 4.0).unwrap(), -7.0);
    }

    #[test]
    fn bitwise_operators_domain_error_out_of_i64_range() {
        assert_eq!(
            eval_binary(BinaryOp::And, 1e30, 1.0),
            Err(CalcError::DomainError("and".to_string()))
        );
        assert_eq!(
            eval_binary(BinaryOp::And, 1.0, 1e30),
            Err(CalcError::DomainError("and".to_string()))
        );
    }

    #[test]
    fn not_shl_shr_functions() {
        assert_eq!(call_function("not", &[1.0]).unwrap(), -2.0);
        assert_eq!(call_function("shl", &[2.0, 1.0]).unwrap(), 4.0);
        assert_eq!(call_function("shr", &[2.0, 1.0]).unwrap(), 1.0);
    }

    #[test]
    fn advanced_functions_dispatch() {
        assert!(
            (call_function("gamma", &[0.5]).unwrap() - std::f64::consts::PI.sqrt()).abs() < 1e-3
        );
        assert!((call_function("beta", &[1.0, 2.0]).unwrap() - 0.5).abs() < 1e-3);
        assert!((call_function("pochhammer", &[5.0, 3.0]).unwrap() - 210.0).abs() < 1e-2);
        assert_eq!(call_function("bth", &[2.0, 3.0, 4.0]).unwrap(), 625.0);
        assert_eq!(call_function("bman", &[2.0, 3.0, 4.0]).unwrap(), 625.0);
        assert!((call_function("elliptice", &[1.0]).unwrap() - 1.0).abs() < 1e-6);
        assert!((call_function("elliptice", &[1.0, 1.0]).unwrap() - 1.0).abs() < 1e-6);
        assert!(call_function("ellipticf", &[0.01]).unwrap() > 0.0);
        assert!(call_function("ellipticf", &[0.01, 1.0]).unwrap() > 0.0);
        assert!(call_function("elliptick", &[0.01]).unwrap() > 0.0);
        assert!(call_function("ellipticce", &[1.0]).unwrap() > 0.0);
        assert!(call_function("ellipticck", &[1.0]).unwrap() > 0.0);
        assert_eq!(call_function("dilog", &[1.0]).unwrap(), 0.0);
        assert_eq!(call_function("dawson", &[0.0]).unwrap(), 0.0);
        assert_eq!(call_function("erf", &[0.0]).unwrap(), 0.0);
        assert_eq!(call_function("erfc", &[0.0]).unwrap(), 1.0);
        assert_eq!(call_function("si", &[0.0]).unwrap(), 0.0);
        assert!((call_function("ssi", &[0.0]).unwrap() + std::f64::consts::FRAC_PI_2).abs() < 1e-9);
        assert!(call_function("ci", &[1.0]).unwrap().is_finite());
        assert!(call_function("chi", &[1.0]).unwrap().is_finite());
        assert_eq!(call_function("fresnelc", &[0.0]).unwrap(), 0.0);
        assert_eq!(call_function("fresnels", &[0.0]).unwrap(), 0.0);
        assert!(call_function("fresnelf", &[1.0]).unwrap().is_finite());
        assert!(call_function("fresnelg", &[1.0]).unwrap().is_finite());
    }

    #[test]
    fn elliptic_wrong_arg_count() {
        assert_eq!(
            call_function("elliptice", &[]),
            Err(CalcError::WrongArgCount {
                name: "elliptice".to_string(),
                expected: "1 or 2".to_string(),
                got: 0,
            })
        );
        assert_eq!(
            call_function("elliptice", &[1.0, 2.0, 3.0]),
            Err(CalcError::WrongArgCount {
                name: "elliptice".to_string(),
                expected: "1 or 2".to_string(),
                got: 3,
            })
        );
        assert_eq!(
            call_function("ellipticf", &[]),
            Err(CalcError::WrongArgCount {
                name: "ellipticf".to_string(),
                expected: "1 or 2".to_string(),
                got: 0,
            })
        );
        assert_eq!(
            call_function("ellipticf", &[1.0, 2.0, 3.0]),
            Err(CalcError::WrongArgCount {
                name: "ellipticf".to_string(),
                expected: "1 or 2".to_string(),
                got: 3,
            })
        );
    }

    #[test]
    fn financial_functions_dispatch() {
        assert_eq!(
            call_function("sln", &[10000.0, 1000.0, 5.0]).unwrap(),
            1800.0
        );
        assert_eq!(
            call_function("syd", &[10000.0, 1000.0, 5.0, 1.0]).unwrap(),
            3000.0
        );
        assert!((call_function("cterm", &[0.1, 2000.0, 1000.0]).unwrap() - 7.27254).abs() < 1e-3);
        assert!((call_function("term", &[100.0, 0.01, 5000.0]).unwrap() - 40.7489).abs() < 1e-3);
        assert!((call_function("pmt", &[10000.0, 0.08, 5.0]).unwrap() - 2504.5645).abs() < 1e-3);
        assert!((call_function("rate", &[2000.0, 1000.0, 10.0]).unwrap() - 0.0717735).abs() < 1e-5);
        assert!((call_function("pv", &[1000.0, 0.08, 5.0]).unwrap() - 3992.71).abs() < 1e-1);
        assert!((call_function("fv", &[1000.0, 0.08, 5.0]).unwrap() - 5866.6).abs() < 1e-1);
        assert!(
            (call_function("npv", &[0.1, 100.0, 200.0, 300.0]).unwrap() - 481.5928).abs() < 1e-3
        );
        assert_eq!(
            call_function("ddb", &[10000.0, 1000.0, 5.0, 1.0]).unwrap(),
            4000.0
        );
        assert!(
            (call_function("db", &[50000.0, 10000.0, 5.0, 1.0]).unwrap() - 13761.0168).abs() < 1e-2
        );
        assert!(
            (call_function("db", &[50000.0, 10000.0, 5.0, 1.0, 3.0]).unwrap() - 3440.254).abs()
                < 1e-2
        );
        assert!(call_function("irate", &[5.0, 100.0, -1000.0, 0.0, 0.0])
            .unwrap()
            .is_finite());
        assert!(
            call_function("nper", &[0.08, 2504.5645, -10000.0, 0.0, 0.0])
                .unwrap()
                .is_finite()
        );
        assert!(call_function("paymt", &[0.08, 5.0, -10000.0, 0.0, 0.0])
            .unwrap()
            .is_finite());
        assert!(
            call_function("fval", &[0.08, 5.0, 2504.5645, -10000.0, 0.0])
                .unwrap()
                .is_finite()
        );
        assert!(call_function("pval", &[0.08, 5.0, 2504.5645, 0.0, 0.0])
            .unwrap()
            .is_finite());
        assert!(
            call_function("ipaymt", &[0.08, 1.0, 5.0, -10000.0, 0.0, 0.0])
                .unwrap()
                .is_finite()
        );
        assert!(
            call_function("ppaymt", &[0.08, 1.0, 5.0, -10000.0, 0.0, 0.0])
                .unwrap()
                .is_finite()
        );
    }

    #[test]
    fn db_wrong_arg_count() {
        assert_eq!(
            call_function("db", &[1.0, 2.0, 3.0]),
            Err(CalcError::WrongArgCount {
                name: "db".to_string(),
                expected: "4 or 5".to_string(),
                got: 3,
            })
        );
        assert_eq!(
            call_function("db", &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
            Err(CalcError::WrongArgCount {
                name: "db".to_string(),
                expected: "4 or 5".to_string(),
                got: 6,
            })
        );
    }

    #[test]
    fn integration_composite_rules_end_to_end() {
        for (expr, expected) in [
            ("simpson(x^2, x, 0, 3, 100)", 9.0),
            ("trapez(x^2, x, 0, 3, 2000)", 9.0),
            ("trapezoide(x^2, x, 0, 3, 2000)", 9.0),
            ("newton(x^2, x, 0, 3, 100)", 9.0),
            ("boole(x^2, x, 0, 3, 100)", 9.0),
            ("ordersix(x^2, x, 0, 3, 100)", 9.0),
            ("ordresix(x^2, x, 0, 3, 100)", 9.0),
            ("weddle(x^2, x, 0, 3, 100)", 9.0),
        ] {
            let result = crate::evaluate(expr).unwrap();
            assert!((result - expected).abs() < 1e-2, "{expr} => {result}");
        }
    }

    #[test]
    fn integration_adaptive_rules_end_to_end() {
        let result = crate::evaluate("int(x^2, x, 0, 3, 0.000001)").unwrap();
        assert!((result - 9.0).abs() < 1e-4);

        let result = crate::evaluate("gauss(sin(x), x, 0, pi, 0.000001)").unwrap();
        assert!((result - 2.0).abs() < 1e-4);
    }

    #[test]
    fn integration_reversed_bounds_negates_result() {
        let result = crate::evaluate("simpson(x^2, x, 3, 0, 100)").unwrap();
        assert!((result - -9.0).abs() < 1e-2);
    }

    #[test]
    fn integration_does_not_leak_or_shadow_outer_context() {
        // The integration variable is scoped to the call; it shouldn't
        // affect (or be affected by) a same-named variable elsewhere.
        let mut ctx = Context::new();
        ctx.set("k", 2.0);
        let expr = crate::parser::parse("simpson(k * x, x, 0, 2, 100) + k").unwrap();
        let result = match eval(&expr, &ctx).unwrap() {
            Value::Number(n) => n,
            Value::Text(_) => panic!("expected a number"),
        };
        // integral of 2*x from 0 to 2 = 4, plus outer k (2) = 6.
        assert!((result - 6.0).abs() < 1e-2);
    }

    #[test]
    fn integration_wrong_arg_count() {
        assert_eq!(
            crate::evaluate("simpson(x^2, x, 0, 3)"),
            Err(CalcError::WrongArgCount {
                name: "simpson".to_string(),
                expected: "5".to_string(),
                got: 4,
            })
        );
        assert_eq!(
            crate::evaluate("int(x^2, x, 0, 3)"),
            Err(CalcError::WrongArgCount {
                name: "int".to_string(),
                expected: "5".to_string(),
                got: 4,
            })
        );
    }

    #[test]
    fn integration_invalid_variable_argument() {
        // Second argument must be a bare, non-reserved variable.
        assert_eq!(
            crate::evaluate("simpson(x^2, 5, 0, 3, 100)"),
            Err(CalcError::DomainError("simpson".to_string()))
        );
        assert_eq!(
            crate::evaluate("simpson(x^2, pi, 0, 3, 100)"),
            Err(CalcError::DomainError("simpson".to_string()))
        );
        assert_eq!(
            crate::evaluate("simpson(x^2, e, 0, 3, 100)"),
            Err(CalcError::DomainError("simpson".to_string()))
        );
        assert_eq!(
            crate::evaluate("simpson(x^2, x + 1, 0, 3, 100)"),
            Err(CalcError::DomainError("simpson".to_string()))
        );
    }

    #[test]
    fn integration_non_whole_or_non_positive_n() {
        assert_eq!(
            crate::evaluate("simpson(x^2, x, 0, 3, 2.5)"),
            Err(CalcError::DomainError("simpson".to_string()))
        );
        assert_eq!(
            crate::evaluate("simpson(x^2, x, 0, 3, 0)"),
            Err(CalcError::DomainError("simpson".to_string()))
        );
    }

    #[test]
    fn integration_non_positive_tolerance() {
        assert_eq!(
            crate::evaluate("int(x^2, x, 0, 3, 0)"),
            Err(CalcError::DomainError("int".to_string()))
        );
        assert_eq!(
            crate::evaluate("int(x^2, x, 0, 3, -1)"),
            Err(CalcError::DomainError("int".to_string()))
        );
    }

    #[test]
    fn integration_propagates_body_errors() {
        assert_eq!(
            crate::evaluate("simpson(sqrt(x), x, -1, 1, 4)"),
            Err(CalcError::DomainError("sqrt".to_string()))
        );
    }

    #[test]
    fn root_finding_finds_sqrt_2() {
        let bisect = crate::evaluate("bisect(x^2 - 2, x, 0, 2, 0.000001)").unwrap();
        assert!((bisect - std::f64::consts::SQRT_2).abs() < 1e-4);
        let secant = crate::evaluate("secant(x^2 - 2, x, 0, 2, 0.000001)").unwrap();
        assert!((secant - std::f64::consts::SQRT_2).abs() < 1e-4);
    }

    #[test]
    fn root_finding_reuses_integration_arg_validation() {
        // Wrong arity and invalid variable argument are validated by the
        // shared `integration_setup`, exercised more thoroughly by the
        // `integration_*` tests above; spot-check that bisect/secant go
        // through the same path.
        assert_eq!(
            crate::evaluate("bisect(x^2 - 2, x, 0, 2)"),
            Err(CalcError::WrongArgCount {
                name: "bisect".to_string(),
                expected: "5".to_string(),
                got: 4,
            })
        );
        assert_eq!(
            crate::evaluate("secant(x^2 - 2, 5, 0, 2, 0.001)"),
            Err(CalcError::DomainError("secant".to_string()))
        );
    }

    #[test]
    fn root_finding_does_not_converge_errors() {
        assert_eq!(
            crate::evaluate("bisect(x^2 + 1, x, -2, 2, 0.001)"),
            Err(CalcError::DomainError("bisect".to_string()))
        );
    }

    #[test]
    fn integration_does_not_converge_overflows() {
        assert_eq!(
            crate::evaluate("int(sin(x * 100000000), x, 0, 1, 0.000000000000001)"),
            Err(CalcError::Overflow)
        );
    }
}
