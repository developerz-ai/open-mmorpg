//! The per-connection **lifecycle** state machine: `Hello` → accept/reject,
//! `Ping`/`Pong` keepalive, and handshake/idle timeouts.
//!
//! This is a pure, clock-free machine driven by decoded [`ClientMsg`]s and a
//! caller-supplied monotonic `now_ms`. Keeping it free of sockets, wire codecs
//! and wall clocks is deliberate: the whole lifecycle — every accept, reject and
//! timeout edge — is deterministically unit-tested, and the shard above supplies
//! the real socket, the token check, and the clock. It never verifies tokens
//! itself; that is the gateway/persistence layer's job, injected as `accept`.
//!
//! [`ConnState`] carries exactly the data each phase needs and no more, so the
//! illegal combinations (Active with no activity stamp, Closed with no reason)
//! cannot be constructed — the type system is the first anti-cheat layer.

use omm_protocol::{ClientMsg, Intent, Tick};

use omm_netcode::transport::ConnId;

/// Why a connection reached [`ConnState::Closed`]. Server-side only — a client
/// sees at most a generic rejection code, never these internal reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseReason {
    /// The `Hello` token was refused by the caller's `accept` check.
    Rejected,
    /// No `Hello` arrived before the handshake deadline.
    HelloTimeout,
    /// No traffic arrived within the idle window on an active session.
    IdleTimeout,
    /// A message illegal for the current phase (e.g. gameplay before `Hello`).
    ProtocolError,
    /// The transport reported the peer gone.
    PeerClosed,
}

/// The lifecycle phase of one connection. Each variant holds only what its phase
/// needs, making illegal states unrepresentable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnState {
    /// Opened; awaiting the client's `Hello`. Closed if none arrives by
    /// `deadline_ms`.
    AwaitingHello {
        /// Monotonic ms at which the unanswered handshake times out.
        deadline_ms: u64,
    },
    /// `Hello` accepted. `last_activity_ms` is refreshed by every client message
    /// and drives the idle timeout.
    Active {
        /// Monotonic ms of the most recent client message.
        last_activity_ms: u64,
    },
    /// Terminal. No further transitions; carries why it closed.
    Closed {
        /// Why the connection ended.
        reason: CloseReason,
    },
}

/// Handshake and idle timeout windows, in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeouts {
    /// How long to wait for the client's `Hello` before closing.
    pub hello_ms: u64,
    /// How long an active connection may go silent before closing.
    pub idle_ms: u64,
}

impl Timeouts {
    /// Default handshake window: 5s to say `Hello`.
    pub const DEFAULT_HELLO_MS: u64 = 5_000;
    /// Default idle window: 15s of silence closes an active connection. Comfortably
    /// above a sane keepalive cadence (a `Ping` every few seconds).
    pub const DEFAULT_IDLE_MS: u64 = 15_000;
}

impl Default for Timeouts {
    fn default() -> Self {
        Self {
            hello_ms: Self::DEFAULT_HELLO_MS,
            idle_ms: Self::DEFAULT_IDLE_MS,
        }
    }
}

/// What the driver must do in response to a client message. The lifecycle layer
/// owns handshake and keepalive; gameplay [`Intent`]s pass straight through.
#[derive(Debug, Clone, PartialEq)]
pub enum ConnEvent {
    /// `Hello` accepted — the connection is now [`ConnState::Active`].
    Accepted,
    /// `Hello` refused — send a rejection then drop the connection.
    Rejected,
    /// Keepalive `Ping`; reply `ServerMsg::Pong` with this nonce.
    Pong(u64),
    /// A gameplay input for the simulation above; the lifecycle doesn't touch it.
    Input {
        /// The client's last-seen tick, stamped on the intent.
        tick: Tick,
        /// The validated-by-sim-later intent.
        intent: Intent,
    },
    /// The message was illegal for the current phase; the connection is Closed.
    ProtocolViolation,
    /// The connection is already Closed; the message was ignored.
    Ignored,
}

/// One connection's lifecycle. Holds its [`ConnState`] and timeout config; every
/// transition is driven by [`Connection::on_msg`] and [`Connection::poll_timeout`].
#[derive(Debug, Clone)]
pub struct Connection {
    conn: ConnId,
    cfg: Timeouts,
    state: ConnState,
}

impl Connection {
    /// Open a fresh connection awaiting `Hello`, arming the handshake deadline at
    /// `now_ms + cfg.hello_ms`.
    #[must_use]
    pub fn open(conn: ConnId, cfg: Timeouts, now_ms: u64) -> Self {
        Self {
            conn,
            cfg,
            state: ConnState::AwaitingHello {
                deadline_ms: now_ms.saturating_add(cfg.hello_ms),
            },
        }
    }

