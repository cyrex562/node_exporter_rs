use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

struct Fscacheinfo {
    index_cookies_allocated: u64,
    data_storage_cookies_allocated: u64,
    special_cookies_allocated: u64,
    objects_allocated: u64,
    object_allocations_failure: u64,
    objects_available: u64,
    objects_dead: u64,
    objects_without_coherency_check: u64,
    objects_with_coherency_check: u64,
    objects_need_coherency_check_update: u64,
    objects_declared_obsolete: u64,
    pages_marked_as_being_cached: u64,
    uncache_pages_request_seen: u64,
    acquire_cookies_request_seen: u64,
    acquire_requests_with_null_parent: u64,
    acquire_requests_rejected_no_cache_available: u64,
    acquire_requests_succeeded: u64,
    acquire_requests_rejected_due_to_error: u64,
    acquire_requests_failed_due_to_enomem: u64,
    lookups_number: u64,
    lookups_negative: u64,
    lookups_positive: u64,
    objects_created_by_lookup: u64,
    lookups_timed_out_and_requed: u64,
    invalidations_number: u64,
    invalidations_running: u64,
    update_cookie_request_seen: u64,
    update_requests_with_null_parent: u64,
    update_requests_running: u64,
    relinquish_cookies_request_seen: u64,
    relinquish_cookies_with_null_parent: u64,
    relinquish_requests_waiting_complete_creation: u64,
    relinquish_retries: u64,
    attribute_changed_requests_seen: u64,
    attribute_changed_requests_queued: u64,
    attribute_changed_reject_due_to_enobufs: u64,
    attribute_changed_failed_due_to_enomem: u64,
    attribute_changed_ops: u64,
    allocation_requests_seen: u64,
    allocation_ok_requests: u64,
    allocation_waiting_on_lookup: u64,
    allocations_rejected_due_to_enobufs: u64,
    allocations_aborted_due_to_erestartsys: u64,
    allocation_operations_submitted: u64,
    allocations_waited_for_cpu: u64,
    allocations_aborted_due_to_object_death: u64,
    retrievals_read_requests: u64,
    retrievals_ok: u64,
    retrievals_waiting_lookup_completion: u64,
    retrievals_returned_enodata: u64,
    retrievals_rejected_due_to_enobufs: u64,
    retrievals_aborted_due_to_erestartsys: u64,
    retrievals_failed_due_to_enomem: u64,
    retrievals_requests: u64,
    retrievals_waiting_cpu: u64,
    retrievals_aborted_due_to_object_death: u64,
    store_write_requests: u64,
    store_successful_requests: u64,
    store_requests_on_pending_storage: u64,
    store_requests_rejected_due_to_enobufs: u64,
    store_requests_failed_due_to_enomem: u64,
    store_requests_submitted: u64,
    store_requests_running: u64,
    store_pages_with_requests_processing: u64,
    store_requests_deleted: u64,
    store_requests_over_store_limit: u64,
    release_requests_against_pages_with_no_pending_storage: u64,
    release_requests_against_pages_stored_by_time_lock_granted: u64,
    release_requests_ignored_due_to_in_progress_store: u64,
    page_stores_cancelled_by_release_requests: u64,
    vmscan_waiting: u64,
    ops_pending: u64,
    ops_running: u64,
    ops_enqueued: u64,
    ops_cancelled: u64,
    ops_rejected: u64,
    ops_initialised: u64,
    ops_deferred: u64,
    ops_released: u64,
    ops_garbage_collected: u64,
    cacheop_allocations_in_progress: u64,
    cacheop_lookup_object_in_progress: u64,
    cacheop_lookup_complete_in_progress: u64,
    cacheop_grab_object_in_progress: u64,
    cacheop_invalidations: u64,
    cacheop_update_object_in_progress: u64,
    cacheop_drop_object_in_progress: u64,
    cacheop_put_object_in_progress: u64,
    cacheop_attribute_change_in_progress: u64,
    cacheop_sync_cache_in_progress: u64,
    cacheop_read_or_alloc_page_in_progress: u64,
    cacheop_read_or_alloc_pages_in_progress: u64,
    cacheop_allocate_page_in_progress: u64,
    cacheop_allocate_pages_in_progress: u64,
    cacheop_write_pages_in_progress: u64,
    cacheop_uncache_pages_in_progress: u64,
    cacheop_dissociate_pages_in_progress: u64,
    cacheev_lookups_and_creations_rejected_lack_space: u64,
    cacheev_stale_objects_deleted: u64,
    cacheev_retired_when_relinquished: u64,
    cacheev_objects_culled: u64,
}

