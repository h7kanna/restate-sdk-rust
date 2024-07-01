// Copyright (c) 2023 -  Restate Software, Inc., Restate GmbH.
// All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

//! This module contains all the core types representing a service invocation.

use std::fmt;
use bytestring::ByteString;
use prost::bytes::Bytes;
use crate::errors::InvocationError;
use crate::identifiers::ServiceId;

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum ServiceType {
    Service,
    VirtualObject,
    Workflow,
}

impl ServiceType {
    pub fn is_keyed(&self) -> bool {
        matches!(self, ServiceType::VirtualObject | ServiceType::Workflow)
    }

    pub fn has_state(&self) -> bool {
        self.is_keyed()
    }
}

impl fmt::Display for ServiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(
    Eq, Hash, PartialEq, Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum VirtualObjectHandlerType {
    #[default]
    Exclusive,
    Shared,
}

impl fmt::Display for VirtualObjectHandlerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(
    Eq, Hash, PartialEq, Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum WorkflowHandlerType {
    #[default]
    Workflow,
    Shared,
}

impl fmt::Display for WorkflowHandlerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum InvocationTargetType {
    Service,
    VirtualObject(VirtualObjectHandlerType),
    Workflow(WorkflowHandlerType),
}

impl InvocationTargetType {
    pub fn is_keyed(&self) -> bool {
        matches!(
            self,
            InvocationTargetType::VirtualObject(_) | InvocationTargetType::Workflow(_)
        )
    }

    pub fn can_read_state(&self) -> bool {
        self.is_keyed()
    }

    pub fn can_write_state(&self) -> bool {
        matches!(
            self,
            InvocationTargetType::VirtualObject(VirtualObjectHandlerType::Exclusive)
                | InvocationTargetType::Workflow(WorkflowHandlerType::Workflow)
        )
    }
}

impl fmt::Display for InvocationTargetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<InvocationTargetType> for ServiceType {
    fn from(value: InvocationTargetType) -> Self {
        match value {
            InvocationTargetType::Service => ServiceType::Service,
            InvocationTargetType::VirtualObject(_) => ServiceType::VirtualObject,
            InvocationTargetType::Workflow(_) => ServiceType::Workflow,
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum InvocationTarget {
    Service {
        name: ByteString,
        handler: ByteString,
    },
    VirtualObject {
        name: ByteString,
        key: ByteString,
        handler: ByteString,
        handler_ty: VirtualObjectHandlerType,
    },
    Workflow {
        name: ByteString,
        key: ByteString,
        handler: ByteString,
        handler_ty: WorkflowHandlerType,
    },
}

impl InvocationTarget {
    pub fn service(name: impl Into<ByteString>, handler: impl Into<ByteString>) -> Self {
        Self::Service {
            name: name.into(),
            handler: handler.into(),
        }
    }

    pub fn virtual_object(
        name: impl Into<ByteString>,
        key: impl Into<ByteString>,
        handler: impl Into<ByteString>,
        handler_ty: VirtualObjectHandlerType,
    ) -> Self {
        Self::VirtualObject {
            name: name.into(),
            key: key.into(),
            handler: handler.into(),
            handler_ty,
        }
    }

    pub fn workflow(
        name: impl Into<ByteString>,
        key: impl Into<ByteString>,
        handler: impl Into<ByteString>,
        handler_ty: WorkflowHandlerType,
    ) -> Self {
        Self::Workflow {
            name: name.into(),
            key: key.into(),
            handler: handler.into(),
            handler_ty,
        }
    }

    pub fn service_name(&self) -> &ByteString {
        match self {
            InvocationTarget::Service { name, .. } => name,
            InvocationTarget::VirtualObject { name, .. } => name,
            InvocationTarget::Workflow { name, .. } => name,
        }
    }

    pub fn key(&self) -> Option<&ByteString> {
        match self {
            InvocationTarget::Service { .. } => None,
            InvocationTarget::VirtualObject { key, .. } => Some(key),
            InvocationTarget::Workflow { key, .. } => Some(key),
        }
    }

    pub fn handler_name(&self) -> &ByteString {
        match self {
            InvocationTarget::Service { handler, .. } => handler,
            InvocationTarget::VirtualObject { handler, .. } => handler,
            InvocationTarget::Workflow { handler, .. } => handler,
        }
    }

    pub fn as_keyed_service_id(&self) -> Option<ServiceId> {
        match self {
            InvocationTarget::Service { .. } => None,
            InvocationTarget::VirtualObject { name, key, .. } => {
                Some(ServiceId::new(name.clone(), key.clone()))
            }
            InvocationTarget::Workflow { name, key, .. } => {
                Some(ServiceId::new(name.clone(), key.clone()))
            }
        }
    }

    pub fn service_ty(&self) -> ServiceType {
        match self {
            InvocationTarget::Service { .. } => ServiceType::Service,
            InvocationTarget::VirtualObject { .. } => ServiceType::VirtualObject,
            InvocationTarget::Workflow { .. } => ServiceType::Workflow,
        }
    }

    pub fn invocation_target_ty(&self) -> InvocationTargetType {
        match self {
            InvocationTarget::Service { .. } => InvocationTargetType::Service,
            InvocationTarget::VirtualObject { handler_ty, .. } => {
                InvocationTargetType::VirtualObject(*handler_ty)
            }
            InvocationTarget::Workflow { handler_ty, .. } => {
                InvocationTargetType::Workflow(*handler_ty)
            }
        }
    }
}

impl fmt::Display for InvocationTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/", self.service_name())?;
        if let Some(key) = self.key() {
            write!(f, "{}/", key)?;
        }
        write!(f, "{}", self.handler_name())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Header {
    pub name: ByteString,
    pub value: ByteString,
}

impl Header {
    pub fn new(name: impl Into<ByteString>, value: impl Into<ByteString>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResponseResult {
    Success(Bytes),
    Failure(InvocationError),
}

impl From<Result<Bytes, InvocationError>> for ResponseResult {
    fn from(value: Result<Bytes, InvocationError>) -> Self {
        match value {
            Ok(v) => ResponseResult::Success(v),
            Err(e) => ResponseResult::Failure(e),
        }
    }
}

impl From<ResponseResult> for Result<Bytes, InvocationError> {
    fn from(value: ResponseResult) -> Self {
        match value {
            ResponseResult::Success(bytes) => Ok(bytes),
            ResponseResult::Failure(e) => Err(e),
        }
    }
}

impl From<InvocationError> for ResponseResult {
    fn from(e: InvocationError) -> Self {
        ResponseResult::Failure(e)
    }
}

impl From<&InvocationError> for ResponseResult {
    fn from(e: &InvocationError) -> Self {
        ResponseResult::Failure(e.clone())
    }
}