use super::*;
use praxis::engine::Action;
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
            let g = super::value::gcd(rn.unsigned_abs(), rd.unsigned_abs());
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

// =============================================================================
// Expr construction and evaluation tests
// =============================================================================

#[test]
fn test_expr_lit() {
    let v = Value::rational(3, 4).unwrap();
    let expr = Expr::lit(v.clone());
    assert_eq!(expr, Expr::Lit(v));
}

#[test]
fn test_expr_int() {
    let expr = Expr::int(42);
    assert_eq!(expr, Expr::Lit(Value::int(42)));
}

#[test]
fn test_expr_unary_construction() {
    let expr = Expr::unary(UnaryOp::Negate, Expr::int(5));
    match &expr {
        Expr::Unary(op, inner) => {
            assert_eq!(*op, UnaryOp::Negate);
            assert_eq!(**inner, Expr::int(5));
        }
        _ => panic!("expected Unary"),
    }
}

#[test]
fn test_expr_binary_construction() {
    let expr = Expr::binary(BinaryOp::Subtract, Expr::int(10), Expr::int(3));
    match &expr {
        Expr::Binary(op, lhs, rhs) => {
            assert_eq!(*op, BinaryOp::Subtract);
            assert_eq!(**lhs, Expr::int(10));
            assert_eq!(**rhs, Expr::int(3));
        }
        _ => panic!("expected Binary"),
    }
}

#[test]
fn test_expr_eval_lit() {
    let expr = Expr::lit(Value::Float(2.5));
    let result = expr.eval(AngleMode::Radians).unwrap();
    assert_eq!(result, Value::Float(2.5));
}

#[test]
fn test_expr_eval_unary() {
    let expr = Expr::unary(UnaryOp::Negate, Expr::int(7));
    let result = expr.eval(AngleMode::Radians).unwrap();
    assert_eq!(result, Value::int(-7));
}

#[test]
fn test_expr_eval_binary_subtract() {
    let expr = Expr::binary(BinaryOp::Subtract, Expr::int(10), Expr::int(3));
    let result = expr.eval(AngleMode::Radians).unwrap();
    assert_eq!(result, Value::int(7));
}

#[test]
fn test_expr_eval_nested() {
    // (2 + 3) * 4 = 20
    let inner = Expr::binary(BinaryOp::Add, Expr::int(2), Expr::int(3));
    let expr = Expr::binary(BinaryOp::Multiply, inner, Expr::int(4));
    let result = expr.eval(AngleMode::Radians).unwrap();
    assert_eq!(result, Value::int(20));
}

#[test]
fn test_expr_eval_error_propagates() {
    // sqrt(-1) should fail
    let expr = Expr::unary(UnaryOp::Sqrt, Expr::int(-1));
    assert_eq!(
        expr.eval(AngleMode::Radians),
        Err(CalcError::NegativeSquareRoot)
    );
}

#[test]
fn test_expr_eval_binary_error_propagates() {
    // 5 / 0 should fail
    let expr = Expr::binary(BinaryOp::Divide, Expr::int(5), Expr::int(0));
    assert_eq!(
        expr.eval(AngleMode::Radians),
        Err(CalcError::DivisionByZero)
    );
}

// Additional simplification tests

#[test]
fn test_simplify_subtract_zero() {
    let expr = Expr::binary(BinaryOp::Subtract, Expr::int(5), Expr::int(0));
    assert_eq!(expr.simplify(), Expr::int(5));
}

#[test]
fn test_simplify_divide_by_one() {
    let expr = Expr::binary(BinaryOp::Divide, Expr::int(9), Expr::int(1));
    assert_eq!(expr.simplify(), Expr::int(9));
}

#[test]
fn test_simplify_zero_times_left() {
    let expr = Expr::binary(BinaryOp::Multiply, Expr::int(0), Expr::int(42));
    assert_eq!(expr.simplify(), Expr::int(0));
}

#[test]
fn test_simplify_one_times_left() {
    let expr = Expr::binary(BinaryOp::Multiply, Expr::int(1), Expr::int(42));
    assert_eq!(expr.simplify(), Expr::int(42));
}

#[test]
fn test_simplify_zero_plus_left() {
    let expr = Expr::binary(BinaryOp::Add, Expr::int(0), Expr::int(7));
    assert_eq!(expr.simplify(), Expr::int(7));
}

#[test]
fn test_simplify_negate_constant_folds() {
    let expr = Expr::unary(UnaryOp::Negate, Expr::int(3));
    assert_eq!(expr.simplify(), Expr::Lit(Value::int(-3)));
}

#[test]
fn test_simplify_unary_constant_folds() {
    // sqrt(4) should constant fold to 2
    let expr = Expr::unary(UnaryOp::Sqrt, Expr::int(4));
    let simplified = expr.simplify();
    let val = simplified.eval(AngleMode::Radians).unwrap();
    assert!((val.to_f64() - 2.0).abs() < 1e-10);
}

#[test]
fn test_simplify_modulo_not_identity() {
    // Modulo doesn't have identity rules, so it should constant fold
    let expr = Expr::binary(BinaryOp::Modulo, Expr::int(10), Expr::int(3));
    let simplified = expr.simplify();
    let val = simplified.eval(AngleMode::Radians).unwrap();
    assert!((val.to_f64() - 1.0).abs() < 1e-10);
}

// =============================================================================
// Complex number tests
// =============================================================================

#[test]
fn test_complex_new() {
    let z = Complex::new(3.0, 4.0);
    assert_eq!(z.re, 3.0);
    assert_eq!(z.im, 4.0);
}

#[test]
fn test_complex_real() {
    let z = Complex::real(5.0);
    assert_eq!(z.re, 5.0);
    assert_eq!(z.im, 0.0);
}

#[test]
fn test_complex_imaginary() {
    let z = Complex::imaginary(3.0);
    assert_eq!(z.re, 0.0);
    assert_eq!(z.im, 3.0);
}

#[test]
fn test_complex_constants() {
    assert_eq!(Complex::I.re, 0.0);
    assert_eq!(Complex::I.im, 1.0);
    assert_eq!(Complex::ZERO.re, 0.0);
    assert_eq!(Complex::ZERO.im, 0.0);
    assert_eq!(Complex::ONE.re, 1.0);
    assert_eq!(Complex::ONE.im, 0.0);
}

#[test]
fn test_complex_is_real() {
    assert!(Complex::real(5.0).is_real());
    assert!(!Complex::new(1.0, 2.0).is_real());
    assert!(Complex::ZERO.is_real());
}

#[test]
fn test_complex_is_imaginary() {
    assert!(Complex::imaginary(3.0).is_imaginary());
    assert!(!Complex::new(1.0, 2.0).is_imaginary());
    assert!(!Complex::ZERO.is_imaginary()); // both parts zero -> not imaginary
    assert!(!Complex::real(5.0).is_imaginary());
}

#[test]
fn test_complex_magnitude() {
    let z = Complex::new(3.0, 4.0);
    assert!((z.magnitude() - 5.0).abs() < 1e-10);
}

