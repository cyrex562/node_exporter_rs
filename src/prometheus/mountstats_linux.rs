
const FLOAT64_MANTISSA: u64 = 9_007_199_254_740_992;

use prometheus::core::Desc;
use slog::Logger;
use std::sync::Arc;

pub struct MountStatsCollector {
    // General statistics
    nfs_age_seconds_total: Arc<Desc>,

    // Byte statistics
    nfs_read_bytes_total: Arc<Desc>,
    nfs_write_bytes_total: Arc<Desc>,
    nfs_direct_read_bytes_total: Arc<Desc>,
    nfs_direct_write_bytes_total: Arc<Desc>,
    nfs_total_read_bytes_total: Arc<Desc>,
    nfs_total_write_bytes_total: Arc<Desc>,
    nfs_read_pages_total: Arc<Desc>,
    nfs_write_pages_total: Arc<Desc>,

    // Per-operation statistics
    nfs_operations_requests_total: Arc<Desc>,
    nfs_operations_transmissions_total: Arc<Desc>,
    nfs_operations_major_timeouts_total: Arc<Desc>,
    nfs_operations_sent_bytes_total: Arc<Desc>,
    nfs_operations_received_bytes_total: Arc<Desc>,
    nfs_operations_queue_time_seconds_total: Arc<Desc>,
    nfs_operations_response_time_seconds_total: Arc<Desc>,
    nfs_operations_request_time_seconds_total: Arc<Desc>,

    // Transport statistics
    nfs_transport_bind_total: Arc<Desc>,
    nfs_transport_connect_total: Arc<Desc>,
    nfs_transport_idle_time_seconds: Arc<Desc>,
    nfs_transport_sends_total: Arc<Desc>,
    nfs_transport_receives_total: Arc<Desc>,
    nfs_transport_bad_transaction_ids_total: Arc<Desc>,
    nfs_transport_backlog_queue_total: Arc<Desc>,
    nfs_transport_maximum_rpc_slots: Arc<Desc>,
    nfs_transport_sending_queue_total: Arc<Desc>,
    nfs_transport_pending_queue_total: Arc<Desc>,

    // Event statistics
    nfs_event_inode_revalidate_total: Arc<Desc>,
    nfs_event_dnode_revalidate_total: Arc<Desc>,
    nfs_event_data_invalidate_total: Arc<Desc>,
    nfs_event_attribute_invalidate_total: Arc<Desc>,
    nfs_event_vfs_open_total: Arc<Desc>,
    nfs_event_vfs_lookup_total: Arc<Desc>,
    nfs_event_vfs_access_total: Arc<Desc>,
    nfs_event_vfs_update_page_total: Arc<Desc>,
    nfs_event_vfs_read_page_total: Arc<Desc>,
    nfs_event_vfs_read_pages_total: Arc<Desc>,
    nfs_event_vfs_write_page_total: Arc<Desc>,
    nfs_event_vfs_write_pages_total: Arc<Desc>,
    nfs_event_vfs_getdents_total: Arc<Desc>,
    nfs_event_vfs_setattr_total: Arc<Desc>,
    nfs_event_vfs_flush_total: Arc<Desc>,
    nfs_event_vfs_fsync_total: Arc<Desc>,
    nfs_event_vfs_lock_total: Arc<Desc>,
    nfs_event_vfs_file_release_total: Arc<Desc>,
    nfs_event_truncation_total: Arc<Desc>,
    nfs_event_write_extension_total: Arc<Desc>,
    nfs_event_silly_rename_total: Arc<Desc>,
    nfs_event_short_read_total: Arc<Desc>,
    nfs_event_short_write_total: Arc<Desc>,
    nfs_event_jukebox_delay_total: Arc<Desc>,
    nfs_event_pnfs_read_total: Arc<Desc>,
    nfs_event_pnfs_write_total: Arc<Desc>,

    proc: procfs::Proc,

    logger: Logger,
}

