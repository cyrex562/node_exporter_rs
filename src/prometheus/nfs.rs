// Copyright 2018 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Package nfs implements parsing of /proc/net/rpc/nfsd.
//! Fields are documented in https://www.svennd.be/nfsd-stats-explained-procnetrpcnfsd/

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
pub struct ReplyCache {
    hits: u64,
    misses: u64,
    no_cache: u64,
}

#[derive(Debug)]
pub struct FileHandles {
    stale: u64,
    total_lookups: u64,
    anon_lookups: u64,
    dir_no_cache: u64,
    no_dir_no_cache: u64,
}

#[derive(Debug)]
pub struct InputOutput {
    read: u64,
    write: u64,
}

#[derive(Debug)]
pub struct Threads {
    threads: u64,
    full_cnt: u64,
}

#[derive(Debug)]
pub struct ReadAheadCache {
    cache_size: u64,
    cache_histogram: Vec<u64>,
    not_found: u64,
}

#[derive(Debug)]
pub struct Network {
    net_count: u64,
    udp_count: u64,
    tcp_count: u64,
    tcp_connect: u64,
}

#[derive(Debug)]
pub struct ClientRPC {
    rpc_count: u64,
    retransmissions: u64,
    auth_refreshes: u64,
}

#[derive(Debug)]
pub struct ServerRPC {
    rpc_count: u64,
    bad_cnt: u64,
    bad_fmt: u64,
    bad_auth: u64,
    badc_int: u64,
}

#[derive(Debug)]
pub struct V2Stats {
    null: u64,
    get_attr: u64,
    set_attr: u64,
    root: u64,
    lookup: u64,
    read_link: u64,
    read: u64,
    wr_cache: u64,
    write: u64,
    create: u64,
    remove: u64,
    rename: u64,
    link: u64,
    sym_link: u64,
    mk_dir: u64,
    rm_dir: u64,
    read_dir: u64,
    fs_stat: u64,
}

#[derive(Debug)]
pub struct V3Stats {
    null: u64,
    get_attr: u64,
    set_attr: u64,
    lookup: u64,
    access: u64,
    read_link: u64,
    read: u64,
    write: u64,
    create: u64,
    mk_dir: u64,
    sym_link: u64,
    mk_nod: u64,
    remove: u64,
    rm_dir: u64,
    rename: u64,
    link: u64,
    read_dir: u64,
    read_dir_plus: u64,
    fs_stat: u64,
    fs_info: u64,
    path_conf: u64,
    commit: u64,
}

#[derive(Debug)]
pub struct ClientV4Stats {
    null: u64,
    read: u64,
    write: u64,
    commit: u64,
    open: u64,
    open_confirm: u64,
    open_noattr: u64,
    open_downgrade: u64,
    close: u64,
    setattr: u64,
    fs_info: u64,
    renew: u64,
    set_client_id: u64,
    set_client_id_confirm: u64,
    lock: u64,
    lockt: u64,
    locku: u64,
    access: u64,
    getattr: u64,
    lookup: u64,
    lookup_root: u64,
    remove: u64,
    rename: u64,
    link: u64,
    symlink: u64,
    create: u64,
    pathconf: u64,
    stat_fs: u64,
    read_link: u64,
    read_dir: u64,
    server_caps: u64,
    deleg_return: u64,
    get_acl: u64,
    set_acl: u64,
    fs_locations: u64,
    release_lockowner: u64,
    secinfo: u64,
    fsid_present: u64,
    exchange_id: u64,
    create_session: u64,
    destroy_session: u64,
    sequence: u64,
    get_lease_time: u64,
    reclaim_complete: u64,
    layout_get: u64,
    get_device_info: u64,
    layout_commit: u64,
    layout_return: u64,
    secinfo_no_name: u64,
    test_state_id: u64,
    free_state_id: u64,
    get_device_list: u64,
    bind_conn_to_session: u64,
    destroy_client_id: u64,
    seek: u64,
    allocate: u64,
    de_allocate: u64,
    layout_stats: u64,
    clone: u64,
}

#[derive(Debug)]
pub struct ServerV4Stats {
    null: u64,
    compound: u64,
}

