pub const DEVICE_ENTRY_LEN: usize = 8;

pub const FIELD_BYTES_LEN: usize = 8;
pub const FIELD_EVENTS_LEN: usize = 27;

pub const STAT_VERSION_10: &str = "1.0";
pub const STAT_VERSION_11: &str = "1.1";

pub const FIELD_TRANSPORT_10_TCP_LEN: usize = 10;
pub const FIELD_TRANSPORT_10_UDP_LEN: usize = 7;

pub const FIELD_TRANSPORT_11_TCP_LEN: usize = 13;
pub const FIELD_TRANSPORT_11_UDP_LEN: usize = 10;

// kernel version >= 4.14 MaxLen
// See: https://elixir.bootlin.com/linux/v6.4.8/source/net/sunrpc/xprtrdma/xprt_rdma.h#L393
pub const FIELD_TRANSPORT_11_RDMA_MAX_LEN: usize = 28;

// kernel version <= 4.2 MinLen
// See: https://elixir.bootlin.com/linux/v4.2.8/source/net/sunrpc/xprtrdma/xprt_rdma.h#L331
pub const FIELD_TRANSPORT_11_RDMA_MIN_LEN: usize = 20;

pub struct Mount {
    // Name of the device.
    pub device: String,
    // The mount point of the device.
    pub mount: String,
    // The filesystem type used by the device.
    pub fs_type: String,
    // If available additional statistics related to this Mount.
    // Use a type assertion to determine if additional statistics are available.
    pub stats: MountStats,
}

pub trait MountStats {
    fn mount_stats(&self);
}

use std::collections::HashMap;
use std::time::Duration;

pub struct MountStatsNFS {
    // The version of statistics provided.
    pub stat_version: String,
    // The mount options of the NFS mount.
    pub opts: HashMap<String, String>,
    // The age of the NFS mount.
    pub age: Duration,
    // Statistics related to byte counters for various operations.
    pub bytes: NFSBytesStats,
    // Statistics related to various NFS event occurrences.
    pub events: NFSEventsStats,
    // Statistics broken down by filesystem operation.
    pub operations: Vec<NFSOperationStats>,
    // Statistics about the NFS RPC transport.
    pub transport: Vec<NFSTransportStats>,
}

impl MountStats for MountStatsNFS {
    fn mount_stats(&self) {}
}

pub struct NFSBytesStats {
    // Number of bytes read using the read() syscall.
    pub read: u64,
    // Number of bytes written using the write() syscall.
    pub write: u64,
    // Number of bytes read using the read() syscall in O_DIRECT mode.
    pub direct_read: u64,
    // Number of bytes written using the write() syscall in O_DIRECT mode.
    pub direct_write: u64,
    // Number of bytes read from the NFS server, in total.
    pub read_total: u64,
    // Number of bytes written to the NFS server, in total.
    pub write_total: u64,
    // Number of pages read directly via mmap()'d files.
    pub read_pages: u64,
    // Number of pages written directly via mmap()'d files.
    pub write_pages: u64,
}