#[test]
fn test_complex_phase() {
    let z = Complex::new(1.0, 1.0);
    assert!((z.phase() - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
}

#[test]
fn test_complex_conjugate() {
    let z = Complex::new(3.0, 4.0);
    let conj = z.conjugate();
    assert_eq!(conj.re, 3.0);
    assert_eq!(conj.im, -4.0);
}

#[test]
fn test_complex_add() {
    let a = Complex::new(1.0, 2.0);
    let b = Complex::new(3.0, 4.0);
    let result = a.add(&b);
    assert!((result.re - 4.0).abs() < 1e-10);
    assert!((result.im - 6.0).abs() < 1e-10);
}

#[test]
fn test_complex_sub() {
    let a = Complex::new(5.0, 7.0);
    let b = Complex::new(2.0, 3.0);
    let result = a.sub(&b);
    assert!((result.re - 3.0).abs() < 1e-10);
    assert!((result.im - 4.0).abs() < 1e-10);
}

#[test]
fn test_complex_mul() {
    // (1+2i)(3+4i) = 3+4i+6i+8i² = 3+10i-8 = -5+10i
    let a = Complex::new(1.0, 2.0);
    let b = Complex::new(3.0, 4.0);
    let result = a.mul(&b);
    assert!((result.re - (-5.0)).abs() < 1e-10);
    assert!((result.im - 10.0).abs() < 1e-10);
}

#[test]
fn test_complex_div() {
    // (1+2i)/(1+0i) = 1+2i
    let a = Complex::new(1.0, 2.0);
    let b = Complex::real(1.0);
    let result = a.div(&b).unwrap();
    assert!((result.re - 1.0).abs() < 1e-10);
    assert!((result.im - 2.0).abs() < 1e-10);
}

#[test]
fn test_complex_div_by_zero() {
    let a = Complex::new(1.0, 2.0);
    let b = Complex::ZERO;
    assert_eq!(a.div(&b), Err(CalcError::DivisionByZero));
}

#[test]
fn test_complex_sqrt() {
    // sqrt(4) = 2
    let z = Complex::real(4.0);
    let result = z.sqrt();
    assert!((result.re - 2.0).abs() < 1e-10);
    assert!(result.im.abs() < 1e-10);
}

#[test]
fn test_complex_sqrt_negative() {
    // sqrt(-4) = 2i
    let z = Complex::real(-4.0);
    let result = z.sqrt();
    assert!(result.re.abs() < 1e-10);
    assert!((result.im - 2.0).abs() < 1e-10);
}

#[test]
fn test_complex_sqrt_imaginary() {
    // sqrt(i) = (1+i)/sqrt(2)
    let z = Complex::I;
    let result = z.sqrt();
    let expected_re = (0.5_f64).sqrt();
    let expected_im = (0.5_f64).sqrt();
    assert!((result.re - expected_re).abs() < 1e-10);
    assert!((result.im - expected_im).abs() < 1e-10);
}

#[test]
fn test_complex_exp() {
    // e^0 = 1
    let z = Complex::ZERO;
    let result = z.exp();
    assert!((result.re - 1.0).abs() < 1e-10);
    assert!(result.im.abs() < 1e-10);
}

#[test]
fn test_complex_exp_real() {
    // e^1 = e
    let z = Complex::real(1.0);
    let result = z.exp();
    assert!((result.re - std::f64::consts::E).abs() < 1e-10);
    assert!(result.im.abs() < 1e-10);
}

#[test]
fn test_complex_ln() {
    // ln(1) = 0
    let z = Complex::ONE;
    let result = z.ln().unwrap();
    assert!(result.re.abs() < 1e-10);
    assert!(result.im.abs() < 1e-10);
}

#[test]
fn test_complex_ln_e() {
    // ln(e) = 1
    let z = Complex::real(std::f64::consts::E);
    let result = z.ln().unwrap();
    assert!((result.re - 1.0).abs() < 1e-10);
    assert!(result.im.abs() < 1e-10);
}

#[test]
fn test_complex_ln_zero() {
    let z = Complex::ZERO;
    assert_eq!(z.ln(), Err(CalcError::LogOfNonPositive));
}

#[test]
fn test_complex_ln_negative() {
    // ln(-1) = iπ
    let z = Complex::real(-1.0);
    let result = z.ln().unwrap();
    assert!(result.re.abs() < 1e-10);
    assert!((result.im - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn test_complex_negate() {
    let z = Complex::new(3.0, -4.0);
    let result = z.negate();
    assert_eq!(result.re, -3.0);
    assert_eq!(result.im, 4.0);
}

#[test]
fn test_complex_pow_integer() {
    // (1+i)^2 = 2i
    let z = Complex::new(1.0, 1.0);
    let w = Complex::real(2.0);
    let result = z.pow(&w).unwrap();
    assert!(result.re.abs() < 1e-8);
    assert!((result.im - 2.0).abs() < 1e-8);
}

#[test]
fn test_complex_pow_zero_base_positive_exp() {
    let z = Complex::ZERO;
    let w = Complex::real(5.0);
    let result = z.pow(&w).unwrap();
    assert_eq!(result.re, 0.0);
    assert_eq!(result.im, 0.0);
}

#[test]
fn test_complex_pow_zero_base_non_positive_exp() {
    let z = Complex::ZERO;
    let w = Complex::real(-1.0);
    assert_eq!(z.pow(&w), Err(CalcError::DivisionByZero));
}

#[test]
fn test_complex_display_real() {
    let z = Complex::real(3.0);
    assert_eq!(format!("{}", z), "3");
}

#[test]
fn test_complex_display_imaginary() {
    let z = Complex::imaginary(4.0);
    assert_eq!(format!("{}", z), "4i");
}

#[test]
fn test_complex_display_positive_imaginary() {
    let z = Complex::new(3.0, 4.0);
    assert_eq!(format!("{}", z), "3 + 4i");
}

#[test]
fn test_complex_display_negative_imaginary() {
    let z = Complex::new(3.0, -4.0);
    assert_eq!(format!("{}", z), "3 - 4i");
}

// =============================================================================
// Bitwise operations tests
// =============================================================================

#[test]
fn test_bitwise_and() {
    assert_eq!(BitwiseOp::And.apply(0b1100, 0b1010).unwrap(), 0b1000);
}

#[test]
fn test_bitwise_or() {
    assert_eq!(BitwiseOp::Or.apply(0b1100, 0b1010).unwrap(), 0b1110);
}

#[test]
fn test_bitwise_xor() {
    assert_eq!(BitwiseOp::Xor.apply(0b1100, 0b1010).unwrap(), 0b0110);
}

#[test]
fn test_bitwise_not() {
    assert_eq!(BitwiseOp::Not.apply(0, 0).unwrap(), !0i64);
    assert_eq!(BitwiseOp::Not.apply(1, 999).unwrap(), !1i64); // rhs is ignored
}

#[test]
fn test_bitwise_shift_left() {
    assert_eq!(BitwiseOp::ShiftLeft.apply(1, 4).unwrap(), 16);
    assert_eq!(BitwiseOp::ShiftLeft.apply(3, 2).unwrap(), 12);
}

#[test]
fn test_bitwise_shift_right() {
    assert_eq!(BitwiseOp::ShiftRight.apply(16, 4).unwrap(), 1);
    assert_eq!(BitwiseOp::ShiftRight.apply(12, 2).unwrap(), 3);
}

#[test]
fn test_bitwise_shift_left_zero() {
    assert_eq!(BitwiseOp::ShiftLeft.apply(5, 0).unwrap(), 5);
}

#[test]
fn test_bitwise_shift_right_zero() {
    assert_eq!(BitwiseOp::ShiftRight.apply(5, 0).unwrap(), 5);
}

#[test]
fn test_bitwise_shift_left_invalid_amount() {
    assert!(BitwiseOp::ShiftLeft.apply(1, 64).is_err());
    assert!(BitwiseOp::ShiftLeft.apply(1, -1).is_err());
}

#[test]
fn test_bitwise_shift_right_invalid_amount() {
    assert!(BitwiseOp::ShiftRight.apply(1, 64).is_err());
    assert!(BitwiseOp::ShiftRight.apply(1, -1).is_err());
}

#[test]
fn test_bitwise_shift_max_valid() {
    // 63 is the max valid shift amount
    assert_eq!(BitwiseOp::ShiftLeft.apply(1, 63).unwrap(), 1i64 << 63);
    assert_eq!(
        BitwiseOp::ShiftRight.apply(1i64 << 63, 63).unwrap(),
        // arithmetic shift right for signed
        -1 // 1<<63 is i64::MIN, shifted right 63 gives -1
    );
}

// =============================================================================
// Base conversion tests
// =============================================================================

#[test]
fn test_base_radix() {
    assert_eq!(Base::Binary.radix(), 2);
    assert_eq!(Base::Octal.radix(), 8);
    assert_eq!(Base::Decimal.radix(), 10);
    assert_eq!(Base::Hexadecimal.radix(), 16);
}

#[test]
fn test_base_format_binary() {
    assert_eq!(Base::Binary.format(10), "0b1010");
    assert_eq!(Base::Binary.format(0), "0b0");
}

#[test]
fn test_base_format_octal() {
    assert_eq!(Base::Octal.format(8), "0o10");
    assert_eq!(Base::Octal.format(255), "0o377");
}

#[test]
fn test_base_format_decimal() {
    assert_eq!(Base::Decimal.format(42), "42");
    assert_eq!(Base::Decimal.format(-5), "-5");
}

#[test]
fn test_base_format_hex() {
    assert_eq!(Base::Hexadecimal.format(255), "0xFF");
    assert_eq!(Base::Hexadecimal.format(16), "0x10");
}

#[test]
fn test_base_parse_binary() {
    assert_eq!(Base::Binary.parse("0b1010").unwrap(), 10);
    assert_eq!(Base::Binary.parse("1010").unwrap(), 10);
}

#[test]
fn test_base_parse_octal() {
    assert_eq!(Base::Octal.parse("0o377").unwrap(), 255);
    assert_eq!(Base::Octal.parse("10").unwrap(), 8);
}

#[test]
fn test_base_parse_decimal() {
    assert_eq!(Base::Decimal.parse("42").unwrap(), 42);
}

#[test]
fn test_base_parse_hex() {
    assert_eq!(Base::Hexadecimal.parse("0xFF").unwrap(), 255);
    assert_eq!(Base::Hexadecimal.parse("FF").unwrap(), 255);
}

#[test]
fn test_base_parse_invalid() {
    assert!(Base::Binary.parse("xyz").is_err());
    assert!(Base::Octal.parse("89").is_err());
    assert!(Base::Hexadecimal.parse("ZZZ").is_err());
}

// =============================================================================
// UnaryOp tests (untested variants)
// =============================================================================

#[test]
fn test_unary_floor() {
    assert_eq!(
        UnaryOp::Floor
            .apply(&Value::Float(3.7), AngleMode::Radians)
            .unwrap(),
        Value::int(3)
    );
    assert_eq!(
        UnaryOp::Floor
            .apply(&Value::Float(-3.2), AngleMode::Radians)
            .unwrap(),
        Value::int(-4)
    );
}

#[test]
fn test_unary_ceil() {
    assert_eq!(
        UnaryOp::Ceil
            .apply(&Value::Float(3.2), AngleMode::Radians)
            .unwrap(),
        Value::int(4)
    );
    assert_eq!(
        UnaryOp::Ceil
            .apply(&Value::Float(-3.7), AngleMode::Radians)
            .unwrap(),
        Value::int(-3)
    );
}

#[test]
fn test_unary_round() {
    assert_eq!(
        UnaryOp::Round
            .apply(&Value::Float(3.5), AngleMode::Radians)
            .unwrap(),
        Value::int(4)
    );
    assert_eq!(
        UnaryOp::Round
            .apply(&Value::Float(3.4), AngleMode::Radians)
            .unwrap(),
        Value::int(3)
    );
    assert_eq!(
        UnaryOp::Round
            .apply(&Value::Float(-3.5), AngleMode::Radians)
            .unwrap(),
        Value::int(-4)
    );
}

#[test]
fn test_unary_to_radians() {
    let result = UnaryOp::ToRadians
        .apply(&Value::Float(180.0), AngleMode::Radians)
        .unwrap();
    assert!((result.to_f64() - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn test_unary_to_degrees() {
    let result = UnaryOp::ToDegrees
        .apply(&Value::Float(std::f64::consts::PI), AngleMode::Radians)
        .unwrap();
    assert!((result.to_f64() - 180.0).abs() < 1e-10);
}

#[test]
fn test_unary_cbrt() {
    let result = UnaryOp::Cbrt
        .apply(&Value::int(27), AngleMode::Radians)
        .unwrap();
    assert!((result.to_f64() - 3.0).abs() < 1e-10);
}

#[test]
fn test_unary_cbrt_negative() {
    let result = UnaryOp::Cbrt
        .apply(&Value::int(-8), AngleMode::Radians)
        .unwrap();
    assert!((result.to_f64() - (-2.0)).abs() < 1e-10);
}

#[test]
fn test_unary_square() {
    assert_eq!(
        UnaryOp::Square
            .apply(&Value::int(5), AngleMode::Radians)
            .unwrap(),
        Value::int(25)
    );
}

#[test]
fn test_unary_square_rational() {
    let v = Value::rational(2, 3).unwrap();
    let result = UnaryOp::Square.apply(&v, AngleMode::Radians).unwrap();
    assert_eq!(result, Value::Rational(4, 9));
}

#[test]
fn test_unary_square_float() {
    let result = UnaryOp::Square
        .apply(&Value::Float(3.0), AngleMode::Radians)
        .unwrap();
    assert!((result.to_f64() - 9.0).abs() < 1e-10);
}

#[test]
fn test_unary_reciprocal() {
    let result = UnaryOp::Reciprocal
        .apply(&Value::int(4), AngleMode::Radians)
        .unwrap();
    assert_eq!(result, Value::Rational(1, 4));
}

#[test]
fn test_unary_reciprocal_zero() {
    assert_eq!(
        UnaryOp::Reciprocal.apply(&Value::int(0), AngleMode::Radians),
        Err(CalcError::DivisionByZero)
    );
}

#[test]
fn test_unary_abs_positive() {
    assert_eq!(
        UnaryOp::Abs
            .apply(&Value::int(5), AngleMode::Radians)
            .unwrap(),
        Value::int(5)
    );
}

#[test]
fn test_unary_abs_negative() {
    assert_eq!(
        UnaryOp::Abs
            .apply(&Value::int(-5), AngleMode::Radians)
            .unwrap(),
        Value::int(5)
    );
}

#[test]
fn test_unary_abs_zero() {
    assert_eq!(
        UnaryOp::Abs
            .apply(&Value::int(0), AngleMode::Radians)
            .unwrap(),
        Value::int(0)
    );
}

#[test]
fn test_unary_abs_negative_float() {
    let result = UnaryOp::Abs
        .apply(&Value::Float(-3.5), AngleMode::Radians)
        .unwrap();
    assert!((result.to_f64() - 3.5).abs() < 1e-10);
}

#[test]
fn test_unary_log10() {
    let result = UnaryOp::Log10
        .apply(&Value::int(100), AngleMode::Radians)
        .unwrap();
    assert!((result.to_f64() - 2.0).abs() < 1e-10);
}

#[test]
fn test_unary_log10_negative_rejected() {
    assert_eq!(
        UnaryOp::Log10.apply(&Value::int(-5), AngleMode::Radians),
        Err(CalcError::LogOfNonPositive)
    );
}

#[test]
fn test_unary_log2() {
    let result = UnaryOp::Log2
        .apply(&Value::int(8), AngleMode::Radians)
        .unwrap();
    assert!((result.to_f64() - 3.0).abs() < 1e-10);
}

#[test]
fn test_unary_log2_negative_rejected() {
    assert_eq!(
        UnaryOp::Log2.apply(&Value::int(-1), AngleMode::Radians),
        Err(CalcError::LogOfNonPositive)
    );
}

#[test]
fn test_unary_exp() {
    let result = UnaryOp::Exp
        .apply(&Value::int(0), AngleMode::Radians)
        .unwrap();
    assert!((result.to_f64() - 1.0).abs() < 1e-10);
}

#[test]
fn test_unary_tan_undefined() {
    // tan(pi/2) is undefined
    let v = Value::Float(std::f64::consts::FRAC_PI_2);
    assert_eq!(
        UnaryOp::Tan.apply(&v, AngleMode::Radians),
        Err(CalcError::TanUndefined)
    );
}

#[test]
fn test_unary_tan_zero() {
    let result = UnaryOp::Tan
        .apply(&Value::int(0), AngleMode::Radians)
        .unwrap();
    assert!(result.to_f64().abs() < 1e-10);
}

#[test]
fn test_unary_asin_valid() {
    let result = UnaryOp::Asin
        .apply(&Value::Float(0.5), AngleMode::Radians)
        .unwrap();
    assert!((result.to_f64() - (0.5_f64).asin()).abs() < 1e-10);
}

#[test]
fn test_unary_acos_valid() {
    let result = UnaryOp::Acos
        .apply(&Value::Float(0.5), AngleMode::Radians)
        .unwrap();
    assert!((result.to_f64() - (0.5_f64).acos()).abs() < 1e-10);
}

#[test]
fn test_unary_acos_out_of_range() {
    assert!(
        UnaryOp::Acos
            .apply(&Value::Float(1.5), AngleMode::Radians)
            .is_err()
    );
    assert!(
        UnaryOp::Acos
            .apply(&Value::Float(-1.5), AngleMode::Radians)
            .is_err()
    );
}

#[test]
fn test_unary_atan() {
    let result = UnaryOp::Atan
        .apply(&Value::Float(1.0), AngleMode::Radians)
        .unwrap();
    assert!((result.to_f64() - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
}

#[test]
fn test_unary_acosh_valid() {
    let result = UnaryOp::Acosh
        .apply(&Value::Float(1.0), AngleMode::Radians)
        .unwrap();
    assert!(result.to_f64().abs() < 1e-10); // acosh(1) = 0
}

#[test]
fn test_unary_acosh_invalid() {
    assert!(
        UnaryOp::Acosh
            .apply(&Value::Float(0.5), AngleMode::Radians)
            .is_err()
    );
}

#[test]
fn test_unary_atanh_valid() {
    let result = UnaryOp::Atanh
        .apply(&Value::Float(0.5), AngleMode::Radians)
        .unwrap();
    assert!((result.to_f64() - (0.5_f64).atanh()).abs() < 1e-10);
}

#[test]
fn test_unary_atanh_invalid() {
    assert!(
        UnaryOp::Atanh
            .apply(&Value::Float(1.0), AngleMode::Radians)
            .is_err()
    );
    assert!(
        UnaryOp::Atanh
            .apply(&Value::Float(-1.0), AngleMode::Radians)
            .is_err()
    );
    assert!(
        UnaryOp::Atanh
            .apply(&Value::Float(2.0), AngleMode::Radians)
            .is_err()
    );
}

#[test]
fn test_trig_degrees_mode() {
    // sin(30 degrees) = 0.5
    let result = UnaryOp::Sin
        .apply(&Value::Float(30.0), AngleMode::Degrees)
        .unwrap();
    assert!((result.to_f64() - 0.5).abs() < 1e-10);

    // cos(60 degrees) = 0.5
    let result = UnaryOp::Cos
        .apply(&Value::Float(60.0), AngleMode::Degrees)
        .unwrap();
    assert!((result.to_f64() - 0.5).abs() < 1e-10);
}

#[test]
fn test_asin_degrees_mode() {
    // asin(0.5) in degrees = 30
    let result = UnaryOp::Asin
        .apply(&Value::Float(0.5), AngleMode::Degrees)
        .unwrap();
    assert!((result.to_f64() - 30.0).abs() < 1e-8);
}

#[test]
fn test_acos_degrees_mode() {
    // acos(0.5) in degrees = 60
    let result = UnaryOp::Acos
        .apply(&Value::Float(0.5), AngleMode::Degrees)
        .unwrap();
    assert!((result.to_f64() - 60.0).abs() < 1e-8);
}

#[test]
fn test_atan_degrees_mode() {
    // atan(1) in degrees = 45
    let result = UnaryOp::Atan
        .apply(&Value::Float(1.0), AngleMode::Degrees)
        .unwrap();
    assert!((result.to_f64() - 45.0).abs() < 1e-8);
}

#[test]
fn test_unary_factorial_non_integer_rejected() {
    assert!(
        UnaryOp::Factorial
            .apply(&Value::Float(3.5), AngleMode::Radians)
            .is_err()
    );
}

// =============================================================================
// BinaryOp precedence tests
// =============================================================================

#[test]
fn test_binary_op_precedence_add_subtract_equal() {
    assert_eq!(BinaryOp::Add.precedence(), BinaryOp::Subtract.precedence());
}

#[test]
fn test_binary_op_precedence_multiply_divide_modulo_equal() {
    assert_eq!(
        BinaryOp::Multiply.precedence(),
        BinaryOp::Divide.precedence()
    );
    assert_eq!(
        BinaryOp::Multiply.precedence(),
        BinaryOp::Modulo.precedence()
    );
}

#[test]
fn test_binary_op_precedence_order() {
    assert!(BinaryOp::Multiply.precedence() > BinaryOp::Add.precedence());
    assert!(BinaryOp::Power.precedence() > BinaryOp::Multiply.precedence());
}

// =============================================================================
// BinaryOp apply tests (untested variants)
// =============================================================================

#[test]
fn test_binary_subtract() {
    let result = BinaryOp::Subtract
        .apply(&Value::int(10), &Value::int(3))
        .unwrap();
    assert_eq!(result, Value::int(7));
}

#[test]
fn test_binary_subtract_fractions() {
    let a = Value::rational(5, 6).unwrap();
    let b = Value::rational(1, 3).unwrap();
    let result = BinaryOp::Subtract.apply(&a, &b).unwrap();
    assert_eq!(result, Value::Rational(1, 2));
}

#[test]
fn test_binary_power() {
    let result = BinaryOp::Power
        .apply(&Value::int(2), &Value::int(10))
        .unwrap();
    assert!((result.to_f64() - 1024.0).abs() < 1e-10);
}

#[test]
fn test_binary_modulo() {
    let result = BinaryOp::Modulo
        .apply(&Value::int(10), &Value::int(3))
        .unwrap();
    assert!((result.to_f64() - 1.0).abs() < 1e-10);
}

#[test]
fn test_binary_modulo_by_zero() {
    assert_eq!(
        BinaryOp::Modulo.apply(&Value::int(10), &Value::int(0)),
        Err(CalcError::DivisionByZero)
    );
}

#[test]
fn test_binary_add_float_and_rational() {
    let a = Value::Float(1.5);
    let b = Value::int(2);
    let result = BinaryOp::Add.apply(&a, &b).unwrap();
    assert!((result.to_f64() - 3.5).abs() < 1e-10);
}

#[test]
fn test_binary_multiply_float_and_rational() {
    let a = Value::Float(2.5);
    let b = Value::int(4);
    let result = BinaryOp::Multiply.apply(&a, &b).unwrap();
    assert!((result.to_f64() - 10.0).abs() < 1e-10);
}

// =============================================================================
// Calculator state tests (additional)
// =============================================================================

#[test]
fn test_calculator_default() {
    let calc = Calculator::default();
    assert!(calc.display.is_zero());
    assert!(calc.memory.is_zero());
    assert_eq!(calc.angle_mode, AngleMode::Radians);
    assert!(calc.history.is_empty());
}

#[test]
fn test_calculator_enter() {
    let mut calc = Calculator::new();
    calc.enter(Value::int(42));
    assert_eq!(calc.display, Value::int(42));
}

#[test]
fn test_calculator_enter_overwrites() {
    let mut calc = Calculator::new();
    calc.enter(Value::int(42));
    calc.enter(Value::int(99));
    assert_eq!(calc.display, Value::int(99));
}

#[test]
fn test_calculator_unary() {
    let mut calc = Calculator::new();
    calc.enter(Value::int(5));
    calc.unary(UnaryOp::Square).unwrap();
    assert_eq!(calc.display, Value::int(25));
    assert_eq!(calc.history.len(), 1);
}

#[test]
fn test_calculator_unary_error() {
    let mut calc = Calculator::new();
    calc.enter(Value::int(-4));
    assert!(calc.unary(UnaryOp::Sqrt).is_err());
    // display should remain unchanged after error
    assert_eq!(calc.display, Value::int(-4));
}

#[test]
fn test_calculator_binary() {
    let mut calc = Calculator::new();
    calc.enter(Value::int(10));
    calc.binary(BinaryOp::Multiply, Value::int(5)).unwrap();
    assert_eq!(calc.display, Value::int(50));
}

#[test]
fn test_calculator_binary_error() {
    let mut calc = Calculator::new();
    calc.enter(Value::int(10));
    assert!(calc.binary(BinaryOp::Divide, Value::int(0)).is_err());
    assert_eq!(calc.display, Value::int(10));
}

#[test]
fn test_calculator_set_angle_mode() {
    let mut calc = Calculator::new();
    assert_eq!(calc.angle_mode, AngleMode::Radians);
    calc.set_angle_mode(AngleMode::Degrees);
    assert_eq!(calc.angle_mode, AngleMode::Degrees);
}

#[test]
fn test_calculator_angle_mode_affects_trig() {
    let mut calc = Calculator::new();
    calc.enter(Value::Float(30.0));
    calc.set_angle_mode(AngleMode::Degrees);
    calc.unary(UnaryOp::Sin).unwrap();
    assert!((calc.display.to_f64() - 0.5).abs() < 1e-10);
}

#[test]
fn test_calculator_memory_clear() {
    let mut calc = Calculator::new();
    calc.enter(Value::int(42));
    calc.memory_op(MemoryOp::Store);
    assert_eq!(calc.memory, Value::int(42));
    calc.memory_op(MemoryOp::Clear);
    assert!(calc.memory.is_zero());
}

#[test]
fn test_calculator_history_accumulates() {
    let mut calc = Calculator::new();
    calc.enter(Value::int(2));
    calc.binary(BinaryOp::Add, Value::int(3)).unwrap();
    calc.binary(BinaryOp::Multiply, Value::int(4)).unwrap();
    assert_eq!(calc.history.len(), 2);
    assert_eq!(calc.display, Value::int(20));
}

#[test]
fn test_calculator_clear_preserves_memory_and_history() {
    let mut calc = Calculator::new();
    calc.enter(Value::int(10));
    calc.memory_op(MemoryOp::Store);
    calc.binary(BinaryOp::Add, Value::int(5)).unwrap();
    calc.clear();
    assert!(calc.display.is_zero());
    assert_eq!(calc.memory, Value::int(10));
    assert_eq!(calc.history.len(), 1);
}

// =============================================================================
// Unit conversion tests (specific cases)
// =============================================================================

#[test]
fn test_unit_category_length() {
    assert_eq!(Unit::Meter.category(), UnitCategory::Length);
    assert_eq!(Unit::Kilometer.category(), UnitCategory::Length);
    assert_eq!(Unit::Centimeter.category(), UnitCategory::Length);
    assert_eq!(Unit::Millimeter.category(), UnitCategory::Length);
    assert_eq!(Unit::Mile.category(), UnitCategory::Length);
    assert_eq!(Unit::Yard.category(), UnitCategory::Length);
    assert_eq!(Unit::Foot.category(), UnitCategory::Length);
    assert_eq!(Unit::Inch.category(), UnitCategory::Length);
}

#[test]
fn test_unit_category_mass() {
    assert_eq!(Unit::Kilogram.category(), UnitCategory::Mass);
    assert_eq!(Unit::Gram.category(), UnitCategory::Mass);
    assert_eq!(Unit::Milligram.category(), UnitCategory::Mass);
    assert_eq!(Unit::Pound.category(), UnitCategory::Mass);
    assert_eq!(Unit::Ounce.category(), UnitCategory::Mass);
}

#[test]
fn test_unit_category_temperature() {
    assert_eq!(Unit::Celsius.category(), UnitCategory::Temperature);
    assert_eq!(Unit::Fahrenheit.category(), UnitCategory::Temperature);
    assert_eq!(Unit::Kelvin.category(), UnitCategory::Temperature);
}

#[test]
fn test_unit_category_speed() {
    assert_eq!(Unit::MetersPerSecond.category(), UnitCategory::Speed);
    assert_eq!(Unit::KilometersPerHour.category(), UnitCategory::Speed);
    assert_eq!(Unit::MilesPerHour.category(), UnitCategory::Speed);
    assert_eq!(Unit::Knot.category(), UnitCategory::Speed);
}

#[test]
fn test_unit_category_area() {
    assert_eq!(Unit::SquareMeter.category(), UnitCategory::Area);
    assert_eq!(Unit::SquareFoot.category(), UnitCategory::Area);
    assert_eq!(Unit::Acre.category(), UnitCategory::Area);
    assert_eq!(Unit::Hectare.category(), UnitCategory::Area);
}

#[test]
fn test_unit_category_volume() {
    assert_eq!(Unit::Liter.category(), UnitCategory::Volume);
    assert_eq!(Unit::Milliliter.category(), UnitCategory::Volume);
    assert_eq!(Unit::Gallon.category(), UnitCategory::Volume);
    assert_eq!(Unit::FluidOunce.category(), UnitCategory::Volume);
}

#[test]
fn test_unit_category_time() {
    assert_eq!(Unit::Second.category(), UnitCategory::Time);
    assert_eq!(Unit::Minute.category(), UnitCategory::Time);
    assert_eq!(Unit::Hour.category(), UnitCategory::Time);
    assert_eq!(Unit::Day.category(), UnitCategory::Time);
}

#[test]
fn test_unit_category_angle() {
    assert_eq!(Unit::Radian.category(), UnitCategory::Angle);
    assert_eq!(Unit::Degree.category(), UnitCategory::Angle);
    assert_eq!(Unit::Gradian.category(), UnitCategory::Angle);
}

#[test]
fn test_convert_meters_to_kilometers() {
    let result = convert(1000.0, Unit::Meter, Unit::Kilometer).unwrap();
    assert!((result - 1.0).abs() < 0.001);
}

#[test]
fn test_convert_kilometers_to_meters() {
    let result = convert(1.0, Unit::Kilometer, Unit::Meter).unwrap();
    assert!((result - 1000.0).abs() < 0.001);
}

#[test]
fn test_convert_meters_to_centimeters() {
    let result = convert(1.0, Unit::Meter, Unit::Centimeter).unwrap();
    assert!((result - 100.0).abs() < 0.001);
}

#[test]
fn test_convert_inches_to_feet() {
    let result = convert(12.0, Unit::Inch, Unit::Foot).unwrap();
    assert!((result - 1.0).abs() < 0.001);
}

#[test]
fn test_convert_miles_to_kilometers() {
    let result = convert(1.0, Unit::Mile, Unit::Kilometer).unwrap();
    assert!((result - 1.609344).abs() < 0.001);
}

#[test]
fn test_convert_kg_to_pounds() {
    let result = convert(1.0, Unit::Kilogram, Unit::Pound).unwrap();
    assert!((result - 2.20462).abs() < 0.01);
}

#[test]
fn test_convert_grams_to_kilograms() {
    let result = convert(1000.0, Unit::Gram, Unit::Kilogram).unwrap();
    assert!((result - 1.0).abs() < 0.001);
}

#[test]
fn test_convert_celsius_to_kelvin() {
    let result = convert(0.0, Unit::Celsius, Unit::Kelvin).unwrap();
    assert!((result - 273.15).abs() < 0.001);
}

#[test]
fn test_convert_kelvin_to_celsius() {
    let result = convert(273.15, Unit::Kelvin, Unit::Celsius).unwrap();
    assert!(result.abs() < 0.001);
}

#[test]
fn test_convert_fahrenheit_to_celsius() {
    let result = convert(32.0, Unit::Fahrenheit, Unit::Celsius).unwrap();
    assert!(result.abs() < 0.001);
}

#[test]
fn test_convert_celsius_to_fahrenheit_body_temp() {
    let result = convert(37.0, Unit::Celsius, Unit::Fahrenheit).unwrap();
    assert!((result - 98.6).abs() < 0.1);
}

#[test]
fn test_convert_below_absolute_zero_celsius() {
    assert!(convert(-274.0, Unit::Celsius, Unit::Kelvin).is_err());
}

#[test]
fn test_convert_below_absolute_zero_fahrenheit() {
    assert!(convert(-500.0, Unit::Fahrenheit, Unit::Kelvin).is_err());
}

#[test]
fn test_convert_speed_kmh_to_mph() {
    let result = convert(100.0, Unit::KilometersPerHour, Unit::MilesPerHour).unwrap();
    assert!((result - 62.137).abs() < 0.1);
}

#[test]
fn test_convert_time_hours_to_seconds() {
    let result = convert(1.0, Unit::Hour, Unit::Second).unwrap();
    assert!((result - 3600.0).abs() < 0.001);
}

#[test]
fn test_convert_time_day_to_hours() {
    let result = convert(1.0, Unit::Day, Unit::Hour).unwrap();
    assert!((result - 24.0).abs() < 0.001);
}

#[test]
fn test_convert_time_minutes_to_seconds() {
    let result = convert(1.0, Unit::Minute, Unit::Second).unwrap();
    assert!((result - 60.0).abs() < 0.001);
}

#[test]
fn test_convert_volume_liters_to_gallons() {
    let result = convert(3.78541, Unit::Liter, Unit::Gallon).unwrap();
    assert!((result - 1.0).abs() < 0.01);
}

#[test]
fn test_convert_volume_ml_to_liters() {
    let result = convert(1000.0, Unit::Milliliter, Unit::Liter).unwrap();
    assert!((result - 1.0).abs() < 0.001);
}

#[test]
fn test_convert_area_hectare_to_sqm() {
    let result = convert(1.0, Unit::Hectare, Unit::SquareMeter).unwrap();
    assert!((result - 10000.0).abs() < 0.001);
}

#[test]
fn test_convert_angle_degrees_to_radians() {
    let result = convert(180.0, Unit::Degree, Unit::Radian).unwrap();
    assert!((result - std::f64::consts::PI).abs() < 0.001);
}

#[test]
fn test_convert_angle_gradians_to_degrees() {
    let result = convert(200.0, Unit::Gradian, Unit::Degree).unwrap();
    assert!((result - 180.0).abs() < 0.001);
}

#[test]
fn test_convert_same_unit() {
    let result = convert(42.0, Unit::Meter, Unit::Meter).unwrap();
    assert!((result - 42.0).abs() < 0.001);
}

#[test]
fn test_convert_incompatible_length_to_mass() {
    assert!(convert(1.0, Unit::Meter, Unit::Kilogram).is_err());
}

#[test]
fn test_convert_incompatible_temp_to_speed() {
    assert!(convert(1.0, Unit::Celsius, Unit::MetersPerSecond).is_err());
}

// =============================================================================
// Constants tests (specific values)
// =============================================================================

#[test]
fn test_constant_pi() {
    let v = Constant::Pi.value();
    assert!((v.to_f64() - std::f64::consts::PI).abs() < 1e-15);
}

#[test]
fn test_constant_e() {
    let v = Constant::E.value();
    assert!((v.to_f64() - std::f64::consts::E).abs() < 1e-15);
}

#[test]
fn test_constant_golden_ratio() {
    let v = Constant::GoldenRatio.value();
    // Golden ratio = (1 + sqrt(5)) / 2 ~ 1.618033988749895
    assert!((v.to_f64() - 1.618033988749895).abs() < 1e-10);
}

#[test]
fn test_constant_sqrt2() {
    let v = Constant::Sqrt2.value();
    assert!((v.to_f64() - std::f64::consts::SQRT_2).abs() < 1e-15);
}

#[test]
fn test_constant_ln2() {
    let v = Constant::Ln2.value();
    assert!((v.to_f64() - std::f64::consts::LN_2).abs() < 1e-15);
}

#[test]
fn test_constant_ln10() {
    let v = Constant::Ln10.value();
    assert!((v.to_f64() - std::f64::consts::LN_10).abs() < 1e-15);
}

#[test]
fn test_constant_names() {
    assert_eq!(Constant::Pi.name(), "π");
    assert_eq!(Constant::E.name(), "e");
    assert_eq!(Constant::GoldenRatio.name(), "φ");
    assert_eq!(Constant::Sqrt2.name(), "√2");
    assert_eq!(Constant::Ln2.name(), "ln(2)");
    assert_eq!(Constant::Ln10.name(), "ln(10)");
}

#[test]
fn test_constant_all_returns_all_variants() {
    let all = Constant::all();
    assert_eq!(all.len(), 6);
    assert!(all.contains(&Constant::Pi));
    assert!(all.contains(&Constant::E));
    assert!(all.contains(&Constant::GoldenRatio));
    assert!(all.contains(&Constant::Sqrt2));
    assert!(all.contains(&Constant::Ln2));
    assert!(all.contains(&Constant::Ln10));
}

#[test]
fn test_constant_all_values_are_float() {
    for c in Constant::all() {
        assert!(matches!(c.value(), Value::Float(_)));
    }
}

// =============================================================================
// Engine / CalcAction tests
// =============================================================================

#[test]
fn test_new_calculator_engine() {
    let engine = new_calculator();
    assert!(engine.situation().display.is_zero());
    assert_eq!(engine.step(), 0);
    assert!(!engine.is_terminal());
}

#[test]
fn test_engine_enter_value() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(42))).unwrap();
    assert_eq!(engine.situation().display, Value::int(42));
    assert_eq!(engine.step(), 1);
}

#[test]
fn test_engine_unary_action() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(5))).unwrap();
    let engine = engine.next(CalcAction::Unary(UnaryOp::Square)).unwrap();
    assert_eq!(engine.situation().display, Value::int(25));
}

#[test]
fn test_engine_binary_action() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(10))).unwrap();
    let engine = engine
        .next(CalcAction::Binary(BinaryOp::Add, Value::int(5)))
        .unwrap();
    assert_eq!(engine.situation().display, Value::int(15));
}

#[test]
fn test_engine_clear() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(42))).unwrap();
    let engine = engine.next(CalcAction::Clear).unwrap();
    assert!(engine.situation().display.is_zero());
}

