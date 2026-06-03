//! ML refinement modules (Phase 2)
//!
//! Reserved ONNX model inference hooks for edge case correction and confidence refinement.
//!
//! Note: This module is feature-gated behind `--features ml` and remains a dependency-free
//! Phase 2 integration point. Phase 1 ships as a pure DSP pipeline.

#[cfg(feature = "ml")]
pub mod onnx_model;

#[cfg(feature = "ml")]
pub mod refinement;

#[cfg(feature = "ml")]
pub mod edge_cases;