impl Fscacheinfo {
    fn from_reader<R: BufRead>(reader: R) -> Result<Self, Box<dyn std::error::Error>> {
        let mut info = Fscacheinfo::default();
        let mut lines = reader.lines();

        while let Some(line) = lines.next() {
            let line = line?;
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 2 {
                return Err(format!("malformed Fscacheinfo line: {}", line).into());
            }

            match fields[0] {
                "Cookies:" => {
                    set_fscache_fields(&fields[1..], &mut [
                        &mut info.index_cookies_allocated,
                        &mut info.data_storage_cookies_allocated,
                        &mut info.special_cookies_allocated,
                    ])?;
                }
                "Objects:" => {
                    set_fscache_fields(&fields[1..], &mut [
                        &mut info.objects_allocated,
                        &mut info.object_allocations_failure,
                        &mut info.objects_available,
                        &mut info.objects_dead,
                    ])?;
                }
                "ChkAux" => {
                    set_fscache_fields(&fields[2..], &mut [
                        &mut info.objects_without_coherency_check,
                        &mut info.objects_with_coherency_check,
                        &mut info.objects_need_coherency_check_update,
                        &mut info.objects_declared_obsolete,
                    ])?;
                }
                "Pages" => {
                    set_fscache_fields(&fields[2..], &mut [
                        &mut info.pages_marked_as_being_cached,
                        &mut info.uncache_pages_request_seen,
                    ])?;
                }
                "Acquire:" => {
                    set_fscache_fields(&fields[1..], &mut [
                        &mut info.acquire_cookies_request_seen,
                        &mut info.acquire_requests_with_null_parent,
                        &mut info.acquire_requests_rejected_no_cache_available,
                        &mut info.acquire_requests_succeeded,
                        &mut info.acquire_requests_rejected_due_to_error,
                        &mut info.acquire_requests_failed_due_to_enomem,
                    ])?;
                }
                "Lookups:" => {
                    set_fscache_fields(&fields[1..], &mut [
                        &mut info.lookups_number,
                        &mut info.lookups_negative,
                        &mut info.lookups_positive,
                        &mut info.objects_created_by_lookup,
                        &mut info.lookups_timed_out_and_requed,
                    ])?;
                }
                "Invals" => {
                    set_fscache_fields(&fields[2..], &mut [
                        &mut info.invalidations_number,
                        &mut info.invalidations_running,
                    ])?;
                }
                "Updates:" => {
                    set_fscache_fields(&fields[1..], &mut [
                        &mut info.update_cookie_request_seen,
                        &mut info.update_requests_with_null_parent,
                        &mut info.update_requests_running,
                    ])?;
                }
                "Relinqs:" => {
                    set_fscache_fields(&fields[1..], &mut [
                        &mut info.relinquish_cookies_request_seen,
                        &mut info.relinquish_cookies_with_null_parent,
                        &mut info.relinquish_requests_waiting_complete_creation,
                        &mut info.relinquish_retries,
                    ])?;
                }
                "AttrChg:" => {
                    set_fscache_fields(&fields[1..], &mut [
                        &mut info.attribute_changed_requests_seen,
                        &mut info.attribute_changed_requests_queued,
                        &mut info.attribute_changed_reject_due_to_enobufs,
                        &mut info.attribute_changed_failed_due_to_enomem,
                        &mut info.attribute_changed_ops,
                    ])?;
                }
                "Allocs" => {
                    if fields[2].starts_with("n=") {
                        set_fscache_fields(&fields[2..], &mut [
                            &mut info.allocation_requests_seen,
                            &mut info.allocation_ok_requests,
                            &mut info.allocation_waiting_on_lookup,
                            &mut info.allocations_rejected_due_to_enobufs,
                            &mut info.allocations_aborted_due_to_erestartsys,
                        ])?;
                    } else {
                        set_fscache_fields(&fields[2..], &mut [
                            &mut info.allocation_operations_submitted,
                            &mut info.allocations_waited_for_cpu,
                            &mut info.allocations_aborted_due_to_object_death,
                        ])?;
                    }
                }
                "Retrvls:" => {
                    if fields[1].starts_with("n=") {
                        set_fscache_fields(&fields[1..], &mut [
                            &mut info.retrievals_read_requests,
                            &mut info.retrievals_ok,
                            &mut info.retrievals_waiting_lookup_completion,
                            &mut info.retrievals_returned_enodata,
                            &mut info.retrievals_rejected_due_to_enobufs,
                            &mut info.retrievals_aborted_due_to_erestartsys,
                            &mut info.retrievals_failed_due_to_enomem,
                        ])?;
                    } else {
                        set_fscache_fields(&fields[1..], &mut [
                            &mut info.retrievals_requests,
                            &mut info.retrievals_waiting_cpu,
                            &mut info.retrievals_aborted_due_to_object_death,
                        ])?;
                    }
                }
                "Stores" => {
                    if fields[2].starts_with("n=") {
                        set_fscache_fields(&fields[2..], &mut [
                            &mut info.store_write_requests,
                            &mut info.store_successful_requests,
                            &mut info.store_requests_on_pending_storage,
                            &mut info.store_requests_rejected_due_to_enobufs,
                            &mut info.store_requests_failed_due_to_enomem,
                        ])?;
                    } else {
                        set_fscache_fields(&fields[2..], &mut [
                            &mut info.store_requests_submitted,
                            &mut info.store_requests_running,
                            &mut info.store_pages_with_requests_processing,
                            &mut info.store_requests_deleted,
                            &mut info.store_requests_over_store_limit,
                        ])?;
                    }
                }
                "VmScan" => {
                    set_fscache_fields(&fields[2..], &mut [
                        &mut info.release_requests_against_pages_with_no_pending_storage,
                        &mut info.release_requests_against_pages_stored_by_time_lock_granted,
                        &mut info.release_requests_ignored_due_to_in_progress_store,
                        &mut info.page_stores_cancelled_by_release_requests,
                        &mut info.vmscan_waiting,
                    ])?;
                }
                "Ops" => {
                    if fields[2].starts_with("pend=") {
                        set_fscache_fields(&fields[2..], &mut [
                            &mut info.ops_pending,
                            &mut info.ops_running,
                            &mut info.ops_enqueued,
                            &mut info.ops_cancelled,
                            &mut info.ops_rejected,
                        ])?;
                    } else {
                        set_fscache_fields(&fields[2..], &mut [
                            &mut info.ops_initialised,
                            &mut info.ops_deferred,
                            &mut info.ops_released,
                            &mut info.ops_garbage_collected,
                        ])?;
                    }
                }
                "CacheOp:" => {
                    if fields[1].starts_with("alo=") {
                        set_fscache_fields(&fields[1..], &mut [
                            &mut info.cacheop_allocations_in_progress,
                            &mut info.cacheop_lookup_object_in_progress,
                            &mut info.cacheop_lookup_complete_in_progress,
                            &mut info.cacheop_grab_object_in_progress,
                        ])?;
                    } else if fields[1].starts_with("inv=") {
                        set_fscache_fields(&fields[1..], &mut [
                            &mut info.cacheop_invalidations,
                            &mut info.cacheop_update_object_in_progress,
                            &mut info.cacheop_drop_object_in_progress,
                            &mut info.cacheop_put_object_in_progress,
                            &mut info.cacheop_attribute_change_in_progress,
                            &mut info.cacheop_sync_cache_in_progress,
                        ])?;
                    } else {
                        set_fscache_fields(&fields[1..], &mut [
                            &mut info.cacheop_read_or_alloc_page_in_progress,
                            &mut info.cacheop_read_or_alloc_pages_in_progress,
                            &mut info.cacheop_allocate_page_in_progress,
                            &mut info.cacheop_allocate_pages_in_progress,
                            &mut info.cacheop_write_pages_in_progress,
                            &mut info.cacheop_uncache_pages_in_progress,
                            &mut info.cacheop_dissociate_pages_in_progress,
                        ])?;
                    }
                }
                "CacheEv:" => {
                    set_fscache_fields(&fields[1..], &mut [
                        &mut info.cacheev_lookups_and_creations_rejected_lack_space,
                        &mut info.cacheev_stale_objects_deleted,
                        &mut info.cacheev_retired_when_relinquished,
                        &mut info.cacheev_objects_culled,
                    ])?;
                }
                _ => {}
            }
        }

        Ok(info)
    }
}

fn set_fscache_fields(fields: &[&str], set_fields: &mut [&mut u64]) -> Result<(), Box<dyn std::error::Error>> {
    if fields.len() < set_fields.len() {
        return Err(format!("Expected {}, but got {}", set_fields.len(), fields.len()).into());
    }

    for (i, field) in set_fields.iter_mut().enumerate() {
        **field = u64::from_str(fields[i].split('=').nth(1).ok_or("Invalid field format")?)?;
    }

    Ok(())
}

fn read_fscacheinfo(path: &Path) -> Result<Fscacheinfo, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Fscacheinfo::from_reader(reader)
}