pub struct NFSEventsStats {
    // Number of times cached inode attributes are re-validated from the server.
    pub inode_revalidate: u64,
    // Number of times cached dentry nodes are re-validated from the server.
    pub dnode_revalidate: u64,
    // Number of times an inode cache is cleared.
    pub data_invalidate: u64,
    // Number of times cached inode attributes are invalidated.
    pub attribute_invalidate: u64,
    // Number of times files or directories have been open()'d.
    pub vfs_open: u64,
    // Number of times a directory lookup has occurred.
    pub vfs_lookup: u64,
    // Number of times permissions have been checked.
    pub vfs_access: u64,
    // Number of updates (and potential writes) to pages.
    pub vfs_update_page: u64,
    // Number of pages read directly via mmap()'d files.
    pub vfs_read_page: u64,
    // Number of times a group of pages have been read.
    pub vfs_read_pages: u64,
    // Number of pages written directly via mmap()'d files.
    pub vfs_write_page: u64,
    // Number of times a group of pages have been written.
    pub vfs_write_pages: u64,
    // Number of times directory entries have been read with getdents().
    pub vfs_getdents: u64,
    // Number of times attributes have been set on inodes.
    pub vfs_setattr: u64,
    // Number of pending writes that have been forcefully flushed to the server.
    pub vfs_flush: u64,
    // Number of times fsync() has been called on directories and files.
    pub vfs_fsync: u64,
    // Number of times locking has been attempted on a file.
    pub vfs_lock: u64,
    // Number of times files have been closed and released.
    pub vfs_file_release: u64,
    // Unknown. Possibly unused.
    pub congestion_wait: u64,
    // Number of times files have been truncated.
    pub truncation: u64,
    // Number of times a file has been grown due to writes beyond its existing end.
    pub write_extension: u64,
    // Number of times a file was removed while still open by another process.
    pub silly_rename: u64,
    // Number of times the NFS server gave less data than expected while reading.
    pub short_read: u64,
    // Number of times the NFS server wrote less data than expected while writing.
    pub short_write: u64,
    // Number of times the NFS server indicated EJUKEBOX; retrieving data from
    // offline storage.
    pub jukebox_delay: u64,
    // Number of NFS v4.1+ pNFS reads.
    pub pnfs_read: u64,
    // Number of NFS v4.1+ pNFS writes.
    pub pnfs_write: u64,
}

pub struct NFSOperationStats {
    // The name of the operation.
    pub operation: String,
    // Number of requests performed for this operation.
    pub requests: u64,
    // Number of times an actual RPC request has been transmitted for this operation.
    pub transmissions: u64,
    // Number of times a request has had a major timeout.
    pub major_timeouts: u64,
    // Number of bytes sent for this operation, including RPC headers and payload.
    pub bytes_sent: u64,
    // Number of bytes received for this operation, including RPC headers and payload.
    pub bytes_received: u64,
    // Duration all requests spent queued for transmission before they were sent.
    pub cumulative_queue_milliseconds: u64,
    // Duration it took to get a reply back after the request was transmitted.
    pub cumulative_total_response_milliseconds: u64,
    // Duration from when a request was enqueued to when it was completely handled.
    pub cumulative_total_request_milliseconds: u64,
    // The count of operations that complete with tk_status < 0. These statuses usually indicate error conditions.
    pub errors: u64,
}

pub struct NFSTransportStats {
    // The transport protocol used for the NFS mount.
    pub protocol: String,
    // The local port used for the NFS mount.
    pub port: u64,
    // Number of times the client has had to establish a connection from scratch
    // to the NFS server.
    pub bind: u64,
    // Number of times the client has made a TCP connection to the NFS server.
    pub connect: u64,
    // Duration (in jiffies, a kernel internal unit of time) the NFS mount has
    // spent waiting for connections to the server to be established.
    pub connect_idle_time: u64,
    // Duration since the NFS mount last saw any RPC traffic.
    pub idle_time_seconds: u64,
    // Number of RPC requests for this mount sent to the NFS server.
    pub sends: u64,
    // Number of RPC responses for this mount received from the NFS server.
    pub receives: u64,
    // Number of times the NFS server sent a response with a transaction ID
    // unknown to this client.
    pub bad_transaction_ids: u64,
    // A running counter, incremented on each request as the current difference
    // between sends and receives.
    pub cumulative_active_requests: u64,
    // A running counter, incremented on each request by the current backlog
    // queue size.
    pub cumulative_backlog: u64,

    // Stats below only available with stat version 1.1.

    // Maximum number of simultaneously active RPC requests ever used.
    pub maximum_rpc_slots_used: u64,
    // A running counter, incremented on each request as the current size of the
    // sending queue.
    pub cumulative_sending_queue: u64,
    // A running counter, incremented on each request as the current size of the
    // pending queue.
    pub cumulative_pending_queue: u64,

    // Stats below only available with stat version 1.1.
    // Transport over RDMA

