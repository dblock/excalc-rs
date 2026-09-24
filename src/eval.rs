//! Evaluates an `Expr` AST into a final `f64` value.

use std::collections::HashMap;

use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::error::{CalcError, CalcResult};
use crate::functions::{stats, trig};

/// Evaluation context: currently just variable bindings. Constants (`pi`,
/// `e`) are always available and can't be shadowed in v1.
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

pub fn eval(expr: &Expr, ctx: &Context) -> CalcResult<f64> {
    match expr {
        Expr::Number(n) => Ok(*n),

        Expr::Variable(name) => resolve_variable(name, ctx),

        Expr::Unary(op, inner) => {
            let v = eval(inner, ctx)?;
            eval_unary(*op, v)
        }

        Expr::Binary(op, lhs, rhs) => {
            let l = eval(lhs, ctx)?;
            let r = eval(rhs, ctx)?;
            eval_binary(*op, l, r)
        }

        Expr::Call(name, arg_exprs) => {
            let mut args = Vec::with_capacity(arg_exprs.len());
            for a in arg_exprs {
                args.push(eval(a, ctx)?);
            }
            call_function(name, &args)
        }
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
    }
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
