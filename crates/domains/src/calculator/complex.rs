use super::value::CalcError;
use std::fmt;

/// Complex number: a + bi.
#[derive(Debug, Clone, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }

    pub fn imaginary(im: f64) -> Self {
        Self { re: 0.0, im }
    }

    pub const I: Complex = Complex { re: 0.0, im: 1.0 };
    pub const ZERO: Complex = Complex { re: 0.0, im: 0.0 };
    pub const ONE: Complex = Complex { re: 1.0, im: 0.0 };

    /// Magnitude (absolute value): |a + bi| = sqrt(a² + b²).
    pub fn magnitude(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    /// Phase angle in radians.
    pub fn phase(&self) -> f64 {
        self.im.atan2(self.re)
    }

    /// Complex conjugate: a - bi.
    pub fn conjugate(&self) -> Complex {
        Complex::new(self.re, -self.im)
    }

    /// Is this a real number (imaginary part is zero)?
    pub fn is_real(&self) -> bool {
        self.im.abs() < 1e-10
    }

    /// Is this purely imaginary?
    pub fn is_imaginary(&self) -> bool {
        self.re.abs() < 1e-10 && self.im.abs() > 1e-10
    }

    pub fn add(&self, other: &Complex) -> Complex {
        Complex::new(self.re + other.re, self.im + other.im)
    }

    pub fn sub(&self, other: &Complex) -> Complex {
        Complex::new(self.re - other.re, self.im - other.im)
    }

    pub fn mul(&self, other: &Complex) -> Complex {
        Complex::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }

    pub fn div(&self, other: &Complex) -> Result<Complex, CalcError> {
        let denom = other.re * other.re + other.im * other.im;
        if denom < 1e-15 {
            return Err(CalcError::DivisionByZero);
        }
        Ok(Complex::new(
            (self.re * other.re + self.im * other.im) / denom,
            (self.im * other.re - self.re * other.im) / denom,
        ))
    }

    /// Square root of a complex number.
    /// sqrt(-1) = i (works where real sqrt fails).
    pub fn sqrt(&self) -> Complex {
        let mag = self.magnitude();
        let re = ((mag + self.re) / 2.0).sqrt();
        let im = ((mag - self.re) / 2.0).sqrt();
        Complex::new(re, if self.im >= 0.0 { im } else { -im })
    }

    /// Complex exponential: e^(a+bi) = e^a * (cos(b) + i*sin(b)).
    pub fn exp(&self) -> Complex {
        let ea = self.re.exp();
        Complex::new(ea * self.im.cos(), ea * self.im.sin())
    }

    /// Complex natural log: ln(z) = ln|z| + i*arg(z).
    pub fn ln(&self) -> Result<Complex, CalcError> {
        let mag = self.magnitude();
        if mag < 1e-15 {
            return Err(CalcError::LogOfNonPositive);
        }
        Ok(Complex::new(mag.ln(), self.phase()))
    }

    /// Negate: -(a+bi) = -a - bi.
    pub fn negate(&self) -> Complex {
        Complex::new(-self.re, -self.im)
    }

    /// Power: z^w for complex z and w.
    pub fn pow(&self, w: &Complex) -> Result<Complex, CalcError> {
        if self.magnitude() < 1e-15 {
            if w.re > 0.0 {
                return Ok(Complex::ZERO);
            }
            return Err(CalcError::DivisionByZero);
        }
        let ln_z = self.ln()?;
        Ok(w.mul(&ln_z).exp())
    }
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.im.abs() < 1e-10 {
            write!(f, "{}", self.re)
        } else if self.re.abs() < 1e-10 {
            write!(f, "{}i", self.im)
        } else if self.im > 0.0 {
            write!(f, "{} + {}i", self.re, self.im)
        } else {
            write!(f, "{} - {}i", self.re, -self.im)
        }
    }
}
