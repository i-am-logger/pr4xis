use crate::*;
use proptest::prelude::*;

fn arb_int() -> impl Strategy<Value = i64> {
    -1000..1000i64
}

fn arb_nonzero_int() -> impl Strategy<Value = i64> {
    prop_oneof![-1000..-1i64, 1..1000i64]
}

fn arb_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        arb_int().prop_map(Value::int),
        (arb_int(), arb_nonzero_int()).prop_map(|(n, d)| Value::rational(n, d).unwrap()),
    ]
}

// =============================================================================
// Value / Fraction simplification tests
// =============================================================================

#[test]
fn test_2_over_4_simplifies_to_1_over_2() {
    let v = Value::rational(2, 4).unwrap();
    assert_eq!(v, Value::Rational(1, 2));
}

#[test]
fn test_6_over_9_simplifies_to_2_over_3() {
    let v = Value::rational(6, 9).unwrap();
    assert_eq!(v, Value::Rational(2, 3));
}

#[test]
fn test_negative_denominator_normalized() {
    let v = Value::rational(3, -6).unwrap();
    assert_eq!(v, Value::Rational(-1, 2));
}

#[test]
fn test_zero_numerator() {
    let v = Value::rational(0, 5).unwrap();
    assert_eq!(v, Value::Rational(0, 1));
}

#[test]
fn test_division_by_zero_rejected() {
    assert!(Value::rational(1, 0).is_err());
}

#[test]
fn test_integer_display() {
    assert_eq!(Value::int(42).to_string(), "42");
}

#[test]
fn test_fraction_display() {
    assert_eq!(Value::rational(3, 4).unwrap().to_string(), "3/4");
}

// =============================================================================
// Arithmetic tests
// =============================================================================

#[test]
fn test_add_fractions() {
    let a = Value::rational(1, 3).unwrap();
    let b = Value::rational(1, 6).unwrap();
    let result = BinaryOp::Add.apply(&a, &b).unwrap();
    assert_eq!(result, Value::Rational(1, 2));
}

#[test]
fn test_multiply_fractions() {
    let a = Value::rational(2, 3).unwrap();
    let b = Value::rational(3, 4).unwrap();
    let result = BinaryOp::Multiply.apply(&a, &b).unwrap();
    assert_eq!(result, Value::Rational(1, 2));
}

#[test]
fn test_divide_fractions() {
    let a = Value::rational(1, 2).unwrap();
    let b = Value::rational(1, 4).unwrap();
    let result = BinaryOp::Divide.apply(&a, &b).unwrap();
    assert_eq!(result, Value::Rational(2, 1));
}

#[test]
fn test_divide_by_zero() {
    let result = BinaryOp::Divide.apply(&Value::int(5), &Value::int(0));
    assert_eq!(result, Err(CalcError::DivisionByZero));
}

// =============================================================================
// Domain enforcement tests
// =============================================================================

#[test]
fn test_sqrt_negative_rejected() {
    let result = UnaryOp::Sqrt.apply(&Value::int(-4), AngleMode::Radians);
    assert_eq!(result, Err(CalcError::NegativeSquareRoot));
}

#[test]
fn test_sqrt_exact_rational() {
    let result = UnaryOp::Sqrt
        .apply(&Value::rational(9, 4).unwrap(), AngleMode::Radians)
        .unwrap();
    assert_eq!(result, Value::Rational(3, 2));
}

#[test]
fn test_ln_zero_rejected() {
    let result = UnaryOp::Ln.apply(&Value::int(0), AngleMode::Radians);
    assert_eq!(result, Err(CalcError::LogOfNonPositive));
}

#[test]
fn test_ln_negative_rejected() {
    let result = UnaryOp::Ln.apply(&Value::int(-5), AngleMode::Radians);
    assert_eq!(result, Err(CalcError::LogOfNonPositive));
}

#[test]
fn test_factorial() {
    let result = UnaryOp::Factorial
        .apply(&Value::int(5), AngleMode::Radians)
        .unwrap();
    assert_eq!(result, Value::int(120));
}

#[test]
fn test_factorial_zero() {
    let result = UnaryOp::Factorial
        .apply(&Value::int(0), AngleMode::Radians)
        .unwrap();
    assert_eq!(result, Value::int(1));
}

#[test]
fn test_factorial_negative_rejected() {
    let result = UnaryOp::Factorial.apply(&Value::int(-3), AngleMode::Radians);
    assert!(result.is_err());
}

