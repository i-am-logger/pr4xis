use crate::formal::math::linear_algebra::matrix::Matrix;

/// Motion models for target tracking.
///
/// Each model defines a state transition matrix F(dt) and process noise Q(dt).
///
/// Source: Bar-Shalom et al. (2001), Chapter 6.
/// Constant velocity model (nearly constant velocity, NCV).
///
/// State: [x, vx] (1D) or [x, vx, y, vy] (2D).
/// F = [[1, dt], [0, 1]]
/// Q = q * [[dt³/3, dt²/2], [dt²/2, dt]]
///
/// Source: Bar-Shalom (2001), Eq. 6.2.2-4.
pub fn constant_velocity_1d(dt: f64, q: f64) -> (Matrix, Matrix) {
    let f = Matrix::new(2, 2, vec![1.0, dt, 0.0, 1.0]);
    let process_noise = Matrix::new(
        2,
        2,
        vec![
            q * dt * dt * dt / 3.0,
            q * dt * dt / 2.0,
            q * dt * dt / 2.0,
            q * dt,
        ],
    );
    (f, process_noise)
}

/// Constant acceleration model (nearly constant acceleration, NCA).
///
/// State: [x, vx, ax] (1D).
/// F = [[1, dt, dt²/2], [0, 1, dt], [0, 0, 1]]
///
/// Source: Bar-Shalom (2001), Eq. 6.2.3-1.
pub fn constant_acceleration_1d(dt: f64, q: f64) -> (Matrix, Matrix) {
    let dt2 = dt * dt;
    let dt3 = dt2 * dt;
    let dt4 = dt3 * dt;
    let dt5 = dt4 * dt;
    let f = Matrix::new(3, 3, vec![1.0, dt, dt2 / 2.0, 0.0, 1.0, dt, 0.0, 0.0, 1.0]);
    let process_noise = Matrix::new(
        3,
        3,
        vec![
            q * dt5 / 20.0,
            q * dt4 / 8.0,
            q * dt3 / 6.0,
            q * dt4 / 8.0,
            q * dt3 / 3.0,
            q * dt2 / 2.0,
            q * dt3 / 6.0,
            q * dt2 / 2.0,
            q * dt,
        ],
    );
    (f, process_noise)
}

/// 2D constant velocity model.
///
/// State: [x, vx, y, vy].
/// Decoupled: x and y axes are independent.
pub fn constant_velocity_2d(dt: f64, q: f64) -> (Matrix, Matrix) {
    let (f1d, q1d) = constant_velocity_1d(dt, q);
    // Block diagonal: F = diag(F1d, F1d), Q = diag(Q1d, Q1d)
    let mut f = Matrix::zeros(4, 4);
    let mut qm = Matrix::zeros(4, 4);
    for i in 0..2 {
        for j in 0..2 {
            f.set(i, j, f1d.get(i, j));
            f.set(i + 2, j + 2, f1d.get(i, j));
            qm.set(i, j, q1d.get(i, j));
            qm.set(i + 2, j + 2, q1d.get(i, j));
        }
    }
    (f, qm)
}