#[test]
fn test_engine_all_clear() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(42))).unwrap();
    let engine = engine.next(CalcAction::StoreMemory).unwrap();
    let engine = engine.next(CalcAction::AllClear).unwrap();
    assert!(engine.situation().display.is_zero());
    assert!(engine.situation().memory.is_zero());
}

#[test]
fn test_engine_memory_store_recall() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(99))).unwrap();
    let engine = engine.next(CalcAction::StoreMemory).unwrap();
    let engine = engine.next(CalcAction::Enter(Value::int(0))).unwrap();
    let engine = engine.next(CalcAction::RecallMemory).unwrap();
    assert_eq!(engine.situation().display, Value::int(99));
}

#[test]
fn test_engine_memory_add() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(10))).unwrap();
    let engine = engine.next(CalcAction::StoreMemory).unwrap();
    let engine = engine.next(CalcAction::Enter(Value::int(5))).unwrap();
    let engine = engine.next(CalcAction::AddToMemory).unwrap();
    let engine = engine.next(CalcAction::RecallMemory).unwrap();
    assert_eq!(engine.situation().display, Value::int(15));
}

#[test]
fn test_engine_memory_clear() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(42))).unwrap();
    let engine = engine.next(CalcAction::StoreMemory).unwrap();
    let engine = engine.next(CalcAction::ClearMemory).unwrap();
    assert!(engine.situation().memory.is_zero());
}

