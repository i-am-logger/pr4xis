use super::*;
use proptest::prelude::*;

fn arb_method() -> impl Strategy<Value = Method> {
    (0..7usize).prop_map(|i| Method::all()[i])
}

fn arb_action() -> impl Strategy<Value = ConnectionAction> {
    prop_oneof![
        Just(ConnectionAction::Connect),
        Just(ConnectionAction::SendRequest),
        Just(ConnectionAction::ReceiveResponse),
        Just(ConnectionAction::Complete),
        Just(ConnectionAction::Retry),
        Just(ConnectionAction::Close),
        Just(ConnectionAction::Reset),
    ]
}

#[test]
fn test_safe_methods() {
    assert!(Method::Get.is_safe());
    assert!(Method::Head.is_safe());
    assert!(!Method::Post.is_safe());
}

#[test]
fn test_idempotent_methods() {
    assert!(Method::Get.is_idempotent());
    assert!(Method::Put.is_idempotent());
    assert!(!Method::Post.is_idempotent());
}

#[test]
fn test_cant_add_body_to_get() {
    let req = Request::new(Method::Get, "/api");
    assert!(req.with_body(vec![1]).is_err());
}

#[test]
fn test_can_add_body_to_post() {
    let req = Request::new(Method::Post, "/api");
    assert!(req.with_body(vec![1]).is_ok());
}

#[test]
fn test_status_classes() {
    assert_eq!(StatusCode::OK.class(), StatusClass::Success);
    assert_eq!(StatusCode::NOT_FOUND.class(), StatusClass::ClientError);
    assert!(StatusCode::new(99).is_err());
}

#[test]
fn test_connection_happy_path() {
    let conn = Connection::new(3)
        .apply(ConnectionAction::Connect)
        .unwrap()
        .apply(ConnectionAction::SendRequest)
        .unwrap()
        .apply(ConnectionAction::ReceiveResponse)
        .unwrap()
        .apply(ConnectionAction::ReceiveResponse)
        .unwrap()
        .apply(ConnectionAction::Complete)
        .unwrap()
        .apply(ConnectionAction::Close)
        .unwrap();
    assert!(conn.is_terminal());
}

#[test]
fn test_cant_send_before_connect() {
    assert!(
        Connection::new(3)
            .apply(ConnectionAction::SendRequest)
            .is_err()
    );
}

#[test]
fn test_error_and_retry() {
    let conn = Connection::new(3)
        .apply(ConnectionAction::Connect)
        .unwrap()
        .apply(ConnectionAction::Reset)
        .unwrap();
    assert!(conn.can_retry());
    let conn = conn.apply(ConnectionAction::Retry).unwrap();
    assert_eq!(conn.retries, 1);
}

#[test]
fn test_max_retries_exhausted() {
    let conn = Connection::new(1)
        .apply(ConnectionAction::Connect)
        .unwrap()
        .apply(ConnectionAction::Reset)
        .unwrap()
        .apply(ConnectionAction::Retry)
        .unwrap()
        .apply(ConnectionAction::Reset)
        .unwrap();
    assert!(!conn.can_retry());
    assert!(conn.apply(ConnectionAction::Retry).is_err());
}

#[test]
fn test_closed_is_terminal() {
    let conn = Connection::new(3).apply(ConnectionAction::Close).unwrap();
    assert!(conn.is_terminal());
    assert!(conn.apply(ConnectionAction::Connect).is_err());
}