pub struct NfsDeviceIdentifier {
    device: String,
    protocol: String,
    mount_address: String,
}

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref MOUNTSTATS_COLLECTOR: MountStatsCollector = {
        register_collector("mountstats", default_disabled(), NewMountStatsCollector::new());
        MountStatsCollector::new()
    };
}

use prometheus::{self, core::Collector, core::Desc, core::Opts, Error};
use slog::Logger;
use std::sync::Arc;

pub struct MountStatsCollector {
    nfs_age_seconds_total: Desc,
    nfs_read_bytes_total: Desc,
    nfs_write_bytes_total: Desc,
    nfs_direct_read_bytes_total: Desc,
    nfs_direct_write_bytes_total: Desc,
    nfs_total_read_bytes_total: Desc,
    nfs_total_write_bytes_total: Desc,
    nfs_read_pages_total: Desc,
    nfs_write_pages_total: Desc,
    nfs_transport_bind_total: Desc,
    nfs_transport_connect_total: Desc,
    nfs_transport_idle_time_seconds: Desc,
    nfs_transport_sends_total: Desc,
    nfs_transport_receives_total: Desc,
    nfs_transport_bad_transaction_ids_total: Desc,
    nfs_transport_backlog_queue_total: Desc,
    nfs_transport_maximum_rpc_slots: Desc,
    nfs_transport_sending_queue_total: Desc,
    nfs_transport_pending_queue_total: Desc,
    nfs_operations_requests_total: Desc,
    nfs_operations_transmissions_total: Desc,
    nfs_operations_major_timeouts_total: Desc,
    nfs_operations_sent_bytes_total: Desc,
    nfs_operations_received_bytes_total: Desc,
    nfs_operations_queue_time_seconds_total: Desc,
    nfs_operations_response_time_seconds_total: Desc,
    nfs_operations_request_time_seconds_total: Desc,
    nfs_event_inode_revalidate_total: Desc,
    nfs_event_dnode_revalidate_total: Desc,
    nfs_event_data_invalidate_total: Desc,
    nfs_event_attribute_invalidate_total: Desc,
    nfs_event_vfs_open_total: Desc,
    nfs_event_vfs_lookup_total: Desc,
    nfs_event_vfs_access_total: Desc,
    nfs_event_vfs_update_page_total: Desc,
    nfs_event_vfs_read_page_total: Desc,
    nfs_event_vfs_read_pages_total: Desc,
    nfs_event_vfs_write_page_total: Desc,
    nfs_event_vfs_write_pages_total: Desc,
    nfs_event_vfs_getdents_total: Desc,
    nfs_event_vfs_setattr_total: Desc,
    nfs_event_vfs_flush_total: Desc,
    nfs_event_vfs_fsync_total: Desc,
    nfs_event_vfs_lock_total: Desc,
    nfs_event_vfs_file_release_total: Desc,
    nfs_event_truncation_total: Desc,
    nfs_event_write_extension_total: Desc,
    nfs_event_silly_rename_total: Desc,
    nfs_event_short_read_total: Desc,
    nfs_event_short_write_total: Desc,
    nfs_event_jukebox_delay_total: Desc,
    nfs_event_pnfs_read_total: Desc,
    nfs_event_pnfs_write_total: Desc,
    proc: procfs::Proc,
    logger: Logger,
}