#[test]
fn test_engine_set_angle_mode() {
    let engine = new_calculator();
    let engine = engine
        .next(CalcAction::SetAngleMode(AngleMode::Degrees))
        .unwrap();
    assert_eq!(engine.situation().angle_mode, AngleMode::Degrees);
}

#[test]
fn test_engine_domain_check_blocks_div_by_zero() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(10))).unwrap();
    let result = engine.next(CalcAction::Binary(BinaryOp::Divide, Value::int(0)));
    assert!(result.is_err());
}

#[test]
fn test_engine_domain_check_blocks_sqrt_negative() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(-4))).unwrap();
    let result = engine.next(CalcAction::Unary(UnaryOp::Sqrt));
    assert!(result.is_err());
}

#[test]
fn test_engine_domain_check_blocks_ln_zero() {
    let engine = new_calculator();
    // display starts at 0
    let result = engine.next(CalcAction::Unary(UnaryOp::Ln));
    assert!(result.is_err());
}

#[test]
fn test_engine_back_and_forward() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(10))).unwrap();
    let engine = engine
        .next(CalcAction::Binary(BinaryOp::Add, Value::int(5)))
        .unwrap();
    assert_eq!(engine.situation().display, Value::int(15));

    // Go back
    let engine = engine.back().unwrap();
    assert_eq!(engine.situation().display, Value::int(10));
    assert_eq!(engine.forward_depth(), 1);

    // Go forward
    let engine = engine.forward().unwrap();
    assert_eq!(engine.situation().display, Value::int(15));
}