#[test]
fn test_factorial_overflow() {
    let result = UnaryOp::Factorial.apply(&Value::int(21), AngleMode::Radians);
    assert_eq!(result, Err(CalcError::Overflow));
}

#[test]
fn test_asin_out_of_range() {
    let result = UnaryOp::Asin.apply(&Value::Float(1.5), AngleMode::Radians);
    assert!(result.is_err());
}

// =============================================================================
// Expression simplification tests
// =============================================================================

#[test]
fn test_constant_folding() {
    let expr = Expr::binary(BinaryOp::Add, Expr::int(2), Expr::int(3));
    let simplified = expr.simplify();
    assert_eq!(simplified, Expr::Lit(Value::int(5)));
}

#[test]
fn test_add_zero_identity() {
    let expr = Expr::binary(BinaryOp::Add, Expr::int(5), Expr::int(0));
    assert_eq!(expr.simplify(), Expr::int(5));
}

#[test]
fn test_multiply_one_identity() {
    let expr = Expr::binary(BinaryOp::Multiply, Expr::int(7), Expr::int(1));
    assert_eq!(expr.simplify(), Expr::int(7));
}

#[test]
fn test_multiply_zero_absorbs() {
    let expr = Expr::binary(BinaryOp::Multiply, Expr::int(99), Expr::int(0));
    assert_eq!(expr.simplify(), Expr::int(0));
}

#[test]
fn test_power_zero() {
    let expr = Expr::binary(BinaryOp::Power, Expr::int(5), Expr::int(0));
    assert_eq!(expr.simplify(), Expr::int(1));
}

#[test]
fn test_power_one() {
    let expr = Expr::binary(BinaryOp::Power, Expr::int(5), Expr::int(1));
    assert_eq!(expr.simplify(), Expr::int(5));
}

#[test]
fn test_double_negation() {
    let expr = Expr::unary(UnaryOp::Negate, Expr::unary(UnaryOp::Negate, Expr::int(5)));
    assert_eq!(expr.simplify(), Expr::int(5));
}

#[test]
fn test_fraction_simplification_in_expr() {
    // (2/4) evaluates to 1/2
    let expr = Expr::binary(BinaryOp::Divide, Expr::int(2), Expr::int(4));
    let result = expr.eval(AngleMode::Radians).unwrap();
    assert_eq!(result, Value::Rational(1, 2));
}

// =============================================================================
// Calculator state tests
// =============================================================================

#[test]
fn test_calculator_starts_at_zero() {
    let calc = Calculator::new();
    assert!(calc.display.is_zero());
}

#[test]
fn test_memory_operations() {
    let mut calc = Calculator::new();
    calc.enter(Value::int(42));
    calc.memory_op(MemoryOp::Store);
    calc.clear();
    assert!(calc.display.is_zero());
    calc.memory_op(MemoryOp::Recall);
    assert_eq!(calc.display, Value::int(42));
}

#[test]
fn test_memory_add() {
    let mut calc = Calculator::new();
    calc.enter(Value::int(10));
    calc.memory_op(MemoryOp::Store);
    calc.enter(Value::int(5));
    calc.memory_op(MemoryOp::Add);
    calc.memory_op(MemoryOp::Recall);
    assert_eq!(calc.display, Value::int(15));
}

#[test]
fn test_history_tracked() {
    let mut calc = Calculator::new();
    calc.enter(Value::int(5));
    calc.binary(BinaryOp::Add, Value::int(3)).unwrap();
    assert_eq!(calc.history.len(), 1);
    assert_eq!(calc.display, Value::int(8));
}

// =============================================================================
// Property-based tests
// =============================================================================