#[derive(Debug)]
pub struct V4Ops {
    op0_unused: u64,
    op1_unused: u64,
    op2_future: u64,
    access: u64,
    close: u64,
    commit: u64,
    create: u64,
    deleg_purge: u64,
    deleg_return: u64,
    get_attr: u64,
    get_fh: u64,
    link: u64,
    lock: u64,
    lockt: u64,
    locku: u64,
    lookup: u64,
    lookup_root: u64,
    nverify: u64,
    open: u64,
    open_attr: u64,
    open_confirm: u64,
    open_dgrd: u64,
    put_fh: u64,
    put_pub_fh: u64,
    put_root_fh: u64,
    read: u64,
    read_dir: u64,
    read_link: u64,
    remove: u64,
    rename: u64,
    renew: u64,
    restore_fh: u64,
    save_fh: u64,
    sec_info: u64,
    set_attr: u64,
    set_client_id: u64,
    set_client_id_confirm: u64,
    verify: u64,
    write: u64,
    rel_lock_owner: u64,
}

#[derive(Debug)]
pub struct ClientRPCStats {
    network: Network,
    client_rpc: ClientRPC,
    v2_stats: V2Stats,
    v3_stats: V3Stats,
    client_v4_stats: ClientV4Stats,
}

#[derive(Debug)]
pub struct ServerRPCStats {
    reply_cache: ReplyCache,
    file_handles: FileHandles,
    input_output: InputOutput,
    threads: Threads,
    read_ahead_cache: ReadAheadCache,
    network: Network,
    server_rpc: ServerRPC,
    v2_stats: V2Stats,
    v3_stats: V3Stats,
    server_v4_stats: ServerV4Stats,
    v4_ops: V4Ops,
    wdeleg_getattr: u64,
}

pub struct FS {
    proc: fs::FS,
}

impl FS {
    pub fn new_default_fs() -> Result<Self, std::io::Error> {
        Self::new(fs::DEFAULT_PROC_MOUNT_POINT)
    }

    pub fn new(mount_point: &str) -> Result<Self, std::io::Error> {
        let mount_point = if mount_point.trim().is_empty() {
            fs::DEFAULT_PROC_MOUNT_POINT
        } else {
            mount_point
        };
        let proc = fs::FS::new(mount_point)?;
        Ok(FS { proc })
    }

    pub fn client_rpc_stats(&self) -> Result<ClientRPCStats, std::io::Error> {
        let path = self.proc.path("net/rpc/nfs");
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        parse_client_rpc_stats(&contents)
    }

    pub fn server_rpc_stats(&self) -> Result<ServerRPCStats, std::io::Error> {
        let path = self.proc.path("net/rpc/nfsd");
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        parse_server_rpc_stats(&contents)
    }
}

use std::io::{self, BufRead};
use std::str::FromStr;

pub fn parse_client_rpc_stats<R: BufRead>(reader: R) -> Result<ClientRPCStats, io::Error> {
    let mut stats = ClientRPCStats::default();
    let mut lines = reader.lines();

    while let Some(line) = lines.next() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("invalid NFS metric line {}", line)));
        }

        let values: Result<Vec<u64>, _> = parts[1..].iter().map(|v| u64::from_str(v)).collect();
        let values = values.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        match parts[0] {
            "net" => stats.network = parse_network(&values)?,
            "rpc" => stats.client_rpc = parse_client_rpc(&values)?,
            "proc2" => stats.v2_stats = parse_v2_stats(&values)?,
            "proc3" => stats.v3_stats = parse_v3_stats(&values)?,
            "proc4" => stats.client_v4_stats = parse_client_v4_stats(&values)?,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unknown NFS metric line {}", parts[0]))),
        }
    }

    Ok(stats)
}

use std::io::{self, BufRead};
use std::str::FromStr;

pub fn parse_server_rpc_stats<R: BufRead>(reader: R) -> Result<ServerRPCStats, io::Error> {
    let mut stats = ServerRPCStats::default();
    let mut lines = reader.lines();

    while let Some(line) = lines.next() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("invalid NFSd metric line {}", line)));
        }

        let values: Result<Vec<u64>, _> = if parts[0] == "th" && parts.len() >= 3 {
            parts[1..3].iter().map(|v| u64::from_str(v)).collect()
        } else {
            parts[1..].iter().map(|v| u64::from_str(v)).collect()
        };
        let values = values.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        match parts[0] {
            "rc" => stats.reply_cache = parse_reply_cache(&values)?,
            "fh" => stats.file_handles = parse_file_handles(&values)?,
            "io" => stats.input_output = parse_input_output(&values)?,
            "th" => stats.threads = parse_threads(&values)?,
            "ra" => stats.read_ahead_cache = parse_read_ahead_cache(&values)?,
            "net" => stats.network = parse_network(&values)?,
            "rpc" => stats.server_rpc = parse_server_rpc(&values)?,
            "proc2" => stats.v2_stats = parse_v2_stats(&values)?,
            "proc3" => stats.v3_stats = parse_v3_stats(&values)?,
            "proc4" => stats.server_v4_stats = parse_server_v4_stats(&values)?,
            "proc4ops" => stats.v4_ops = parse_v4_ops(&values)?,
            "wdeleg_getattr" => stats.wdeleg_getattr = values[0],
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unknown NFSd metric line {}", parts[0]))),
        }
    }

    Ok(stats)
}

