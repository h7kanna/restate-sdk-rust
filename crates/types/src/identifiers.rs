// Copyright (c) 2023 -  Restate Software, Inc., Restate GmbH.
// All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

//! Restate uses many identifiers to uniquely identify its services and entities.

use bytestring::ByteString;
use std::fmt;

pub type EntryIndex = u32;

/// Identifying to which partition a key belongs. This is unlike the [`PartitionId`]
/// which identifies a consecutive range of partition keys.
pub type PartitionKey = u64;

/// Trait for data structures that have a partition key
pub trait WithPartitionKey {
    /// Returns the partition key
    fn partition_key(&self) -> PartitionKey;
}

/// Id of a keyed service instance.
///
/// Services are isolated by key. This means that there cannot be two concurrent
/// invocations for the same service instance (service name, key).
#[derive(Eq, Hash, PartialEq, PartialOrd, Ord, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ServiceId {
    // TODO rename this to KeyedServiceId. This type can be used only by keyed service types (virtual objects and workflows)
    /// Identifies the grpc service
    pub service_name: ByteString,
    /// Identifies the service instance for the given service name
    pub key: ByteString,

    partition_key: PartitionKey,
}

impl ServiceId {
    pub fn new(service_name: impl Into<ByteString>, key: impl Into<ByteString>) -> Self {
        let key = key.into();
        //let partition_key = partitioner::HashPartitioner::compute_partition_key(&key);
        Self::with_partition_key(0, service_name, key)
    }

    /// # Important
    /// The `partition_key` must be hash of the `key` computed via [`HashPartitioner`].
    pub fn with_partition_key(
        partition_key: PartitionKey,
        service_name: impl Into<ByteString>,
        key: impl Into<ByteString>,
    ) -> Self {
        Self::from_parts(partition_key, service_name.into(), key.into())
    }

    /// # Important
    /// The `partition_key` must be hash of the `key` computed via [`HashPartitioner`].
    pub const fn from_parts(partition_key: PartitionKey, service_name: ByteString, key: ByteString) -> Self {
        Self {
            service_name,
            key,
            partition_key,
        }
    }
}

impl WithPartitionKey for ServiceId {
    fn partition_key(&self) -> PartitionKey {
        self.partition_key
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.service_name, self.key)
    }
}
