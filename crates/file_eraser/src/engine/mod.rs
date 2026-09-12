//! Step 6.1 — Mock File Erasure Engine Abstraction.
//!
//! # Strict Simulation Invariants
//! 1. Non-Destructive: Pure in-memory simulation. Performs NO filesystem writes,
//!    NO deletions, NO truncations, NO metadata alterations, NO stream removals,
//!    and NO slack/free-space clearing.
//! 2. Explicit Simulation Identity: All execution results are explicitly marked
//!    as simulated mock runs (`is_simulated_mock == true`).
//! 3. Safety Gate Enforced: Demands an approved in-memory `SafetyValidationResult`
//!    (`SafetyValidationStatus::Allowed`) prior to starting simulation.
//! 4. Deterministic Progress and Tokens: Identical input requests yield identical
//!    progress stages and reproducible mock verification tokens.

pub mod mock;

pub use mock::MockFileErasureEngine;

use crate::errors::EraserResult;
use crate::method_select::{FileSanitizationMethod, FileSanitizationPlan};
use crate::safety_validate::SafetyValidationResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Cooperative cancellation token for mock file erasure operations.
#[derive(Debug, Clone, Default)]
pub struct MockFileCancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl MockFileCancellationToken {
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

/// Request to simulate the execution of an approved `FileSanitizationPlan`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockFileExecutionRequest {
    /// Target file path to simulate sanitization for.
    pub target_path: PathBuf,
    /// Pre-selected sanitization plan from Step 5.2.
    pub plan: FileSanitizationPlan,
    /// In-memory safety validation result from Step 5.3.
    pub safety_validation: SafetyValidationResult,
    /// Optional seed to ensure deterministic mock verification token generation.
    pub test_seed: Option<String>,
}

/// Deterministic progress event emitted during simulated file erasure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockFileProgressEvent {
    pub target_path: PathBuf,
    pub current_method: Option<FileSanitizationMethod>,
    pub percentage_complete: u8,
    pub status_message: String,
}

/// Final outcome classification for the simulated operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MockFileExecutionOutcome {
    /// All proposed methods in the plan were simulated to completion.
    Completed,
    /// Operation was cancelled cooperatively before completion.
    Cancelled,
    /// Request was refused because Step 5.3 safety validation was Blocked.
    RefusedSafetyBlocked,
    /// Plan was valid and safe, but had zero executable methods to simulate.
    NoExecutableMethods,
}

/// Full audit-ready result of a simulated mock file erasure execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockFileExecutionResult {
    /// Target path of the simulated operation.
    pub target_path: PathBuf,
    /// Methods that were simulated during the execution.
    pub simulated_methods: Vec<FileSanitizationMethod>,
    /// Termination outcome of the simulation.
    pub outcome: MockFileExecutionOutcome,
    /// True ONLY if execution ran to completion without cancellation or refusal.
    pub was_successful: bool,
    /// Explicit flag designating that this run is a simulation and no real file was touched.
    pub is_simulated_mock: bool,
    /// Detailed diagnostic notes and safety disclaimers.
    pub note: String,
    /// Deterministic verification token for future mock verification (Step 6.2).
    pub simulated_verification_token: Option<String>,
}

/// Trait defining the contract for file erasure engines.
pub trait FileErasureEngine {
    fn execute(
        &self,
        request: &MockFileExecutionRequest,
        cancellation: &MockFileCancellationToken,
        progress_callback: &mut dyn FnMut(MockFileProgressEvent),
    ) -> EraserResult<MockFileExecutionResult>;
}