use std::fmt;

fn parse_reply_cache(v: &[u64]) -> Result<ReplyCache, String> {
    if v.len() != 3 {
        return Err(format!("invalid ReplyCache line {:?}", v));
    }
    Ok(ReplyCache {
        hits: v[0],
        misses: v[1],
        no_cache: v[2],
    })
}

fn parse_file_handles(v: &[u64]) -> Result<FileHandles, String> {
    if v.len() != 5 {
        return Err(format!("invalid FileHandles line {:?}", v));
    }
    Ok(FileHandles {
        stale: v[0],
        total_lookups: v[1],
        anon_lookups: v[2],
        dir_no_cache: v[3],
        no_dir_no_cache: v[4],
    })
}

fn parse_input_output(v: &[u64]) -> Result<InputOutput, String> {
    if v.len() != 2 {
        return Err(format!("invalid InputOutput line {:?}", v));
    }
    Ok(InputOutput {
        read: v[0],
        write: v[1],
    })
}

fn parse_threads(v: &[u64]) -> Result<Threads, String> {
    if v.len() != 2 {
        return Err(format!("invalid Threads line {:?}", v));
    }
    Ok(Threads {
        threads: v[0],
        full_cnt: v[1],
    })
}

fn parse_read_ahead_cache(v: &[u64]) -> Result<ReadAheadCache, String> {
    if v.len() != 12 {
        return Err(format!("invalid ReadAheadCache line {:?}", v));
    }
    Ok(ReadAheadCache {
        cache_size: v[0],
        cache_histogram: v[1..11].to_vec(),
        not_found: v[11],
    })
}

fn parse_network(v: &[u64]) -> Result<Network, String> {
    if v.len() != 4 {
        return Err(format!("invalid Network line {:?}", v));
    }
    Ok(Network {
        net_count: v[0],
        udp_count: v[1],
        tcp_count: v[2],
        tcp_connect: v[3],
    })
}

fn parse_server_rpc(v: &[u64]) -> Result<ServerRPC, String> {
    if v.len() != 5 {
        return Err(format!("invalid ServerRPC line {:?}", v));
    }
    Ok(ServerRPC {
        rpc_count: v[0],
        bad_cnt: v[1],
        bad_fmt: v[2],
        bad_auth: v[3],
        badc_int: v[4],
    })
}

fn parse_client_rpc(v: &[u64]) -> Result<ClientRPC, String> {
    if v.len() != 3 {
        return Err(format!("invalid ClientRPC line {:?}", v));
    }
    Ok(ClientRPC {
        rpc_count: v[0],
        retransmissions: v[1],
        auth_refreshes: v[2],
    })
}

fn parse_v2_stats(v: &[u64]) -> Result<V2Stats, String> {
    let values = v[0] as usize;
    if v.len() != values + 1 || values < 18 {
        return Err(format!("invalid V2Stats line {:?}", v));
    }
    Ok(V2Stats {
        null: v[1],
        get_attr: v[2],
        set_attr: v[3],
        root: v[4],
        lookup: v[5],
        read_link: v[6],
        read: v[7],
        wr_cache: v[8],
        write: v[9],
        create: v[10],
        remove: v[11],
        rename: v[12],
        link: v[13],
        sym_link: v[14],
        mk_dir: v[15],
        rm_dir: v[16],
        read_dir: v[17],
        fs_stat: v[18],
    })
}