    /// The connection this lifecycle serves.
    #[must_use]
    pub const fn conn_id(&self) -> ConnId {
        self.conn
    }

    /// The current lifecycle phase.
    #[must_use]
    pub const fn state(&self) -> ConnState {
        self.state
    }

    /// Whether the handshake has completed and gameplay may flow.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.state, ConnState::Active { .. })
    }

    /// Whether the connection has reached its terminal phase.
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        matches!(self.state, ConnState::Closed { .. })
    }

    /// Process one decoded client message at `now_ms`. `accept` is consulted only
    /// for `Hello` and decides whether the session token is good — it is never
    /// this layer's job to verify credentials.
    pub fn on_msg(
        &mut self,
        msg: &ClientMsg,
        now_ms: u64,
        accept: impl FnOnce(&str) -> bool,
    ) -> ConnEvent {
        match (self.state, msg) {
            (ConnState::AwaitingHello { .. }, ClientMsg::Hello { token }) => {
                if accept(token) {
                    self.state = ConnState::Active {
                        last_activity_ms: now_ms,
                    };
                    ConnEvent::Accepted
                } else {
                    self.close(CloseReason::Rejected);
                    ConnEvent::Rejected
                }
            }
            // Anything spoken before a successful `Hello` is a violation.
            (ConnState::AwaitingHello { .. }, _) => {
                self.close(CloseReason::ProtocolError);
                ConnEvent::ProtocolViolation
            }
            (ConnState::Active { .. }, ClientMsg::Ping { nonce }) => {
                self.touch(now_ms);
                ConnEvent::Pong(*nonce)
            }
            (ConnState::Active { .. }, ClientMsg::Input { tick, intent }) => {
                self.touch(now_ms);
                ConnEvent::Input {
                    tick: *tick,
                    intent: intent.clone(),
                }
            }
            // A second `Hello` on an established session is a violation.
            (ConnState::Active { .. }, ClientMsg::Hello { .. }) => {
                self.close(CloseReason::ProtocolError);
                ConnEvent::ProtocolViolation
            }
            (ConnState::Closed { .. }, _) => ConnEvent::Ignored,
        }
    }

    /// Advance timers to `now_ms`. Returns `Some(reason)` iff this call closed the
    /// connection — the driver then tears the transport down.
    pub fn poll_timeout(&mut self, now_ms: u64) -> Option<CloseReason> {
        match self.state {
            ConnState::AwaitingHello { deadline_ms } if now_ms >= deadline_ms => {
                self.close(CloseReason::HelloTimeout);
                Some(CloseReason::HelloTimeout)
            }
            ConnState::Active { last_activity_ms }
                if now_ms.saturating_sub(last_activity_ms) >= self.cfg.idle_ms =>
            {
                self.close(CloseReason::IdleTimeout);
                Some(CloseReason::IdleTimeout)
            }
            _ => None,
        }
    }

    /// Mark the connection closed because the transport reported the peer gone.
    pub fn on_peer_closed(&mut self) {
        self.close(CloseReason::PeerClosed);
    }

    fn touch(&mut self, now_ms: u64) {
        if let ConnState::Active { last_activity_ms } = &mut self.state {
            *last_activity_ms = now_ms;
        }
    }

    fn close(&mut self, reason: CloseReason) {
        if !self.is_closed() {
            self.state = ConnState::Closed { reason };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omm_protocol::Vec3;

    fn conn(now: u64) -> Connection {
        Connection::open(ConnId::new(1), Timeouts::default(), now)
    }

    fn hello() -> ClientMsg {
        ClientMsg::Hello {
            token: "tok".into(),
        }
    }

    fn move_input(tick: u64) -> ClientMsg {
        ClientMsg::Input {
            tick: Tick(tick),
            intent: Intent::Move {
                dir: Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        }
    }

    #[test]
    fn opens_awaiting_hello_with_armed_deadline() {
        let c = conn(1_000);
        assert_eq!(c.conn_id(), ConnId::new(1));
        assert_eq!(
            c.state(),
            ConnState::AwaitingHello {
                deadline_ms: 1_000 + Timeouts::DEFAULT_HELLO_MS
            }
        );
        assert!(!c.is_active());
        assert!(!c.is_closed());
    }

    #[test]
    fn accepted_hello_activates_and_stamps_activity() {
        let mut c = conn(0);
        assert_eq!(c.on_msg(&hello(), 100, |t| t == "tok"), ConnEvent::Accepted);
        assert_eq!(
            c.state(),
            ConnState::Active {
                last_activity_ms: 100
            }
        );
        assert!(c.is_active());
    }

    #[test]
    fn rejected_hello_closes_the_connection() {
        let mut c = conn(0);
        assert_eq!(c.on_msg(&hello(), 100, |_| false), ConnEvent::Rejected);
        assert_eq!(
            c.state(),
            ConnState::Closed {
                reason: CloseReason::Rejected
            }
        );
        assert!(c.is_closed());
    }

    #[test]
    fn gameplay_before_hello_is_a_protocol_violation() {
        let mut c = conn(0);
        let ev = c.on_msg(&move_input(1), 10, |_| true);
        assert_eq!(ev, ConnEvent::ProtocolViolation);
        assert_eq!(
            c.state(),
            ConnState::Closed {
                reason: CloseReason::ProtocolError
            }
        );
    }

    #[test]
    fn ping_while_active_pongs_and_refreshes_activity() {
        let mut c = conn(0);
        c.on_msg(&hello(), 0, |_| true);
        let ev = c.on_msg(&ClientMsg::Ping { nonce: 42 }, 500, |_| true);
        assert_eq!(ev, ConnEvent::Pong(42));
        assert_eq!(
            c.state(),
            ConnState::Active {
                last_activity_ms: 500
            }
        );
    }

    #[test]
    fn input_while_active_passes_through_and_refreshes_activity() {
        let mut c = conn(0);
        c.on_msg(&hello(), 0, |_| true);
        let ev = c.on_msg(&move_input(9), 700, |_| true);
        match ev {
            ConnEvent::Input { tick, intent } => {
                assert_eq!(tick, Tick(9));
                assert_eq!(
                    intent,
                    Intent::Move {
                        dir: Vec3 {
                            x: 1.0,
                            y: 0.0,
                            z: 0.0
                        }
                    }
                );
            }
            other => panic!("expected Input, got {other:?}"),
        }
        assert_eq!(
            c.state(),
            ConnState::Active {
                last_activity_ms: 700
            }
        );
    }

    #[test]
    fn second_hello_while_active_is_a_violation() {
        let mut c = conn(0);
        c.on_msg(&hello(), 0, |_| true);
        assert_eq!(
            c.on_msg(&hello(), 10, |_| true),
            ConnEvent::ProtocolViolation
        );
        assert!(c.is_closed());
    }

    #[test]
    fn hello_timeout_fires_at_the_deadline() {
        let mut c = conn(0);
        let deadline = Timeouts::DEFAULT_HELLO_MS;
        assert_eq!(c.poll_timeout(deadline - 1), None);
        assert_eq!(c.poll_timeout(deadline), Some(CloseReason::HelloTimeout));
        assert_eq!(
            c.state(),
            ConnState::Closed {
                reason: CloseReason::HelloTimeout
            }
        );
    }

    #[test]
    fn idle_timeout_fires_and_activity_resets_it() {
        let mut c = conn(0);
        c.on_msg(&hello(), 0, |_| true);
        let idle = Timeouts::DEFAULT_IDLE_MS;
        // Not yet idle.
        assert_eq!(c.poll_timeout(idle - 1), None);
        // Traffic refreshes the window.
        c.on_msg(&ClientMsg::Ping { nonce: 1 }, idle - 1, |_| true);
        assert_eq!(c.poll_timeout(idle - 1 + idle - 1), None);
        // Silence past the window closes it.
        assert_eq!(
            c.poll_timeout(idle - 1 + idle),
            Some(CloseReason::IdleTimeout)
        );
    }

    #[test]
    fn messages_on_a_closed_connection_are_ignored() {
        let mut c = conn(0);
        c.on_peer_closed();
        assert_eq!(
            c.state(),
            ConnState::Closed {
                reason: CloseReason::PeerClosed
            }
        );
        assert_eq!(c.on_msg(&hello(), 10, |_| true), ConnEvent::Ignored);
    }

    #[test]
    fn close_is_sticky_first_reason_wins() {
        let mut c = conn(0);
        c.on_msg(&hello(), 0, |_| false); // Rejected
        c.on_peer_closed(); // must not overwrite
        assert_eq!(
            c.state(),
            ConnState::Closed {
                reason: CloseReason::Rejected
            }
        );
    }

    #[test]
    fn poll_timeout_is_a_noop_once_closed() {
        let mut c = conn(0);
        c.on_peer_closed();
        assert_eq!(c.poll_timeout(u64::MAX), None);
    }
}