    // accessed when sending a call
    pub read_chunk_count: u64,
    pub write_chunk_count: u64,
    pub reply_chunk_count: u64,
    pub total_rdma_request: u64,

    // rarely accessed error counters
    pub pullup_copy_count: u64,
    pub hardway_register_count: u64,
    pub failed_marshal_count: u64,
    pub bad_reply_count: u64,
    pub mrs_recovered: u64,
    pub mrs_orphaned: u64,
    pub mrs_allocated: u64,
    pub empty_sendctx_q: u64,

    // accessed when receiving a reply
    pub total_rdma_reply: u64,
    pub fixup_copy_count: u64,
    pub reply_waits_for_send: u64,
    pub local_inv_needed: u64,
    pub nomsg_call_count: u64,
    pub bcall_count: u64,
}

pub fn parse_mount_stats<R: BufRead>(r: R) -> Result<Vec<Mount>, String> {
    let mut mounts = Vec::new();
    let mut s = io::BufReader::new(r).lines();

    while let Some(line) = s.next() {
        let line = line.map_err(|e| e.to_string())?;
        let ss: Vec<&str> = line.split_whitespace().collect();
        if ss.is_empty() || ss[0] != DEVICE {
            continue;
        }

        let m = parse_mount(&ss).map_err(|e| e.to_string())?;

        if ss.len() > DEVICE_ENTRY_LEN {
            if m.fs_type != NFS3_TYPE && m.fs_type != NFS4_TYPE {
                return Err(format!("Cannot parse MountStats for {}", m.fs_type));
            }

            let stat_version = ss[8].trim_start_matches(STAT_VERSION_PREFIX);
            let stats = parse_mount_stats_nfs(&mut s, stat_version).map_err(|e| e.to_string())?;
            m.stats = Some(stats);
        }

        mounts.push(m);
    }

    Ok(mounts)
}

#[derive(Debug)]
pub struct FileParseError;

impl fmt::Display for FileParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "File parse error")
    }
}

impl Error for FileParseError {}

pub fn parse_mount(ss: &[&str]) -> Result<Mount, Box<dyn Error>> {
    if ss.len() < DEVICE_ENTRY_LEN {
        return Err(Box::new(FileParseError));
    }

    let format = [
        (0, "device"),
        (2, "mounted"),
        (3, "on"),
        (5, "with"),
        (6, "fstype"),
    ];

    for &(i, s) in &format {
        if ss[i] != s {
            return Err(Box::new(FileParseError));
        }
    }

    Ok(Mount {
        device: ss[1].to_string(),
        mount: ss[4].to_string(),
        fs_type: ss[7].to_string(),
        stats: todo!(),
    })
}

pub fn parse_mount_stats_nfs<R: BufRead>(s: &mut R, stat_version: &str) -> Result<MountStatsNFS, Box<dyn std::error::Error>> {
    const FIELD_OPTS: &str = "opts:";
    const FIELD_AGE: &str = "age:";
    const FIELD_BYTES: &str = "bytes:";
    const FIELD_EVENTS: &str = "events:";
    const FIELD_PER_OP_STATS: &str = "per-op";
    const FIELD_TRANSPORT: &str = "xprt:";

    let mut stats = MountStatsNFS {
        stat_version: stat_version.to_string(),
        opts: HashMap::new(),
        age: Duration::new(0, 0),
        bytes: NFSBytesStats::default(),
        events: NFSEventsStats::default(),
        transport: Vec::new(),
        operations: Vec::new(),
    };

    let mut lines = s.lines();
    while let Some(line) = lines.next() {
        let line = line?;
        let ss: Vec<&str> = line.split_whitespace().collect();
        if ss.is_empty() {
            break;
        }

        match ss[0] {
            FIELD_OPTS => {
                if ss.len() < 2 {
                    return Err("Incomplete information for NFS stats".into());
                }
                for opt in ss[1].split(',') {
                    let split: Vec<&str> = opt.split('=').collect();
                    if split.len() == 2 {
                        stats.opts.insert(split[0].to_string(), split[1].to_string());
                    } else {
                        stats.opts.insert(opt.to_string(), String::new());
                    }
                }
            }
            FIELD_AGE => {
                if ss.len() < 2 {
                    return Err("Incomplete information for NFS stats".into());
                }
                let d = Duration::from_secs(u64::from_str(ss[1])?);
                stats.age = d;
            }
            FIELD_BYTES => {
                if ss.len() < 2 {
                    return Err("Incomplete information for NFS stats".into());
                }
                stats.bytes = parse_nfs_bytes_stats(&ss[1..])?;
            }
            FIELD_EVENTS => {
                if ss.len() < 2 {
                    return Err("Incomplete information for NFS events".into());
                }
                stats.events = parse_nfs_events_stats(&ss[1..])?;
            }
            FIELD_TRANSPORT => {
                if ss.len() < 3 {
                    return Err("Incomplete information for NFS transport stats".into());
                }
                let tstats = parse_nfs_transport_stats(&ss[1..], stat_version)?;
                stats.transport.push(tstats);
            }
            _ => {}
        }

        if ss[0] == FIELD_PER_OP_STATS {
            break;
        }
    }

    stats.operations = parse_nfs_operation_stats(&mut lines)?;

    Ok(stats)
}

