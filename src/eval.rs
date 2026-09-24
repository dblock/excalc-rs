//! Evaluates an `Expr` AST into a final `f64` value.

use std::collections::HashMap;

use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::error::{CalcError, CalcResult};
use crate::functions::{general, logic, number_theory, stats, trig};

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
        "gcd" => return number_theory::gcd(args),
        "lcm" => return number_theory::lcm(args),
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

        "abs" => one_arg(general::abs),
        "trunc" | "intg" => one_arg(general::trunc),
        "frac" => one_arg(general::frac),
        "round" => one_arg(general::round),
        "ceil" => one_arg(general::ceil),
        "floor" => one_arg(general::floor),
        "random" => one_arg(general::random),

        "fib" | "fibonacci" => one_arg(number_theory::fib),
        "prime?" => one_arg(number_theory::isprime),
        "moebius" => one_arg(number_theory::moebius),
        "mersenne" => one_arg(number_theory::mersenne),
        "perfect" => one_arg(number_theory::perfect),
        "fermat" => one_arg(number_theory::fermat),
        "safeprime" => one_arg(number_theory::safeprime),
        "primec" => one_arg(number_theory::primec),
        "primen" => one_arg(number_theory::primen),
        "mersennegen" => one_arg(number_theory::mersennegen),
        "mersgen" => one_arg(number_theory::mersgen),
        "genmers" => one_arg(number_theory::genmers),
        "tau" => one_arg(number_theory::tau),
        "phi" | "eind" => one_arg(number_theory::phi),
        "sigma" => {
            expect_args(&lower, args, 2)?;
            number_theory::sigma(args[0], args[1])
        }

        "not" => one_arg(logic::not),
        "shl" => {
            expect_args(&lower, args, 2)?;
            logic::shl(args[0], args[1])
        }
        "shr" => {
            expect_args(&lower, args, 2)?;
            logic::shr(args[0], args[1])
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
        assert_eq!(eval(&Expr::Variable("x".to_string()), &ctx).unwrap(), 42.0);
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
}