proptest! {
    /// Rational values are always simplified (gcd of num/den = 1)
    #[test]
    fn prop_rational_simplified(n in arb_int(), d in arb_nonzero_int()) {
        let v = Value::rational(n, d).unwrap();
        if let Value::Rational(rn, rd) = v {
            let g = crate::value::gcd(rn.unsigned_abs(), rd.unsigned_abs());
            prop_assert_eq!(g, 1, "{}/{} not fully simplified (gcd={})", rn, rd, g);
        }
    }

    /// Denominator is always positive after normalization
    #[test]
    fn prop_positive_denominator(n in arb_int(), d in arb_nonzero_int()) {
        let v = Value::rational(n, d).unwrap();
        if let Value::Rational(_, rd) = v {
            prop_assert!(rd > 0, "denominator {} should be positive", rd);
        }
    }

    /// Division by zero always fails
    #[test]
    fn prop_div_zero_fails(n in arb_int()) {
        let result = BinaryOp::Divide.apply(&Value::int(n), &Value::int(0));
        prop_assert_eq!(result, Err(CalcError::DivisionByZero));
    }

    /// a + 0 = a
    #[test]
    fn prop_add_zero_identity(v in arb_value()) {
        let result = BinaryOp::Add.apply(&v, &Value::int(0)).unwrap();
        prop_assert_eq!(result.to_f64(), v.to_f64());
    }

    /// a * 1 = a
    #[test]
    fn prop_mul_one_identity(v in arb_value()) {
        let result = BinaryOp::Multiply.apply(&v, &Value::int(1)).unwrap();
        prop_assert_eq!(result.to_f64(), v.to_f64());
    }

    /// a * 0 = 0
    #[test]
    fn prop_mul_zero_absorbs(v in arb_value()) {
        let result = BinaryOp::Multiply.apply(&v, &Value::int(0)).unwrap();
        prop_assert!(result.is_zero());
    }

    /// Addition is commutative: a + b = b + a
    #[test]
    fn prop_add_commutative(a in arb_value(), b in arb_value()) {
        let ab = BinaryOp::Add.apply(&a, &b).unwrap();
        let ba = BinaryOp::Add.apply(&b, &a).unwrap();
        prop_assert!((ab.to_f64() - ba.to_f64()).abs() < 1e-10);
    }

    /// Multiplication is commutative: a * b = b * a
    #[test]
    fn prop_mul_commutative(a in arb_value(), b in arb_value()) {
        let ab = BinaryOp::Multiply.apply(&a, &b).unwrap();
        let ba = BinaryOp::Multiply.apply(&b, &a).unwrap();
        prop_assert!((ab.to_f64() - ba.to_f64()).abs() < 1e-10);
    }

    /// Double negation: --a = a
    #[test]
    fn prop_double_negate(v in arb_value()) {
        prop_assert_eq!(v.negate().negate(), v);
    }

    /// Reciprocal of reciprocal = original (for non-zero)
    #[test]
    fn prop_double_reciprocal(n in arb_nonzero_int(), d in arb_nonzero_int()) {
        let v = Value::rational(n, d).unwrap();
        let rr = v.reciprocal().unwrap().reciprocal().unwrap();
        prop_assert!((rr.to_f64() - v.to_f64()).abs() < 1e-10);
    }

    /// sqrt(x)^2 = x for non-negative integers
    #[test]
    fn prop_sqrt_square_identity(n in 0..1000i64) {
        let v = Value::int(n);
        if let Ok(root) = UnaryOp::Sqrt.apply(&v, AngleMode::Radians) {
            let squared = UnaryOp::Square.apply(&root, AngleMode::Radians).unwrap();
            prop_assert!((squared.to_f64() - n as f64).abs() < 1e-6);
        }
    }

    /// sqrt of negative always fails
    #[test]
    fn prop_sqrt_negative_fails(n in 1..1000i64) {
        let result = UnaryOp::Sqrt.apply(&Value::int(-n), AngleMode::Radians);
        prop_assert_eq!(result, Err(CalcError::NegativeSquareRoot));
    }

    /// ln(e^x) ≈ x
    #[test]
    fn prop_ln_exp_inverse(x in -10.0..10.0f64) {
        let exp_x = UnaryOp::Exp.apply(&Value::Float(x), AngleMode::Radians).unwrap();
        let ln_exp_x = UnaryOp::Ln.apply(&exp_x, AngleMode::Radians).unwrap();
        prop_assert!((ln_exp_x.to_f64() - x).abs() < 1e-10);
    }

    /// sin^2 + cos^2 = 1
    #[test]
    fn prop_pythagorean_identity(angle in -360.0..360.0f64) {
        let v = Value::Float(angle);
        let sin = UnaryOp::Sin.apply(&v, AngleMode::Degrees).unwrap().to_f64();
        let cos = UnaryOp::Cos.apply(&v, AngleMode::Degrees).unwrap().to_f64();
        prop_assert!((sin * sin + cos * cos - 1.0).abs() < 1e-10,
            "sin²({}) + cos²({}) = {} ≠ 1", angle, angle, sin * sin + cos * cos);
    }

    /// sin(0) = 0, cos(0) = 1
    #[test]
    fn prop_trig_at_zero(_x in 0..1u8) {
        let zero = Value::int(0);
        let sin0 = UnaryOp::Sin.apply(&zero, AngleMode::Radians).unwrap().to_f64();
        let cos0 = UnaryOp::Cos.apply(&zero, AngleMode::Radians).unwrap().to_f64();
        prop_assert!(sin0.abs() < 1e-10);
        prop_assert!((cos0 - 1.0).abs() < 1e-10);
    }

    /// Expression simplification preserves value
    #[test]
    fn prop_simplify_preserves_value(a in arb_int(), b in arb_int()) {
        let expr = Expr::binary(BinaryOp::Add, Expr::int(a), Expr::int(b));
        let original = expr.eval(AngleMode::Radians).unwrap();
        let simplified = expr.simplify().eval(AngleMode::Radians).unwrap();
        prop_assert!((original.to_f64() - simplified.to_f64()).abs() < 1e-10);
    }

    /// Simplify is idempotent: simplify(simplify(x)) = simplify(x)
    #[test]
    fn prop_simplify_idempotent(a in arb_int(), b in arb_int()) {
        let expr = Expr::binary(BinaryOp::Multiply, Expr::int(a), Expr::int(b));
        let s1 = expr.simplify();
        let s2 = s1.simplify();
        prop_assert_eq!(s1, s2);
    }

    /// Calculator clear resets to zero
    #[test]
    fn prop_clear_is_zero(n in arb_int()) {
        let mut calc = Calculator::new();
        calc.enter(Value::int(n));
        calc.clear();
        prop_assert!(calc.display.is_zero());
    }

    /// All clear resets everything
    #[test]
    fn prop_all_clear(n in arb_int()) {
        let mut calc = Calculator::new();
        calc.enter(Value::int(n));
        calc.memory_op(MemoryOp::Store);
        calc.all_clear();
        prop_assert!(calc.display.is_zero());
        prop_assert!(calc.memory.is_zero());
        prop_assert!(calc.history.is_empty());
    }

    /// Memory store then recall gives same value
    #[test]
    fn prop_memory_store_recall(n in arb_int()) {
        let mut calc = Calculator::new();
        calc.enter(Value::int(n));
        calc.memory_op(MemoryOp::Store);
        calc.clear();
        calc.memory_op(MemoryOp::Recall);
        prop_assert_eq!(calc.display, Value::int(n));
    }

    /// Precedence: multiply > add
    #[test]
    fn prop_precedence(_x in 0..1u8) {
        prop_assert!(BinaryOp::Multiply.precedence() > BinaryOp::Add.precedence());
        prop_assert!(BinaryOp::Power.precedence() > BinaryOp::Multiply.precedence());
    }

    // === Hyperbolic functions ===

    /// cosh^2 - sinh^2 = 1 (small values to avoid precision loss)
    #[test]
    fn prop_hyperbolic_identity(x in -5.0..5.0f64) {
        let v = Value::Float(x);
        let sinh = UnaryOp::Sinh.apply(&v, AngleMode::Radians).unwrap().to_f64();
        let cosh = UnaryOp::Cosh.apply(&v, AngleMode::Radians).unwrap().to_f64();
        prop_assert!((cosh * cosh - sinh * sinh - 1.0).abs() < 1e-6);
    }

    /// tanh is bounded [-1, 1]
    #[test]
    fn prop_tanh_bounded(x in -100.0..100.0f64) {
        let result = UnaryOp::Tanh.apply(&Value::Float(x), AngleMode::Radians).unwrap().to_f64();
        prop_assert!(result >= -1.0 && result <= 1.0);
    }

    /// asinh(sinh(x)) = x
    #[test]
    fn prop_asinh_inverse(x in -10.0..10.0f64) {
        let sinh_x = UnaryOp::Sinh.apply(&Value::Float(x), AngleMode::Radians).unwrap();
        let result = UnaryOp::Asinh.apply(&sinh_x, AngleMode::Radians).unwrap().to_f64();
        prop_assert!((result - x).abs() < 1e-8);
    }

    // === Complex numbers ===

    /// |z * conj(z)| = |z|^2
    #[test]
    fn prop_complex_magnitude_squared(re in -100.0..100.0f64, im in -100.0..100.0f64) {
        let z = Complex::new(re, im);
        let product = z.mul(&z.conjugate());
        let mag_sq = z.magnitude() * z.magnitude();
        prop_assert!((product.re - mag_sq).abs() < 1e-6);
        prop_assert!(product.im.abs() < 1e-6);
    }

    /// z + conj(z) = 2 * re(z)
    #[test]
    fn prop_complex_conj_add(re in -100.0..100.0f64, im in -100.0..100.0f64) {
        let z = Complex::new(re, im);
        let sum = z.add(&z.conjugate());
        prop_assert!((sum.re - 2.0 * re).abs() < 1e-10);
        prop_assert!(sum.im.abs() < 1e-10);
    }

    /// z - conj(z) = 2i * im(z)
    #[test]
    fn prop_complex_conj_sub(re in -100.0..100.0f64, im in -100.0..100.0f64) {
        let z = Complex::new(re, im);
        let diff = z.sub(&z.conjugate());
        prop_assert!(diff.re.abs() < 1e-10);
        prop_assert!((diff.im - 2.0 * im).abs() < 1e-10);
    }

    /// sqrt(-1) = i
    #[test]
    fn prop_sqrt_neg_one(_x in 0..1u8) {
        let z = Complex::real(-1.0);
        let result = z.sqrt();
        prop_assert!(result.re.abs() < 1e-10);
        prop_assert!((result.im - 1.0).abs() < 1e-10);
    }

    /// Complex division: z / z = 1 (for non-zero z)
    #[test]
    fn prop_complex_div_self(re in 1.0..100.0f64, im in 1.0..100.0f64) {
        let z = Complex::new(re, im);
        let result = z.div(&z).unwrap();
        prop_assert!((result.re - 1.0).abs() < 1e-8);
        prop_assert!(result.im.abs() < 1e-8);
    }

    /// e^(iπ) + 1 = 0 (Euler's identity)
    #[test]
    fn prop_euler_identity(_x in 0..1u8) {
        let i_pi = Complex::new(0.0, std::f64::consts::PI);
        let result = i_pi.exp().add(&Complex::ONE);
        prop_assert!(result.re.abs() < 1e-10);
        prop_assert!(result.im.abs() < 1e-10);
    }

    // === Bitwise operations ===

    /// AND is commutative
    #[test]
    fn prop_and_commutative(a in -1000..1000i64, b in -1000..1000i64) {
        prop_assert_eq!(
            BitwiseOp::And.apply(a, b).unwrap(),
            BitwiseOp::And.apply(b, a).unwrap()
        );
    }

    /// OR is commutative
    #[test]
    fn prop_or_commutative(a in -1000..1000i64, b in -1000..1000i64) {
        prop_assert_eq!(
            BitwiseOp::Or.apply(a, b).unwrap(),
            BitwiseOp::Or.apply(b, a).unwrap()
        );
    }

    /// XOR is commutative
    #[test]
    fn prop_xor_commutative(a in -1000..1000i64, b in -1000..1000i64) {
        prop_assert_eq!(
            BitwiseOp::Xor.apply(a, b).unwrap(),
            BitwiseOp::Xor.apply(b, a).unwrap()
        );
    }

    /// a XOR a = 0
    #[test]
    fn prop_xor_self_zero(a in -1000..1000i64) {
        prop_assert_eq!(BitwiseOp::Xor.apply(a, a).unwrap(), 0);
    }

    /// NOT NOT a = a
    #[test]
    fn prop_double_not(a in -1000..1000i64) {
        let not_a = BitwiseOp::Not.apply(a, 0).unwrap();
        let not_not_a = BitwiseOp::Not.apply(not_a, 0).unwrap();
        prop_assert_eq!(not_not_a, a);
    }

    /// a AND 0 = 0
    #[test]
    fn prop_and_zero(a in -1000..1000i64) {
        prop_assert_eq!(BitwiseOp::And.apply(a, 0).unwrap(), 0);
    }

    /// a OR 0 = a
    #[test]
    fn prop_or_zero(a in -1000..1000i64) {
        prop_assert_eq!(BitwiseOp::Or.apply(a, 0).unwrap(), a);
    }

    // === Base conversion ===

    /// Roundtrip: format then parse = original
    #[test]
    fn prop_base_roundtrip(n in 0..10000i64) {
        for base in [Base::Binary, Base::Octal, Base::Decimal, Base::Hexadecimal] {
            let formatted = base.format(n);
            let parsed = base.parse(&formatted).unwrap();
            prop_assert_eq!(parsed, n, "roundtrip failed for {} in {:?}", n, base);
        }
    }

    // === Unit conversion ===

    /// Roundtrip conversion: convert A→B→A = original
    #[test]
    fn prop_unit_roundtrip(meters in 0.1..10000.0f64) {
        let miles = convert(meters, Unit::Meter, Unit::Mile).unwrap();
        let back = convert(miles, Unit::Mile, Unit::Meter).unwrap();
        prop_assert!((back - meters).abs() < 0.001);
    }

    /// Temperature roundtrip: C→F→C
    #[test]
    fn prop_temp_roundtrip(celsius in -273.15..1000.0f64) {
        let f = convert(celsius, Unit::Celsius, Unit::Fahrenheit).unwrap();
        let back = convert(f, Unit::Fahrenheit, Unit::Celsius).unwrap();
        prop_assert!((back - celsius).abs() < 0.001);
    }

    /// Below absolute zero is rejected
    #[test]
    fn prop_absolute_zero_enforced(kelvin in -1000.0..-0.01f64) {
        let result = convert(kelvin, Unit::Kelvin, Unit::Celsius);
        prop_assert!(result.is_err());
    }

    /// Cross-category conversion rejected
    #[test]
    fn prop_incompatible_units_rejected(meters in 0.1..1000.0f64) {
        let result = convert(meters, Unit::Meter, Unit::Kilogram);
        prop_assert!(result.is_err());
    }

    /// 0°C = 32°F
    #[test]
    fn prop_freezing_point(_x in 0..1u8) {
        let f = convert(0.0, Unit::Celsius, Unit::Fahrenheit).unwrap();
        prop_assert!((f - 32.0).abs() < 0.001);
    }

    /// 100°C = 212°F
    #[test]
    fn prop_boiling_point(_x in 0..1u8) {
        let f = convert(100.0, Unit::Celsius, Unit::Fahrenheit).unwrap();
        prop_assert!((f - 212.0).abs() < 0.001);
    }

    // === Combinatorics ===

    /// nC0 = 1
    #[test]
    fn prop_ncr_zero(n in 0..20u64) {
        prop_assert_eq!(combinations(n, 0).unwrap(), 1);
    }

    /// nCn = 1
    #[test]
    fn prop_ncr_n(n in 0..20u64) {
        prop_assert_eq!(combinations(n, n).unwrap(), 1);
    }

    /// nC1 = n
    #[test]
    fn prop_ncr_one(n in 1..20u64) {
        prop_assert_eq!(combinations(n, 1).unwrap(), n);
    }

    /// nCr = nC(n-r) (symmetry)
    #[test]
    fn prop_ncr_symmetric(n in 2..15u64, r in 0..15u64) {
        prop_assume!(r <= n);
        prop_assert_eq!(combinations(n, r).unwrap(), combinations(n, n - r).unwrap());
    }

    /// nPr >= nCr (permutations >= combinations)
    #[test]
    fn prop_npr_gte_ncr(n in 2..15u64, r in 1..15u64) {
        prop_assume!(r <= n);
        prop_assert!(permutations(n, r).unwrap() >= combinations(n, r).unwrap());
    }

    /// nPn = n!
    #[test]
    fn prop_npn_is_factorial(n in 0..15u64) {
        let npr = permutations(n, n).unwrap();
        let fact: u64 = (1..=n.max(1)).product();
        prop_assert_eq!(npr, fact);
    }

    /// r > n is rejected for both nCr and nPr
    #[test]
    fn prop_r_gt_n_rejected(n in 0..10u64) {
        prop_assert!(combinations(n, n + 1).is_err());
        prop_assert!(permutations(n, n + 1).is_err());
    }

    // === Constants ===

    /// All constants have positive values
    #[test]
    fn prop_constants_positive(_x in 0..1u8) {
        for c in Constant::all() {
            prop_assert!(c.value().to_f64() > 0.0, "{} should be positive", c.name());
        }
    }

    /// pi is approximately 3.14159
    #[test]
    fn prop_pi_value(_x in 0..1u8) {
        let pi = Constant::Pi.value().to_f64();
        prop_assert!((pi - 3.14159265358979).abs() < 1e-10);
    }

    /// e is approximately 2.71828
    #[test]
    fn prop_e_value(_x in 0..1u8) {
        let e = Constant::E.value().to_f64();
        prop_assert!((e - 2.71828182845904).abs() < 1e-10);
    }
}