pub fn parse_nfs_bytes_stats(ss: &[&str]) -> Result<NFSBytesStats, Box<dyn std::error::Error>> {
    if ss.len() != FIELD_BYTES_LEN {
        return Err(format!("Invalid NFS bytes stats: {:?}", ss).into());
    }

    let mut ns = Vec::with_capacity(FIELD_BYTES_LEN);
    for s in ss {
        let n = s.parse::<u64>()?;
        ns.push(n);
    }

    Ok(NFSBytesStats {
        read: ns[0],
        write: ns[1],
        direct_read: ns[2],
        direct_write: ns[3],
        read_total: ns[4],
        write_total: ns[5],
        read_pages: ns[6],
        write_pages: ns[7],
    })
}

pub fn parse_nfs_events_stats(ss: &[&str]) -> Result<NFSEventsStats, Box<dyn std::error::Error>> {
    if ss.len() != FIELD_EVENTS_LEN {
        return Err(format!("Invalid NFS events stats: {:?}", ss).into());
    }

    let mut ns = Vec::with_capacity(FIELD_EVENTS_LEN);
    for s in ss {
        let n = s.parse::<u64>()?;
        ns.push(n);
    }

    Ok(NFSEventsStats {
        inode_revalidate: ns[0],
        dnode_revalidate: ns[1],
        data_invalidate: ns[2],
        attribute_invalidate: ns[3],
        vfs_open: ns[4],
        vfs_lookup: ns[5],
        vfs_access: ns[6],
        vfs_update_page: ns[7],
        vfs_read_page: ns[8],
        vfs_read_pages: ns[9],
        vfs_write_page: ns[10],
        vfs_write_pages: ns[11],
        vfs_getdents: ns[12],
        vfs_setattr: ns[13],
        vfs_flush: ns[14],
        vfs_fsync: ns[15],
        vfs_lock: ns[16],
        vfs_file_release: ns[17],
        congestion_wait: ns[18],
        truncation: ns[19],
        write_extension: ns[20],
        silly_rename: ns[21],
        short_read: ns[22],
        short_write: ns[23],
        jukebox_delay: ns[24],
        pnfs_read: ns[25],
        pnfs_write: ns[26],
    })
}

