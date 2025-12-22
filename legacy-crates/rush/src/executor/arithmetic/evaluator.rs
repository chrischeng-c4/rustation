//! Evaluator for arithmetic expressions.
//!
//! Evaluates an AST with variable resolution via a callback.

use super::parser::Expr;
use super::{ArithmeticError, Result};

/// Context for variable resolution during evaluation.
pub trait VariableContext {
    /// Get the value of a variable, returning 0 if not found.
    fn get(&self, name: &str) -> i64;

    /// Set the value of a variable.
    fn set(&mut self, name: &str, value: i64);
}

/// Evaluate an arithmetic expression with the given variable context.
pub fn evaluate<C: VariableContext>(expr: &Expr, ctx: &mut C) -> Result<i64> {
    match expr {
        Expr::Number(n) => Ok(*n),

        Expr::Variable(name) => Ok(ctx.get(name)),

        // Unary operators
        Expr::Negate(e) => Ok(-evaluate(e, ctx)?),
        Expr::Positive(e) => evaluate(e, ctx),
        Expr::LogicalNot(e) => Ok(if evaluate(e, ctx)? == 0 { 1 } else { 0 }),
        Expr::BitwiseNot(e) => Ok(!evaluate(e, ctx)?),

        // Binary arithmetic operators
        Expr::Add(l, r) => Ok(evaluate(l, ctx)?.wrapping_add(evaluate(r, ctx)?)),
        Expr::Sub(l, r) => Ok(evaluate(l, ctx)?.wrapping_sub(evaluate(r, ctx)?)),
        Expr::Mul(l, r) => Ok(evaluate(l, ctx)?.wrapping_mul(evaluate(r, ctx)?)),
        Expr::Div(l, r) => {
            let right = evaluate(r, ctx)?;
            if right == 0 {
                return Err(ArithmeticError::DivisionByZero);
            }
            Ok(evaluate(l, ctx)? / right)
        }
        Expr::Mod(l, r) => {
            let right = evaluate(r, ctx)?;
            if right == 0 {
                return Err(ArithmeticError::DivisionByZero);
            }
            Ok(evaluate(l, ctx)? % right)
        }
        Expr::Pow(l, r) => {
            let base = evaluate(l, ctx)?;
            let exp = evaluate(r, ctx)?;
            if exp < 0 {
                // Integer exponentiation with negative exponent
                // In bash, this is an error; we'll return 0 for simplicity
                Ok(0)
            } else {
                Ok(base.wrapping_pow(exp as u32))
            }
        }

        // Comparison operators (return 1 for true, 0 for false)
        Expr::Lt(l, r) => Ok(if evaluate(l, ctx)? < evaluate(r, ctx)? {
            1
        } else {
            0
        }),
        Expr::Gt(l, r) => Ok(if evaluate(l, ctx)? > evaluate(r, ctx)? {
            1
        } else {
            0
        }),
        Expr::Le(l, r) => Ok(if evaluate(l, ctx)? <= evaluate(r, ctx)? {
            1
        } else {
            0
        }),
        Expr::Ge(l, r) => Ok(if evaluate(l, ctx)? >= evaluate(r, ctx)? {
            1
        } else {
            0
        }),
        Expr::Eq(l, r) => Ok(if evaluate(l, ctx)? == evaluate(r, ctx)? {
            1
        } else {
            0
        }),
        Expr::Ne(l, r) => Ok(if evaluate(l, ctx)? != evaluate(r, ctx)? {
            1
        } else {
            0
        }),

        // Logical operators (short-circuit)
        Expr::And(l, r) => {
            let left = evaluate(l, ctx)?;
            if left == 0 {
                Ok(0) // Short-circuit: false && anything = false
            } else {
                Ok(if evaluate(r, ctx)? != 0 { 1 } else { 0 })
            }
        }
        Expr::Or(l, r) => {
            let left = evaluate(l, ctx)?;
            if left != 0 {
                Ok(1) // Short-circuit: true || anything = true
            } else {
                Ok(if evaluate(r, ctx)? != 0 { 1 } else { 0 })
            }
        }

        // Bitwise operators
        Expr::BitAnd(l, r) => Ok(evaluate(l, ctx)? & evaluate(r, ctx)?),
        Expr::BitOr(l, r) => Ok(evaluate(l, ctx)? | evaluate(r, ctx)?),
        Expr::BitXor(l, r) => Ok(evaluate(l, ctx)? ^ evaluate(r, ctx)?),
        Expr::Shl(l, r) => {
            let shift = evaluate(r, ctx)?;
            if shift < 0 || shift >= 64 {
                Ok(0) // Undefined behavior in C, we return 0
            } else {
                Ok(evaluate(l, ctx)? << shift)
            }
        }
        Expr::Shr(l, r) => {
            let shift = evaluate(r, ctx)?;
            if shift < 0 || shift >= 64 {
                Ok(0) // Undefined behavior in C, we return 0
            } else {
                Ok(evaluate(l, ctx)? >> shift)
            }
        }

        // Assignment operators
        Expr::Assign(name, e) => {
            let value = evaluate(e, ctx)?;
            ctx.set(name, value);
            Ok(value)
        }
        Expr::AddAssign(name, e) => {
            let current = ctx.get(name);
            let delta = evaluate(e, ctx)?;
            let value = current.wrapping_add(delta);
            ctx.set(name, value);
            Ok(value)
        }
        Expr::SubAssign(name, e) => {
            let current = ctx.get(name);
            let delta = evaluate(e, ctx)?;
            let value = current.wrapping_sub(delta);
            ctx.set(name, value);
            Ok(value)
        }
        Expr::MulAssign(name, e) => {
            let current = ctx.get(name);
            let factor = evaluate(e, ctx)?;
            let value = current.wrapping_mul(factor);
            ctx.set(name, value);
            Ok(value)
        }
        Expr::DivAssign(name, e) => {
            let current = ctx.get(name);
            let divisor = evaluate(e, ctx)?;
            if divisor == 0 {
                return Err(ArithmeticError::DivisionByZero);
            }
            let value = current / divisor;
            ctx.set(name, value);
            Ok(value)
        }
        Expr::ModAssign(name, e) => {
            let current = ctx.get(name);
            let divisor = evaluate(e, ctx)?;
            if divisor == 0 {
                return Err(ArithmeticError::DivisionByZero);
            }
            let value = current % divisor;
            ctx.set(name, value);
            Ok(value)
        }
        Expr::AndAssign(name, e) => {
            let current = ctx.get(name);
            let rhs = evaluate(e, ctx)?;
            let value = current & rhs;
            ctx.set(name, value);
            Ok(value)
        }
        Expr::OrAssign(name, e) => {
            let current = ctx.get(name);
            let rhs = evaluate(e, ctx)?;
            let value = current | rhs;
            ctx.set(name, value);
            Ok(value)
        }
        Expr::XorAssign(name, e) => {
            let current = ctx.get(name);
            let rhs = evaluate(e, ctx)?;
            let value = current ^ rhs;
            ctx.set(name, value);
            Ok(value)
        }
        Expr::ShlAssign(name, e) => {
            let current = ctx.get(name);
            let shift = evaluate(e, ctx)?;
            let value = if shift < 0 || shift >= 64 {
                0
            } else {
                current << shift
            };
            ctx.set(name, value);
            Ok(value)
        }
        Expr::ShrAssign(name, e) => {
            let current = ctx.get(name);
            let shift = evaluate(e, ctx)?;
            let value = if shift < 0 || shift >= 64 {
                0
            } else {
                current >> shift
            };
            ctx.set(name, value);
            Ok(value)
        }

        // Increment/Decrement
        Expr::PreIncrement(name) => {
            let value = ctx.get(name).wrapping_add(1);
            ctx.set(name, value);
            Ok(value)
        }
        Expr::PostIncrement(name) => {
            let old = ctx.get(name);
            ctx.set(name, old.wrapping_add(1));
            Ok(old)
        }
        Expr::PreDecrement(name) => {
            let value = ctx.get(name).wrapping_sub(1);
            ctx.set(name, value);
            Ok(value)
        }
        Expr::PostDecrement(name) => {
            let old = ctx.get(name);
            ctx.set(name, old.wrapping_sub(1));
            Ok(old)
        }

        // Ternary (short-circuit)
        Expr::Ternary(cond, then_expr, else_expr) => {
            if evaluate(cond, ctx)? != 0 {
                evaluate(then_expr, ctx)
            } else {
                evaluate(else_expr, ctx)
            }
        }

        // Comma (evaluate all, return last)
        Expr::Comma(exprs) => {
            let mut result = 0;
            for e in exprs {
                result = evaluate(e, ctx)?;
            }
            Ok(result)
        }
    }
}