impl MountStatsCollector {
    pub fn new(logger: Logger) -> Result<Self, Error> {
        let fs = procfs::Proc::new("/proc")?;
        let proc = fs.self_proc()?;

        let labels = vec!["export", "protocol", "mountaddr"];
        let op_labels = vec!["export", "protocol", "mountaddr", "operation"];
        let namespace = "mountstats_nfs";

        Ok(MountStatsCollector {
            nfs_age_seconds_total: Desc::new(
                Opts::new("age_seconds_total", "The age of the NFS mount in seconds.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_read_bytes_total: Desc::new(
                Opts::new("read_bytes_total", "Number of bytes read using the read() syscall.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_write_bytes_total: Desc::new(
                Opts::new("write_bytes_total", "Number of bytes written using the write() syscall.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_direct_read_bytes_total: Desc::new(
                Opts::new("direct_read_bytes_total", "Number of bytes read using the read() syscall in O_DIRECT mode.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_direct_write_bytes_total: Desc::new(
                Opts::new("direct_write_bytes_total", "Number of bytes written using the write() syscall in O_DIRECT mode.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_total_read_bytes_total: Desc::new(
                Opts::new("total_read_bytes_total", "Number of bytes read from the NFS server, in total.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_total_write_bytes_total: Desc::new(
                Opts::new("total_write_bytes_total", "Number of bytes written to the NFS server, in total.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_read_pages_total: Desc::new(
                Opts::new("read_pages_total", "Number of pages read directly via mmap()'d files.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_write_pages_total: Desc::new(
                Opts::new("write_pages_total", "Number of pages written directly via mmap()'d files.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_transport_bind_total: Desc::new(
                Opts::new("transport_bind_total", "Number of times the client has had to establish a connection from scratch to the NFS server.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_transport_connect_total: Desc::new(
                Opts::new("transport_connect_total", "Number of times the client has made a TCP connection to the NFS server.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_transport_idle_time_seconds: Desc::new(
                Opts::new("transport_idle_time_seconds", "Duration since the NFS mount last saw any RPC traffic, in seconds.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_transport_sends_total: Desc::new(
                Opts::new("transport_sends_total", "Number of RPC requests for this mount sent to the NFS server.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_transport_receives_total: Desc::new(
                Opts::new("transport_receives_total", "Number of RPC responses for this mount received from the NFS server.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_transport_bad_transaction_ids_total: Desc::new(
                Opts::new("transport_bad_transaction_ids_total", "Number of times the NFS server sent a response with a transaction ID unknown to this client.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_transport_backlog_queue_total: Desc::new(
                Opts::new("transport_backlog_queue_total", "Total number of items added to the RPC backlog queue.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_transport_maximum_rpc_slots: Desc::new(
                Opts::new("transport_maximum_rpc_slots", "Maximum number of simultaneously active RPC requests ever used.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_transport_sending_queue_total: Desc::new(
                Opts::new("transport_sending_queue_total", "Total number of items added to the RPC transmission sending queue.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_transport_pending_queue_total: Desc::new(
                Opts::new("transport_pending_queue_total", "Total number of items added to the RPC transmission pending queue.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_operations_requests_total: Desc::new(
                Opts::new("operations_requests_total", "Number of requests performed for a given operation.")
                    .namespace(namespace)
                    .variable_labels(op_labels.clone()),
                vec![],
            )?,
            nfs_operations_transmissions_total: Desc::new(
                Opts::new("operations_transmissions_total", "Number of times an actual RPC request has been transmitted for a given operation.")
                    .namespace(namespace)
                    .variable_labels(op_labels.clone()),
                vec![],
            )?,
            nfs_operations_major_timeouts_total: Desc::new(
                Opts::new("operations_major_timeouts_total", "Number of times a request has had a major timeout for a given operation.")
                    .namespace(namespace)
                    .variable_labels(op_labels.clone()),
                vec![],
            )?,
            nfs_operations_sent_bytes_total: Desc::new(
                Opts::new("operations_sent_bytes_total", "Number of bytes sent for a given operation, including RPC headers and payload.")
                    .namespace(namespace)
                    .variable_labels(op_labels.clone()),
                vec![],
            )?,
            nfs_operations_received_bytes_total: Desc::new(
                Opts::new("operations_received_bytes_total", "Number of bytes received for a given operation, including RPC headers and payload.")
                    .namespace(namespace)
                    .variable_labels(op_labels.clone()),
                vec![],
            )?,
            nfs_operations_queue_time_seconds_total: Desc::new(
                Opts::new("operations_queue_time_seconds_total", "Duration all requests spent queued for transmission for a given operation before they were sent, in seconds.")
                    .namespace(namespace)
                    .variable_labels(op_labels.clone()),
                vec![],
            )?,
            nfs_operations_response_time_seconds_total: Desc::new(
                Opts::new("operations_response_time_seconds_total", "Duration all requests took to get a reply back after a request for a given operation was transmitted, in seconds.")
                    .namespace(namespace)
                    .variable_labels(op_labels.clone()),
                vec![],
            )?,
            nfs_operations_request_time_seconds_total: Desc::new(
                Opts::new("operations_request_time_seconds_total", "Duration all requests took from when a request was enqueued to when it was completely handled for a given operation, in seconds.")
                    .namespace(namespace)
                    .variable_labels(op_labels.clone()),
                vec![],
            )?,
            nfs_event_inode_revalidate_total: Desc::new(
                Opts::new("event_inode_revalidate_total", "Number of times cached inode attributes are re-validated from the server.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_dnode_revalidate_total: Desc::new(
                Opts::new("event_dnode_revalidate_total", "Number of times cached dentry nodes are re-validated from the server.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_data_invalidate_total: Desc::new(
                Opts::new("event_data_invalidate_total", "Number of times an inode cache is cleared.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_attribute_invalidate_total: Desc::new(
                Opts::new("event_attribute_invalidate_total", "Number of times cached inode attributes are invalidated.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_open_total: Desc::new(
                Opts::new("event_vfs_open_total", "Number of times cached inode attributes are invalidated.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_lookup_total: Desc::new(
                Opts::new("event_vfs_lookup_total", "Number of times a directory lookup has occurred.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_access_total: Desc::new(
                Opts::new("event_vfs_access_total", "Number of times permissions have been checked.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_update_page_total: Desc::new(
                Opts::new("event_vfs_update_page_total", "Number of updates (and potential writes) to pages.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_read_page_total: Desc::new(
                Opts::new("event_vfs_read_page_total", "Number of pages read directly via mmap()'d files.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_read_pages_total: Desc::new(
                Opts::new("event_vfs_read_pages_total", "Number of times a group of pages have been read.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_write_page_total: Desc::new(
                Opts::new("event_vfs_write_page_total", "Number of pages written directly via mmap()'d files.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_write_pages_total: Desc::new(
                Opts::new("event_vfs_write_pages_total", "Number of times a group of pages have been written.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_getdents_total: Desc::new(
                Opts::new("event_vfs_getdents_total", "Number of times directory entries have been read with getdents().")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_setattr_total: Desc::new(
                Opts::new("event_vfs_setattr_total", "Number of times directory entries have been read with getdents().")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_flush_total: Desc::new(
                Opts::new("event_vfs_flush_total", "Number of pending writes that have been forcefully flushed to the server.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_fsync_total: Desc::new(
                Opts::new("event_vfs_fsync_total", "Number of times fsync() has been called on directories and files.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_lock_total: Desc::new(
                Opts::new("event_vfs_lock_total", "Number of times locking has been attempted on a file.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_vfs_file_release_total: Desc::new(
                Opts::new("event_vfs_file_release_total", "Number of times files have been closed and released.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_truncation_total: Desc::new(
                Opts::new("event_truncation_total", "Number of times files have been truncated.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_write_extension_total: Desc::new(
                Opts::new("event_write_extension_total", "Number of times a file has been grown due to writes beyond its existing end.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_silly_rename_total: Desc::new(
                Opts::new("event_silly_rename_total", "Number of times a file was removed while still open by another process.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_short_read_total: Desc::new(
                Opts::new("event_short_read_total", "Number of times the NFS server gave less data than expected while reading.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_short_write_total: Desc::new(
                Opts::new("event_short_write_total", "Number of times the NFS server wrote less data than expected while writing.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_jukebox_delay_total: Desc::new(
                Opts::new("event_jukebox_delay_total", "Number of times the NFS server indicated EJUKEBOX; retrieving data from offline storage.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_pnfs_read_total: Desc::new(
                Opts::new("event_pnfs_read_total", "Number of NFS v4.1+ pNFS reads.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            nfs_event_pnfs_write_total: Desc::new(
                Opts::new("event_pnfs_write_total", "Number of NFS v4.1+ pNFS writes.")
                    .namespace(namespace)
                    .variable_labels(labels.clone()),
                vec![],
            )?,
            proc,
            logger,
        })
    }

    pub fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Error> {
        let mounts = self.proc.mount_stats().map_err(|e| format!("failed to parse mountstats: {}", e))?;
        let mounts_info = self.proc.mount_info().map_err(|e| format!("failed to parse mountinfo: {}", e))?;

        let mut device_list: HashMap<NfsDeviceIdentifier, bool> = HashMap::new();

        for (idx, m) in mounts.iter().enumerate() {
            if let Some(stats) = m.stats.as_ref().and_then(|s| s.as_nfs()) {
                let mount_address = if idx < mounts_info.len() {
                    mounts_info[idx].super_options.get("addr").cloned().unwrap_or_default()
                } else {
                    String::new()
                };

                for transport in &stats.transport {
                    let device_identifier = NfsDeviceIdentifier {
                        device: m.device.clone(),
                        protocol: transport.protocol.clone(),
                        mount_address: mount_address.clone(),
                    };

                    if device_list.contains_key(&device_identifier) {
                        self.logger.debug("Skipping duplicate device entry", o!("device" => format!("{:?}", device_identifier)));
                        break;
                    }

                    device_list.insert(device_identifier.clone(), true);
                    self.update_nfs_stats(ch, stats, &m.device, &transport.protocol, &mount_address);
                }
            }
        }

        Ok(())
    }

    pub fn update_nfs_stats(&self, ch: &mut dyn FnMut(prometheus::Metric), s: &MountStatsNFS, export: &str, protocol: &str, mount_address: &str) {
        let label_values = &[export, protocol, mount_address];

        ch(prometheus::Metric::new(
            &self.nfs_age_seconds_total,
            prometheus::ValueType::Counter,
            s.age.seconds(),
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_read_bytes_total,
            prometheus::ValueType::Counter,
            s.bytes.read as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_write_bytes_total,
            prometheus::ValueType::Counter,
            s.bytes.write as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_direct_read_bytes_total,
            prometheus::ValueType::Counter,
            s.bytes.direct_read as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_direct_write_bytes_total,
            prometheus::ValueType::Counter,
            s.bytes.direct_write as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_total_read_bytes_total,
            prometheus::ValueType::Counter,
            s.bytes.read_total as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_total_write_bytes_total,
            prometheus::ValueType::Counter,
            s.bytes.write_total as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_read_pages_total,
            prometheus::ValueType::Counter,
            s.bytes.read_pages as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_write_pages_total,
            prometheus::ValueType::Counter,
            s.bytes.write_pages as f64,
            label_values,
        ));

        for transport in &s.transport {
            ch(prometheus::Metric::new(
                &self.nfs_transport_bind_total,
                prometheus::ValueType::Counter,
                transport.bind as f64,
                label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_transport_connect_total,
                prometheus::ValueType::Counter,
                transport.connect as f64,
                label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_transport_idle_time_seconds,
                prometheus::ValueType::Gauge,
                transport.idle_time_seconds % f64::MANTISSA_DIGITS as f64,
                label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_transport_sends_total,
                prometheus::ValueType::Counter,
                transport.sends as f64,
                label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_transport_receives_total,
                prometheus::ValueType::Counter,
                transport.receives as f64,
                label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_transport_bad_transaction_ids_total,
                prometheus::ValueType::Counter,
                transport.bad_transaction_ids as f64,
                label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_transport_backlog_queue_total,
                prometheus::ValueType::Counter,
                transport.cumulative_backlog as f64,
                label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_transport_maximum_rpc_slots,
                prometheus::ValueType::Gauge,
                transport.maximum_rpc_slots_used as f64,
                label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_transport_sending_queue_total,
                prometheus::ValueType::Counter,
                transport.cumulative_sending_queue as f64,
                label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_transport_pending_queue_total,
                prometheus::ValueType::Counter,
                transport.cumulative_pending_queue as f64,
                label_values,
            ));
        }

        for op in &s.operations {
            let op_label_values = &[export, protocol, mount_address, &op.operation];

            ch(prometheus::Metric::new(
                &self.nfs_operations_requests_total,
                prometheus::ValueType::Counter,
                op.requests as f64,
                op_label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_operations_transmissions_total,
                prometheus::ValueType::Counter,
                op.transmissions as f64,
                op_label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_operations_major_timeouts_total,
                prometheus::ValueType::Counter,
                op.major_timeouts as f64,
                op_label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_operations_sent_bytes_total,
                prometheus::ValueType::Counter,
                op.bytes_sent as f64,
                op_label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_operations_received_bytes_total,
                prometheus::ValueType::Counter,
                op.bytes_received as f64,
                op_label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_operations_queue_time_seconds_total,
                prometheus::ValueType::Counter,
                (op.cumulative_queue_milliseconds % f64::MANTISSA_DIGITS as f64) / 1000.0,
                op_label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_operations_response_time_seconds_total,
                prometheus::ValueType::Counter,
                (op.cumulative_total_response_milliseconds % f64::MANTISSA_DIGITS as f64) / 1000.0,
                op_label_values,
            ));

            ch(prometheus::Metric::new(
                &self.nfs_operations_request_time_seconds_total,
                prometheus::ValueType::Counter,
                (op.cumulative_total_request_milliseconds % f64::MANTISSA_DIGITS as f64) / 1000.0,
                op_label_values,
            ));
        }

        ch(prometheus::Metric::new(
            &self.nfs_event_inode_revalidate_total,
            prometheus::ValueType::Counter,
            s.events.inode_revalidate as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_dnode_revalidate_total,
            prometheus::ValueType::Counter,
            s.events.dnode_revalidate as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_data_invalidate_total,
            prometheus::ValueType::Counter,
            s.events.data_invalidate as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_attribute_invalidate_total,
            prometheus::ValueType::Counter,
            s.events.attribute_invalidate as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_open_total,
            prometheus::ValueType::Counter,
            s.events.vfs_open as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_lookup_total,
            prometheus::ValueType::Counter,
            s.events.vfs_lookup as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_access_total,
            prometheus::ValueType::Counter,
            s.events.vfs_access as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_update_page_total,
            prometheus::ValueType::Counter,
            s.events.vfs_update_page as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_read_page_total,
            prometheus::ValueType::Counter,
            s.events.vfs_read_page as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_read_pages_total,
            prometheus::ValueType::Counter,
            s.events.vfs_read_pages as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_write_page_total,
            prometheus::ValueType::Counter,
            s.events.vfs_write_page as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_write_pages_total,
            prometheus::ValueType::Counter,
            s.events.vfs_write_pages as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_getdents_total,
            prometheus::ValueType::Counter,
            s.events.vfs_getdents as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_setattr_total,
            prometheus::ValueType::Counter,
            s.events.vfs_setattr as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_flush_total,
            prometheus::ValueType::Counter,
            s.events.vfs_flush as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_fsync_total,
            prometheus::ValueType::Counter,
            s.events.vfs_fsync as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_lock_total,
            prometheus::ValueType::Counter,
            s.events.vfs_lock as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_vfs_file_release_total,
            prometheus::ValueType::Counter,
            s.events.vfs_file_release as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_truncation_total,
            prometheus::ValueType::Counter,
            s.events.truncation as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_write_extension_total,
            prometheus::ValueType::Counter,
            s.events.write_extension as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_silly_rename_total,
            prometheus::ValueType::Counter,
            s.events.silly_rename as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_short_read_total,
            prometheus::ValueType::Counter,
            s.events.short_read as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_short_write_total,
            prometheus::ValueType::Counter,
            s.events.short_write as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_jukebox_delay_total,
            prometheus::ValueType::Counter,
            s.events.jukebox_delay as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_pnfs_read_total,
            prometheus::ValueType::Counter,
            s.events.pnfs_read as f64,
            label_values,
        ));

        ch(prometheus::Metric::new(
            &self.nfs_event_pnfs_write_total,
            prometheus::ValueType::Counter,
            s.events.pnfs_write as f64,
            label_values,
        ));
    }
}


pub fn init() {
    register_collector("mountstats", default_disabled(), MountStatsCollector::new);
}