pub fn parse_nfs_operation_stats<R: BufRead>(s: &mut R) -> Result<Vec<NFSOperationStats>, Box<dyn std::error::Error>> {
    let mut ops = Vec::new();
    let mut lines = s.lines();

    while let Some(line) = lines.next() {
        let line = line?;
        let ss: Vec<&str> = line.split_whitespace().collect();
        if ss.is_empty() {
            break;
        }

        if ss.len() < MIN_FIELDS {
            return Err(format!("Invalid NFS per-operations stats: {:?}", ss).into());
        }

        let mut ns = Vec::with_capacity(MIN_FIELDS - 1);
        for st in &ss[1..] {
            let n = st.parse::<u64>()?;
            ns.push(n);
        }

        let op_stats = NFSOperationStats {
            operation: ss[0].trim_end_matches(':').to_string(),
            requests: ns[0],
            transmissions: ns[1],
            major_timeouts: ns[2],
            bytes_sent: ns[3],
            bytes_received: ns[4],
            cumulative_queue_milliseconds: ns[5],
            cumulative_total_response_milliseconds: ns[6],
            cumulative_total_request_milliseconds: ns[7],
            errors: if ns.len() > 8 { Some(ns[8]) } else { None },
        };

        ops.push(op_stats);
    }

    Ok(ops)
}

pub fn parse_nfs_transport_stats(ss: &[&str], stat_version: &str) -> Result<NFSTransportStats, ParseError> {
    let protocol = ss[0].to_string();
    let ss = &ss[1..];

    let expected_length = match (stat_version, protocol.as_str()) {
        (STAT_VERSION_10, "tcp") => FIELD_TRANSPORT_10_TCP_LEN,
        (STAT_VERSION_10, "udp") => FIELD_TRANSPORT_10_UDP_LEN,
        (STAT_VERSION_11, "tcp") => FIELD_TRANSPORT_11_TCP_LEN,
        (STAT_VERSION_11, "udp") => FIELD_TRANSPORT_11_UDP_LEN,
        (STAT_VERSION_11, "rdma") => FIELD_TRANSPORT_11_RDMA_MIN_LEN,
        _ => return Err(ParseError::InvalidProtocol(protocol)),
    };

    if (protocol == "rdma" && ss.len() < expected_length) || (protocol != "rdma" && ss.len() != expected_length) {
        return Err(ParseError::InvalidLength(format!("Invalid length for protocol {}: {:?}", protocol, ss)));
    }

    let mut ns = vec![0u64; FIELD_TRANSPORT_11_RDMA_MAX_LEN + 3];
    for (i, s) in ss.iter().enumerate() {
        ns[i] = s.parse()?;
    }

    if protocol == "udp" {
        ns.splice(2..2, vec![0, 0, 0]);
    } else if protocol == "tcp" {
        ns.splice(FIELD_TRANSPORT_11_TCP_LEN..FIELD_TRANSPORT_11_RDMA_MAX_LEN + 3, vec![0; FIELD_TRANSPORT_11_RDMA_MAX_LEN - FIELD_TRANSPORT_11_TCP_LEN + 3]);
    } else if protocol == "rdma" {
        ns.splice(FIELD_TRANSPORT_10_TCP_LEN..FIELD_TRANSPORT_10_TCP_LEN, vec![0, 0, 0]);
    }

    Ok(NFSTransportStats {
        protocol,
        port: ns[0],
        bind: ns[1],
        connect: ns[2],
        connect_idle_time: ns[3],
        idle_time_seconds: ns[4],
        sends: ns[5],
        receives: ns[6],
        bad_transaction_ids: ns[7],
        cumulative_active_requests: ns[8],
        cumulative_backlog: ns[9],
        maximum_rpc_slots_used: ns[10],
        cumulative_sending_queue: ns[11],
        cumulative_pending_queue: ns[12],
        read_chunk_count: ns[13],
        write_chunk_count: ns[14],
        reply_chunk_count: ns[15],
        total_rdma_request: ns[16],
        pullup_copy_count: ns[17],
        hardway_register_count: ns[18],
        failed_marshal_count: ns[19],
        bad_reply_count: ns[20],
        mrs_recovered: ns[21],
        mrs_orphaned: ns[22],
        mrs_allocated: ns[23],
        empty_sendctx_q: ns[24],
        total_rdma_reply: ns[25],
        fixup_copy_count: ns[26],
        reply_waits_for_send: ns[27],
        local_inv_needed: ns[28],
        nomsg_call_count: ns[29],
        bcall_count: ns[30],
    })
}