/// Simple variable context using a HashMap.
#[derive(Default)]
pub struct SimpleContext {
    vars: std::collections::HashMap<String, i64>,
}

impl SimpleContext {
    pub fn new() -> Self {
        Self::default()
    }
}

impl VariableContext for SimpleContext {
    fn get(&self, name: &str) -> i64 {
        *self.vars.get(name).unwrap_or(&0)
    }

    fn set(&mut self, name: &str, value: i64) {
        self.vars.insert(name.to_string(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::arithmetic::parser::parse;

    fn eval(input: &str) -> Result<i64> {
        let expr = parse(input)?;
        let mut ctx = SimpleContext::new();
        evaluate(&expr, &mut ctx)
    }

    fn eval_with_var(input: &str, name: &str, value: i64) -> Result<i64> {
        let expr = parse(input)?;
        let mut ctx = SimpleContext::new();
        ctx.set(name, value);
        evaluate(&expr, &mut ctx)
    }

    #[test]
    fn test_number() {
        assert_eq!(eval("42").unwrap(), 42);
    }

    #[test]
    fn test_negative() {
        assert_eq!(eval("-5").unwrap(), -5);
    }

    #[test]
    fn test_addition() {
        assert_eq!(eval("2 + 3").unwrap(), 5);
    }

    #[test]
    fn test_subtraction() {
        assert_eq!(eval("10 - 4").unwrap(), 6);
    }

    #[test]
    fn test_multiplication() {
        assert_eq!(eval("6 * 7").unwrap(), 42);
    }

    #[test]
    fn test_division() {
        assert_eq!(eval("17 / 5").unwrap(), 3);
    }

    #[test]
    fn test_modulo() {
        assert_eq!(eval("17 % 5").unwrap(), 2);
    }

    #[test]
    fn test_power() {
        assert_eq!(eval("2 ** 10").unwrap(), 1024);
    }

    #[test]
    fn test_precedence() {
        assert_eq!(eval("2 + 3 * 4").unwrap(), 14);
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(eval("(2 + 3) * 4").unwrap(), 20);
    }

    #[test]
    fn test_variable() {
        assert_eq!(eval_with_var("x + 5", "x", 10).unwrap(), 15);
    }

    #[test]
    fn test_undefined_variable() {
        // Undefined variables evaluate to 0
        assert_eq!(eval("undefined").unwrap(), 0);
    }

    #[test]
    fn test_comparison_gt() {
        assert_eq!(eval("5 > 3").unwrap(), 1);
        assert_eq!(eval("3 > 5").unwrap(), 0);
    }

    #[test]
    fn test_comparison_lt() {
        assert_eq!(eval("3 < 5").unwrap(), 1);
        assert_eq!(eval("5 < 3").unwrap(), 0);
    }

    #[test]
    fn test_comparison_eq() {
        assert_eq!(eval("5 == 5").unwrap(), 1);
        assert_eq!(eval("5 == 3").unwrap(), 0);
    }

    #[test]
    fn test_comparison_ne() {
        assert_eq!(eval("5 != 3").unwrap(), 1);
        assert_eq!(eval("5 != 5").unwrap(), 0);
    }

    #[test]
    fn test_logical_and() {
        assert_eq!(eval("1 && 1").unwrap(), 1);
        assert_eq!(eval("1 && 0").unwrap(), 0);
        assert_eq!(eval("0 && 1").unwrap(), 0);
    }

    #[test]
    fn test_logical_or() {
        assert_eq!(eval("1 || 0").unwrap(), 1);
        assert_eq!(eval("0 || 1").unwrap(), 1);
        assert_eq!(eval("0 || 0").unwrap(), 0);
    }

    #[test]
    fn test_logical_not() {
        assert_eq!(eval("!0").unwrap(), 1);
        assert_eq!(eval("!1").unwrap(), 0);
        assert_eq!(eval("!5").unwrap(), 0);
    }

    #[test]
    fn test_bitwise_and() {
        assert_eq!(eval("5 & 3").unwrap(), 1);
    }

    #[test]
    fn test_bitwise_or() {
        assert_eq!(eval("5 | 3").unwrap(), 7);
    }

    #[test]
    fn test_bitwise_xor() {
        assert_eq!(eval("5 ^ 3").unwrap(), 6);
    }

    #[test]
    fn test_bitwise_not() {
        assert_eq!(eval("~0").unwrap(), -1);
    }

    #[test]
    fn test_shift_left() {
        assert_eq!(eval("1 << 4").unwrap(), 16);
    }

    #[test]
    fn test_shift_right() {
        assert_eq!(eval("16 >> 2").unwrap(), 4);
    }

    #[test]
    fn test_ternary() {
        assert_eq!(eval("1 ? 10 : 20").unwrap(), 10);
        assert_eq!(eval("0 ? 10 : 20").unwrap(), 20);
    }

    #[test]
    fn test_comma() {
        assert_eq!(eval("1, 2, 3").unwrap(), 3);
    }

    #[test]
    fn test_assignment() {
        let expr = parse("x = 5").unwrap();
        let mut ctx = SimpleContext::new();
        let result = evaluate(&expr, &mut ctx).unwrap();
        assert_eq!(result, 5);
        assert_eq!(ctx.get("x"), 5);
    }

    #[test]
    fn test_add_assign() {
        let expr = parse("x += 3").unwrap();
        let mut ctx = SimpleContext::new();
        ctx.set("x", 5);
        let result = evaluate(&expr, &mut ctx).unwrap();
        assert_eq!(result, 8);
        assert_eq!(ctx.get("x"), 8);
    }

    #[test]
    fn test_pre_increment() {
        let expr = parse("++x").unwrap();
        let mut ctx = SimpleContext::new();
        ctx.set("x", 5);
        let result = evaluate(&expr, &mut ctx).unwrap();
        assert_eq!(result, 6);
        assert_eq!(ctx.get("x"), 6);
    }

    #[test]
    fn test_post_increment() {
        let expr = parse("x++").unwrap();
        let mut ctx = SimpleContext::new();
        ctx.set("x", 5);
        let result = evaluate(&expr, &mut ctx).unwrap();
        assert_eq!(result, 5); // Post-increment returns old value
        assert_eq!(ctx.get("x"), 6);
    }

    #[test]
    fn test_division_by_zero() {
        let result = eval("5 / 0");
        assert!(matches!(result, Err(ArithmeticError::DivisionByZero)));
    }

    #[test]
    fn test_modulo_by_zero() {
        let result = eval("5 % 0");
        assert!(matches!(result, Err(ArithmeticError::DivisionByZero)));
    }

    #[test]
    fn test_hex_number() {
        assert_eq!(eval("0x10").unwrap(), 16);
        assert_eq!(eval("0xFF").unwrap(), 255);
    }

    #[test]
    fn test_octal_number() {
        assert_eq!(eval("010").unwrap(), 8);
        assert_eq!(eval("0o17").unwrap(), 15);
    }

    #[test]
    fn test_empty() {
        assert_eq!(eval("").unwrap(), 0);
    }

    #[test]
    fn test_complex_expression() {
        // (2 + 3) * 4 - 5 / 2 = 5 * 4 - 2 = 20 - 2 = 18
        assert_eq!(eval("(2 + 3) * 4 - 5 / 2").unwrap(), 18);
    }
}
