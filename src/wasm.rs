// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! WebAssembly bindings for Schubert access control.
//!
//! Exposes a JavaScript API for creating access controllers, registering
//! capabilities, creating principals, granting capabilities, and checking
//! access — all running in the browser via WebAssembly.
//!
//! # Usage (JavaScript)
//!
//! ```js
//! import init, { WasmController } from './schubert.js';
//!
//! await init();
//!
//! // Create a controller for Gr(2,4)
//! const acl = WasmController.new(2, 4);
//!
//! // Register capabilities
//! acl.register_capability("read:data", "Read data", [1], "ReadLike");
//! acl.register_capability("write:data", "Write data", [2], "WriteLike");
//!
//! // Create a principal and grant capabilities
//! const alice = acl.create_principal("alice");
//! acl.grant(alice, "read:data");
//! acl.grant(alice, "write:data");
//!
//! // Check access
//! const result = acl.check(alice, ["read:data", "write:data"]);
//! console.log(result.tag, result.configurations);
//! ```

use crate::AccessController;
use wasm_bindgen::prelude::*;

/// WebAssembly wrapper around [`AccessController`].
///
/// Provides a JavaScript-friendly API for Schubert access control.
#[wasm_bindgen]
pub struct WasmController {
    inner: AccessController,
}

#[wasm_bindgen]
impl WasmController {
    /// Create a new access controller for Gr(k,n).
    ///
    /// # Panics
    ///
    /// Panics if k,n are invalid (k ≥ 1, n ≥ 2, k < n required).
    #[wasm_bindgen(constructor)]
    pub fn new(k: usize, n: usize) -> WasmController {
        let inner = AccessController::new(k, n).expect("invalid Grassmannian parameters");
        WasmController { inner }
    }

    /// Get the Grassmannian parameters as [k, n].
    #[wasm_bindgen(getter)]
    pub fn grassmannian(&self) -> Vec<usize> {
        let (k, n) = self.inner.grassmannian();
        vec![k, n]
    }

    /// Register a capability.
    ///
    /// Returns `true` if registration succeeded, `false` if the capability
    /// already exists or the partition is invalid.
    pub fn register_capability(
        &mut self,
        id: &str,
        label: &str,
        partition: Vec<usize>,
        kind: &str,
    ) -> bool {
        let kind = match kind {
            "ReadLike" => crate::CapabilityKind::ReadLike,
            "WriteLike" => crate::CapabilityKind::WriteLike,
            "AdminLike" => crate::CapabilityKind::AdminLike,
            _ => crate::CapabilityKind::Custom,
        };
        let cap = crate::Capability::new(id, label, partition, kind);
        self.inner.register_capability(cap).is_ok()
    }

    /// Create a principal and return its ID.
    pub fn create_principal(&mut self, id: &str) -> String {
        match self.inner.create_principal(id) {
            Ok(pid) => pid.to_string(),
            Err(_) => String::new(),
        }
    }

    /// Grant a capability to a principal.
    ///
    /// Returns `true` if the grant succeeded.
    pub fn grant(&mut self, principal_id: &str, capability_id: &str) -> bool {
        let pid = crate::PrincipalId::new(principal_id);
        self.inner.grant(&pid, capability_id).is_ok()
    }

    /// Revoke a capability from a principal.
    ///
    /// Returns `true` if the revocation succeeded.
    pub fn revoke(&mut self, principal_id: &str, capability_id: &str) -> bool {
        let pid = crate::PrincipalId::new(principal_id);
        self.inner.revoke(&pid, capability_id).is_ok()
    }

    /// Check whether a principal satisfies a set of capability requirements.
    ///
    /// Returns a JavaScript object with:
    /// - `tag`: "granted", "impossible", "denied", or "underconstrained"
    /// - `configurations`: number of valid configurations (if granted)
    /// - `dimension`: solution variety dimension (if underconstrained)
    /// - `path`: computation path used
    /// - `conflicting`: conflicting capability IDs (if impossible)
    pub fn check(&self, principal_id: &str, required: Vec<String>) -> JsValue {
        let pid = crate::PrincipalId::new(principal_id);
        let req_refs: Vec<&str> = required.iter().map(|s| s.as_str()).collect();

        match self.inner.check(&pid, &req_refs) {
            Ok(decision) => {
                let result = js_sys::Object::new();
                match decision {
                    crate::AccessDecision::Granted {
                        configurations,
                        path,
                    } => {
                        js_sys::Reflect::set(&result, &"tag".into(), &"granted".into()).unwrap();
                        js_sys::Reflect::set(
                            &result,
                            &"configurations".into(),
                            &(configurations as f64).into(),
                        )
                        .unwrap();
                        js_sys::Reflect::set(&result, &"path".into(), &format!("{path:?}").into())
                            .unwrap();
                    }
                    crate::AccessDecision::Impossible { conflicting } => {
                        js_sys::Reflect::set(&result, &"tag".into(), &"impossible".into()).unwrap();
                        let ids: Vec<JsValue> =
                            conflicting.iter().map(|c| c.as_str().into()).collect();
                        let arr = js_sys::Array::from_iter(ids);
                        js_sys::Reflect::set(&result, &"conflicting".into(), &arr).unwrap();
                    }
                    crate::AccessDecision::Denied => {
                        js_sys::Reflect::set(&result, &"tag".into(), &"denied".into()).unwrap();
                    }
                    crate::AccessDecision::Underconstrained { dimension } => {
                        js_sys::Reflect::set(&result, &"tag".into(), &"underconstrained".into())
                            .unwrap();
                        js_sys::Reflect::set(
                            &result,
                            &"dimension".into(),
                            &(dimension as f64).into(),
                        )
                        .unwrap();
                    }
                }
                result.into()
            }
            Err(e) => {
                let err = js_sys::Error::new(&format!("{e}"));
                JsValue::from(err)
            }
        }
    }

    /// Check whether a principal holds a specific capability.
    pub fn holds(&self, principal_id: &str, capability_id: &str) -> bool {
        let pid = crate::PrincipalId::new(principal_id);
        self.inner
            .principal(&pid)
            .is_some_and(|p| p.holds(capability_id))
    }

    /// Get the capability count for a principal.
    pub fn capability_count(&self, principal_id: &str) -> usize {
        let pid = crate::PrincipalId::new(principal_id);
        self.inner
            .principal(&pid)
            .map(|p| p.capability_count())
            .unwrap_or(0)
    }

    /// Get the effective access dimension for a principal.
    pub fn effective_dimension(&self, principal_id: &str) -> i64 {
        let pid = crate::PrincipalId::new(principal_id);
        self.inner.effective_dimension(&pid).unwrap_or(0) as i64
    }
}
