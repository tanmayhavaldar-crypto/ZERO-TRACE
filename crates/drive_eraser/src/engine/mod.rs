//! Step 6.1 — Drive Erasure Engine Abstraction.
//!
//! # Strict Design Invariants
//! 1. Non-Destructive: The mock engine executes in-memory simulation only. It does NOT open
//!    physical drives for writing, issue ATA/NVMe/SCSI commands, write sectors, or modify partitions.
//! 2. Explicit Simulation Marking: All execution results are explicitly marked as simulated mock runs.
//! 3. Safety Enforcement: A request with a blocked safety assessment is refused immediately.
//! 4. Determinism: Identical mock requests produce identical verification tokens and progression steps.

pub mod mock;

pub use mock::MockDriveErasureEngine;

use crate::error::Result;
use crate::method_select::SanitizationMethod;
use crate::safety::SafetyAssessment;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Cooperative cancellation token for mock drive erasure operations.
#[derive(Debug, Clone, Default)]
pub struct MockCancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl MockCancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// Request to execute a simulated sanitization operation on a physical drive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockExecutionRequest {
    /// Identifier of the target physical drive (e.g., "MockDrive_Disk1").
    pub target_device_id: String,
    /// Physical drive index number, if known.
    pub physical_drive_index: Option<u32>,
    /// The pre-selected sanitization method from Step 5.2.
    pub selected_method: SanitizationMethod,
    /// Authoritative pre-validated safety assessment from Step 5.3.
    pub safety_assessment: SafetyAssessment,
    /// Optional seed string for deterministic token calculation in testing.
    pub test_seed: Option<String>,
}

/// Incremental progress event emitted during simulated drive erasure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockProgressEvent {
    pub target_device_id: String,
    pub percentage_complete: u8,
    pub stage_description: String,
}

/// Termination status of the simulated drive erasure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MockExecutionOutcome {
    Completed,
    Cancelled,
    Failed,
}

/// Complete audit-ready outcome of a simulated mock drive erasure operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockExecutionResult {
    /// Target physical drive identifier.
    pub target_device_id: String,
    /// The sanitization method that was simulated.
    pub method: SanitizationMethod,
    /// Final execution status.
    pub outcome: MockExecutionOutcome,
    /// True only if the simulation ran to completion without cancellation or error.
    pub was_successful: bool,
    /// Flag explicitly marking that this is a simulated operation and NOT a physical drive sanitization.
    pub is_simulated_mock: bool,
    /// Audit note explaining the mock execution status.
    pub note: String,
    /// Deterministic pseudo-random verification token representing simulated state.
    pub simulated_verification_token: Option<String>,
}

/// Abstract contract for drive erasure engines.
pub trait DriveErasureEngine {
    fn execute(
        &self,
        request: &MockExecutionRequest,
        cancellation: &MockCancellationToken,
        progress_callback: &mut dyn FnMut(MockProgressEvent),
    ) -> Result<MockExecutionResult>;
}