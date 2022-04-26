// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use aptos_logger::{error, info};
use aptos_types::transaction::Version;
use schemadb::SchemaBatch;
use std::{thread::sleep, time::Duration};

/// Defines the trait for pruner for different DB
pub trait DBPruner {
    /// Find out the first undeleted item in the stale node index.
    ///
    /// Seeking from the beginning (version 0) is potentially costly, we do it once upon worker
    /// thread start, record the progress and seek from that position afterwards.
    fn initialize(&self) {
        loop {
            match self.initialize_least_readable_version() {
                Ok(least_readable_version) => {
                    info!(
                        least_readable_version = least_readable_version,
                        "{} initialized.",
                        self.name()
                    );
                    self.record_progress(least_readable_version);
                    return;
                }
                Err(e) => {
                    error!(
                        error = ?e,
                        "{} Error on first seek. Retrying in 1 second.", self.name()
                    );
                    sleep(Duration::from_secs(1));
                }
            }
        }
    }
    fn name(&self) -> &'static str;

    /// Performs the actual pruning, a target version is passed, which is the target the pruner
    /// tries to prune.
    fn prune(&self, db_batch: &mut SchemaBatch) -> anyhow::Result<Version>;

    /// Initializes the least readable version stored in underlying DB storage
    fn initialize_least_readable_version(&self) -> anyhow::Result<Version>;

    /// Returns the least readable version stores in the DB pruner
    fn least_readable_version(&self) -> Version;

    /// Sets the target version for the pruner
    fn set_target_version(&self, target_version: Version);

    /// Returns the target version for the DB pruner
    fn target_version(&self) -> Version;

    /// Records the current progress of the pruner by updating the least readable version
    fn record_progress(&self, least_readable_version: Version);

    /// True if there is pruning work pending to be done
    fn is_pruning_pending(&self) -> bool {
        self.target_version() > self.least_readable_version()
    }
}
