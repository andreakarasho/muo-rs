//! Network traffic statistics.

/// One send buffer growth tier's pool usage, in buffers of `buffer_size` bytes.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct SendBufferTierUsage {
    pub buffer_size: i32,
    pub capacity: i32,
    pub in_use: i32,
    pub retain_floor: i32,
}

/// What the last once-a-minute send buffer maintenance sweep saw.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct SendBufferSweep {
    pub ran: bool,
    pub tick: i64,
    pub tier_capacity_bytes: i64,
    pub tier_in_use: i32,
    pub tier_retain_floor: i32,
    pub tier_buffers_released: i32,
    pub base_capacity_bytes: i64,
    pub base_buffers_released: i32,
    pub budget_refusals: i32,
    pub cap_refusals: i32,
    pub ceiling_refusals: i32,
}

/// Read-only snapshot of the network layer for diagnostics (`NetworkStats`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NetworkStats {
    /// Sockets the transport currently holds, authenticated or not.
    pub connected: i32,
    pub max_connections: i32,
    /// Connections past login.
    pub authenticated: i32,
    pub recv_buffer_size: i32,
    pub send_buffer_size: i32,
    pub max_send_buffer_size: i32,
    /// Pre-auth buffer sizes; 0 when pre-auth buffers are off (always, in this transport).
    pub initial_recv_buffer_size: i32,
    pub initial_send_buffer_size: i32,
    pub send_buffer_growth_budget: i64,
    pub memory_ceiling_percent: i32,
    pub available_memory_bytes: i64,
    pub tiers: Vec<SendBufferTierUsage>,
    pub last_sweep: SendBufferSweep,
    /// Connections whose parser is paused on a packet throttle.
    pub throttled: i32,
    /// Connections with unsent bytes queued for the next flush.
    pub flush_pending: i32,
    /// Connections closing after their final flush.
    pub pending_disconnects: i32,
}