fn parse_v3_stats(v: &[u64]) -> Result<V3Stats, String> {
    let values = v[0] as usize;
    if v.len() != values + 1 || values < 22 {
        return Err(format!("invalid V3Stats line {:?}", v));
    }
    Ok(V3Stats {
        null: v[1],
        get_attr: v[2],
        set_attr: v[3],
        lookup: v[4],
        access: v[5],
        read_link: v[6],
        read: v[7],
        write: v[8],
        create: v[9],
        mk_dir: v[10],
        sym_link: v[11],
        mk_nod: v[12],
        remove: v[13],
        rm_dir: v[14],
        rename: v[15],
        link: v[16],
        read_dir: v[17],
        read_dir_plus: v[18],
        fs_stat: v[19],
        fs_info: v[20],
        path_conf: v[21],
        commit: v[22],
    })
}

fn parse_client_v4_stats(v: &[u64]) -> Result<ClientV4Stats, String> {
    let values = v[0] as usize;
    if v.len() != values + 1 {
        return Err(format!("invalid ClientV4Stats line {:?}", v));
    }

    let mut v = v.to_vec();
    if values < 59 {
        v.resize(60, 0);
    }

    Ok(ClientV4Stats {
        null: v[1],
        read: v[2],
        write: v[3],
        commit: v[4],
        open: v[5],
        open_confirm: v[6],
        open_noattr: v[7],
        open_downgrade: v[8],
        close: v[9],
        setattr: v[10],
        fs_info: v[11],
        renew: v[12],
        set_client_id: v[13],
        set_client_id_confirm: v[14],
        lock: v[15],
        lockt: v[16],
        locku: v[17],
        access: v[18],
        getattr: v[19],
        lookup: v[20],
        lookup_root: v[21],
        remove: v[22],
        rename: v[23],
        link: v[24],
        symlink: v[25],
        create: v[26],
        pathconf: v[27],
        stat_fs: v[28],
        read_link: v[29],
        read_dir: v[30],
        server_caps: v[31],
        deleg_return: v[32],
        get_acl: v[33],
        set_acl: v[34],
        fs_locations: v[35],
        release_lockowner: v[36],
        secinfo: v[37],
        fsid_present: v[38],
        exchange_id: v[39],
        create_session: v[40],
        destroy_session: v[41],
        sequence: v[42],
        get_lease_time: v[43],
        reclaim_complete: v[44],
        layout_get: v[45],
        get_device_info: v[46],
        layout_commit: v[47],
        layout_return: v[48],
        secinfo_no_name: v[49],
        test_state_id: v[50],
        free_state_id: v[51],
        get_device_list: v[52],
        bind_conn_to_session: v[53],
        destroy_client_id: v[54],
        seek: v[55],
        allocate: v[56],
        de_allocate: v[57],
        layout_stats: v[58],
        clone: v[59],
    })
}

fn parse_server_v4_stats(v: &[u64]) -> Result<ServerV4Stats, String> {
    let values = v[0] as usize;
    if v.len() != values + 1 || values != 2 {
        return Err(format!("invalid ServerV4Stats line {:?}", v));
    }
    Ok(ServerV4Stats {
        null: v[1],
        compound: v[2],
    })
}

fn parse_v4_ops(v: &[u64]) -> Result<V4Ops, String> {
    let values = v[0] as usize;
    if v.len() != values + 1 || values < 39 {
        return Err(format!("invalid V4Ops line {:?}", v));
    }

    let v40 = if values > 39 { v[40] } else { 0 };

    Ok(V4Ops {
        op0_unused: v[1],
        op1_unused: v[2],
        op2_future: v[3],
        access: v[4],
        close: v[5],
        commit: v[6],
        create: v[7],
        deleg_purge: v[8],
        deleg_return: v[9],
        get_attr: v[10],
        get_fh: v[11],
        link: v[12],
        lock: v[13],
        lockt: v[14],
        locku: v[15],
        lookup: v[16],
        lookup_root: v[17],
        nverify: v[18],
        open: v[19],
        open_attr: v[20],
        open_confirm: v[21],
        open_dgrd: v[22],
        put_fh: v[23],
        put_pub_fh: v[24],
        put_root_fh: v[25],
        read: v[26],
        read_dir: v[27],
        read_link: v[28],
        remove: v[29],
        rename: v[30],
        renew: v[31],
        restore_fh: v[32],
        save_fh: v[33],
        sec_info: v[34],
        set_attr: v[35],
        set_client_id: v[36],
        set_client_id_confirm: v[37],
        verify: v[38],
        write: v[39],
        rel_lock_owner: v40,
    })
}