#[test]
fn test_engine_back_at_start_fails() {
    let engine = new_calculator();
    assert!(engine.back().is_err());
}

#[test]
fn test_engine_forward_without_back_fails() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(10))).unwrap();
    assert!(engine.forward().is_err());
}

#[test]
fn test_engine_trace() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(10))).unwrap();
    let engine = engine
        .next(CalcAction::Binary(BinaryOp::Add, Value::int(5)))
        .unwrap();
    assert_eq!(engine.trace().successful_steps(), 2);
    assert_eq!(engine.trace().violations(), 0);
}

#[test]
fn test_engine_trace_records_violations() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(-4))).unwrap();
    let (engine, violations) = engine.next(CalcAction::Unary(UnaryOp::Sqrt)).unwrap_err();
    assert!(!violations.is_empty());
    assert_eq!(engine.trace().violations(), 1);
}

#[test]
fn test_engine_try_next_success() {
    let engine = new_calculator();
    let engine = engine.try_next(CalcAction::Enter(Value::int(42))).unwrap();
    assert_eq!(engine.situation().display, Value::int(42));
}

#[test]
fn test_engine_try_next_failure() {
    let engine = new_calculator();
    let engine = engine.try_next(CalcAction::Enter(Value::int(-1))).unwrap();
    let result = engine.try_next(CalcAction::Unary(UnaryOp::Sqrt));
    assert!(result.is_err());
    let errs = result.unwrap_err();
    assert!(!errs.is_empty());
}