proptest! {
    #[test]
    fn prop_safe_is_idempotent(method in arb_method()) {
        if method.is_safe() {
            prop_assert!(method.is_idempotent());
        }
    }

    #[test]
    fn prop_body_not_safe(method in arb_method()) {
        if method.has_body() {
            prop_assert!(!method.is_safe());
        }
    }

    #[test]
    fn prop_body_enforcement(method in arb_method()) {
        let req = Request::new(method, "/test");
        let result = req.with_body(vec![1]);
        prop_assert_eq!(result.is_ok(), method.has_body());
    }

    #[test]
    fn prop_status_class_deterministic(code in 100..600u16) {
        let s = StatusCode::new(code).unwrap();
        prop_assert_eq!(s.class(), s.class());
    }

    #[test]
    fn prop_status_categories(code in 100..600u16) {
        let s = StatusCode::new(code).unwrap();
        match code {
            200..=299 => prop_assert!(s.is_success()),
            400..=599 => prop_assert!(s.is_error()),
            300..=399 => prop_assert!(s.is_redirect()),
            _ => {}
        }
    }

    #[test]
    fn prop_invalid_status_rejected(code in prop_oneof![0..100u16, 600..1000u16]) {
        prop_assert!(StatusCode::new(code).is_err());
    }

    #[test]
    fn prop_starts_idle(retries in 0..10u32) {
        prop_assert_eq!(Connection::new(retries).state, ConnectionState::Idle);
    }

    #[test]
    fn prop_closed_rejects_all(action in arb_action()) {
        let closed = Connection::new(3).apply(ConnectionAction::Close).unwrap();
        prop_assert!(closed.apply(action).is_err());
    }

    #[test]
    fn prop_retry_increments(max in 1..10u32) {
        let conn = Connection::new(max)
            .apply(ConnectionAction::Connect).unwrap()
            .apply(ConnectionAction::Reset).unwrap()
            .apply(ConnectionAction::Retry).unwrap();
        prop_assert_eq!(conn.retries, 1);
    }

    #[test]
    fn prop_retries_bounded(max in 1..5u32) {
        let mut conn = Connection::new(max);
        for _ in 0..max + 5 {
            if let Ok(c) = conn.apply(ConnectionAction::Connect) { conn = c; }
            if let Ok(c) = conn.apply(ConnectionAction::Reset) { conn = c; }
            if let Ok(c) = conn.apply(ConnectionAction::Retry) { conn = c; }
        }
        prop_assert!(conn.retries <= max);
    }

    #[test]
    fn prop_keep_alive_reuse(_x in 0..1u8) {
        let mut conn = Connection::new(3);
        conn.keep_alive = true;
        let conn = conn.apply(ConnectionAction::Connect).unwrap()
            .apply(ConnectionAction::SendRequest).unwrap()
            .apply(ConnectionAction::ReceiveResponse).unwrap()
            .apply(ConnectionAction::ReceiveResponse).unwrap()
            .apply(ConnectionAction::Complete).unwrap();
        prop_assert!(conn.apply(ConnectionAction::SendRequest).is_ok());
    }
}

// =============================================================================
// Engine tests — Situation/Action/Precondition/Trace
// =============================================================================

use praxis::engine::*;

#[test]
fn engine_full_request_cycle() {
    let e = new_connection(3);
    // Idle → Connecting → SendingRequest → AwaitingResponse → ReceivingResponse → Complete → Closed
    let e = e.try_next(HttpAction(ConnectionAction::Connect)).unwrap();
    let e = e
        .try_next(HttpAction(ConnectionAction::SendRequest))
        .unwrap();
    let e = e
        .try_next(HttpAction(ConnectionAction::ReceiveResponse))
        .unwrap(); // Sending → Awaiting
    let e = e
        .try_next(HttpAction(ConnectionAction::ReceiveResponse))
        .unwrap(); // Awaiting → Receiving
    let e = e.try_next(HttpAction(ConnectionAction::Complete)).unwrap();
    let e = e.try_next(HttpAction(ConnectionAction::Close)).unwrap();
    assert!(e.is_terminal());
    assert_eq!(e.step(), 6);
}

#[test]
fn engine_invalid_transition_rejected() {
    let e = new_connection(3);
    // Can't send request before connecting
    let result = e.try_next(HttpAction(ConnectionAction::SendRequest));
    assert!(result.is_err());
}

#[test]
fn engine_back_forward() {
    let e = new_connection(3);
    let e = e.try_next(HttpAction(ConnectionAction::Connect)).unwrap();
    let e = e
        .try_next(HttpAction(ConnectionAction::SendRequest))
        .unwrap();
    let e = e.back().unwrap();
    assert_eq!(e.step(), 1);
    let e = e.forward().unwrap();
    assert_eq!(e.step(), 2);
}

#[test]
fn engine_trace_on_failure() {
    let e = new_connection(3);
    let (e, _) = e
        .next(HttpAction(ConnectionAction::SendRequest))
        .unwrap_err();
    // Trace records the failed attempt
    assert_eq!(e.trace().entries.len(), 1);
    assert!(!e.trace().entries[0].success);
}