#[test]
fn test_engine_multi_step_calculation() {
    // Compute: (5 + 3) * 2 - 1 = 15
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(5))).unwrap();
    let engine = engine
        .next(CalcAction::Binary(BinaryOp::Add, Value::int(3)))
        .unwrap();
    let engine = engine
        .next(CalcAction::Binary(BinaryOp::Multiply, Value::int(2)))
        .unwrap();
    let engine = engine
        .next(CalcAction::Binary(BinaryOp::Subtract, Value::int(1)))
        .unwrap();
    assert_eq!(engine.situation().display, Value::int(15));
    assert_eq!(engine.step(), 4);
}

#[test]
fn test_calc_action_describe() {
    assert_eq!(CalcAction::Enter(Value::int(42)).describe(), "enter 42");
    assert_eq!(CalcAction::Unary(UnaryOp::Sqrt).describe(), "Sqrt");
    assert_eq!(
        CalcAction::Binary(BinaryOp::Add, Value::int(5)).describe(),
        "Add 5"
    );
    assert_eq!(CalcAction::Clear.describe(), "clear");
    assert_eq!(CalcAction::AllClear.describe(), "all clear");
    assert_eq!(CalcAction::StoreMemory.describe(), "M store");
    assert_eq!(CalcAction::RecallMemory.describe(), "M recall");
    assert_eq!(CalcAction::AddToMemory.describe(), "M+");
    assert_eq!(CalcAction::ClearMemory.describe(), "MC");
    assert_eq!(
        CalcAction::SetAngleMode(AngleMode::Degrees).describe(),
        "angle mode Degrees"
    );
}

#[test]
fn test_calculator_situation_describe() {
    use praxis::engine::Situation;
    let calc = Calculator::new();
    let desc = calc.describe();
    assert!(desc.contains("display=0"));
    assert!(desc.contains("memory=0"));
    assert!(desc.contains("Radians"));
}

#[test]
fn test_calculator_situation_not_terminal() {
    use praxis::engine::Situation;
    let calc = Calculator::new();
    assert!(!calc.is_terminal());
}

// =============================================================================
// Value helper tests (additional coverage)
// =============================================================================

#[test]
fn test_value_is_one_rational() {
    assert!(Value::int(1).is_one());
    assert!(Value::Rational(3, 3).is_one());
    assert!(!Value::int(2).is_one());
}

#[test]
fn test_value_is_one_float() {
    assert!(Value::Float(1.0).is_one());
    assert!(!Value::Float(1.1).is_one());
}

#[test]
fn test_value_is_negative() {
    assert!(Value::int(-1).is_negative());
    assert!(!Value::int(0).is_negative());
    assert!(!Value::int(1).is_negative());
    assert!(Value::Float(-0.5).is_negative());
    assert!(!Value::Float(0.5).is_negative());
}

#[test]
fn test_value_negate_rational() {
    assert_eq!(Value::int(5).negate(), Value::int(-5));
    assert_eq!(Value::int(-3).negate(), Value::int(3));
}

#[test]
fn test_value_negate_float() {
    assert_eq!(Value::Float(2.5).negate(), Value::Float(-2.5));
}

#[test]
fn test_value_reciprocal_rational() {
    let v = Value::rational(3, 4).unwrap();
    let r = v.reciprocal().unwrap();
    assert_eq!(r, Value::Rational(4, 3));
}

#[test]
fn test_value_reciprocal_zero_rational() {
    assert_eq!(Value::int(0).reciprocal(), Err(CalcError::DivisionByZero));
}

#[test]
fn test_value_reciprocal_float() {
    let v = Value::Float(4.0);
    let r = v.reciprocal().unwrap();
    assert!((r.to_f64() - 0.25).abs() < 1e-10);
}

#[test]
fn test_value_reciprocal_zero_float() {
    assert_eq!(
        Value::Float(0.0).reciprocal(),
        Err(CalcError::DivisionByZero)
    );
}

#[test]
fn test_value_float_overflow() {
    assert_eq!(Value::float(f64::INFINITY), Err(CalcError::Overflow));
}

#[test]
fn test_value_float_nan() {
    assert!(Value::float(f64::NAN).is_err());
}

#[test]
fn test_value_float_display() {
    assert_eq!(format!("{}", Value::Float(3.14)), "3.14");
}

#[test]
fn test_gcd() {
    assert_eq!(gcd(12, 8), 4);
    assert_eq!(gcd(7, 13), 1);
    assert_eq!(gcd(0, 5), 5);
    assert_eq!(gcd(0, 0), 1); // gcd returns max(a, 1) when a=0, b=0
}

#[test]
fn test_lcm() {
    assert_eq!(lcm(4, 6), 12);
    assert_eq!(lcm(3, 5), 15);
    assert_eq!(lcm(7, 7), 7);
}

#[test]
fn test_calc_error_display() {
    assert_eq!(format!("{}", CalcError::DivisionByZero), "division by zero");
    assert_eq!(
        format!("{}", CalcError::NegativeSquareRoot),
        "square root of negative number"
    );
    assert_eq!(
        format!("{}", CalcError::LogOfNonPositive),
        "logarithm of non-positive number"
    );
    assert_eq!(
        format!("{}", CalcError::TanUndefined),
        "tangent undefined at this angle"
    );
    assert_eq!(format!("{}", CalcError::Overflow), "result too large");
    assert_eq!(format!("{}", CalcError::Underflow), "result too small");
    assert_eq!(
        format!(
            "{}",
            CalcError::InvalidDomain {
                op: "test".into(),
                value: 42.0
            }
        ),
        "test undefined for 42"
    );
}

// =============================================================================
// NumberDomainCheck engine integration tests
// =============================================================================

#[test]
fn test_number_domain_factorial_on_negative() {
    // Factorial on a negative integer should be blocked by domain check
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(-3))).unwrap();
    let result = engine.next(CalcAction::Unary(UnaryOp::Factorial));
    assert!(result.is_err());
}

#[test]
fn test_number_domain_factorial_on_natural() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(5))).unwrap();
    let engine = engine.next(CalcAction::Unary(UnaryOp::Factorial)).unwrap();
    assert_eq!(engine.situation().display, Value::int(120));
}

#[test]
fn test_number_domain_divide_valid() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(10))).unwrap();
    let engine = engine
        .next(CalcAction::Binary(BinaryOp::Divide, Value::int(2)))
        .unwrap();
    assert_eq!(engine.situation().display, Value::int(5));
}

#[test]
fn test_number_domain_log_of_negative() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(-5))).unwrap();
    let result = engine.next(CalcAction::Unary(UnaryOp::Ln));
    assert!(result.is_err());
}

#[test]
fn test_number_domain_log10_of_negative() {
    let engine = new_calculator();
    let engine = engine.next(CalcAction::Enter(Value::int(-5))).unwrap();
    let result = engine.next(CalcAction::Unary(UnaryOp::Log10));
    assert!(result.is_err());
}

#[test]
fn test_number_domain_log2_of_zero() {
    let engine = new_calculator();
    // display starts at 0
    let result = engine.next(CalcAction::Unary(UnaryOp::Log2));
    assert!(result.is_err());
}

#[test]
fn test_engine_angle_mode_with_trig() {
    let engine = new_calculator();
    let engine = engine
        .next(CalcAction::SetAngleMode(AngleMode::Degrees))
        .unwrap();
    let engine = engine.next(CalcAction::Enter(Value::Float(30.0))).unwrap();
    let engine = engine.next(CalcAction::Unary(UnaryOp::Sin)).unwrap();
    assert!((engine.situation().display.to_f64() - 0.5).abs() < 1e-10);
}

// =============================================================================
// DomainCheck describe tests
// =============================================================================

#[test]
fn test_domain_check_describe() {
    use super::engine::DomainCheck;
    use praxis::engine::Precondition;
    let dc = DomainCheck;
    assert!(dc.describe().contains("domain"));
}

#[test]
fn test_number_domain_check_describe() {
    use super::engine::NumberDomainCheck;
    use praxis::engine::Precondition;
    let ndc = NumberDomainCheck;
    assert!(ndc.describe().contains("domain hierarchy"));
}

// =============================================================================
// DomainCheck precondition satisfied cases
// =============================================================================

#[test]
fn test_domain_check_satisfied_for_non_math_actions() {
    use super::engine::DomainCheck;
    use praxis::engine::Precondition;
    let calc = Calculator::new();
    let dc = DomainCheck;
    let result = dc.check(&calc, &CalcAction::Clear);
    assert!(result.is_satisfied());
    let result = dc.check(&calc, &CalcAction::AllClear);
    assert!(result.is_satisfied());
    let result = dc.check(&calc, &CalcAction::StoreMemory);
    assert!(result.is_satisfied());
    let result = dc.check(&calc, &CalcAction::RecallMemory);
    assert!(result.is_satisfied());
    let result = dc.check(&calc, &CalcAction::SetAngleMode(AngleMode::Degrees));
    assert!(result.is_satisfied());
}

#[test]
fn test_domain_check_satisfied_for_valid_unary() {
    use super::engine::DomainCheck;
    use praxis::engine::Precondition;
    let mut calc = Calculator::new();
    calc.enter(Value::int(4));
    let dc = DomainCheck;
    let result = dc.check(&calc, &CalcAction::Unary(UnaryOp::Sqrt));
    assert!(result.is_satisfied());
}

#[test]
fn test_domain_check_violated_for_invalid_unary() {
    use super::engine::DomainCheck;
    use praxis::engine::Precondition;
    let mut calc = Calculator::new();
    calc.enter(Value::int(-4));
    let dc = DomainCheck;
    let result = dc.check(&calc, &CalcAction::Unary(UnaryOp::Sqrt));
    assert!(!result.is_satisfied());
}

#[test]
fn test_domain_check_satisfied_for_valid_binary() {
    use super::engine::DomainCheck;
    use praxis::engine::Precondition;
    let mut calc = Calculator::new();
    calc.enter(Value::int(10));
    let dc = DomainCheck;
    let result = dc.check(&calc, &CalcAction::Binary(BinaryOp::Add, Value::int(5)));
    assert!(result.is_satisfied());
}

#[test]
fn test_domain_check_violated_for_div_by_zero() {
    use super::engine::DomainCheck;
    use praxis::engine::Precondition;
    let mut calc = Calculator::new();
    calc.enter(Value::int(10));
    let dc = DomainCheck;
    let result = dc.check(&calc, &CalcAction::Binary(BinaryOp::Divide, Value::int(0)));
    assert!(!result.is_satisfied());
}
