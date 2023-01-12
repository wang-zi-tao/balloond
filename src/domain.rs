use crate::cli::Opt;
use failure::{format_err, Fallible};
use libc::c_void;
use log::{debug, info};
use std::collections::{HashMap, VecDeque};
use sysinfo::{System, SystemExt};
use virt::domain::{sys, Domain, VIR_DOMAIN_SHUTDOWN, VIR_DOMAIN_SHUTOFF};

#[derive(Debug)]
struct DomainMemory {
    memory: u64,
}
#[derive(Default)]
pub(crate) struct DomainDataBase {
    pub(crate) records: HashMap<String, DomainMemoryRecord>,
}

#[link(name = "virt")]
extern "C" {
    fn virDomainMemoryStats(
        ptr: *mut c_void,
        stats: sys::virDomainMemoryStatsPtr,
        nr_stats: libc::c_uint,
        flags: libc::c_uint,
    ) -> libc::c_int;
}
#[derive(Default)]
pub(crate) struct DomainMemoryRecord {
    records: VecDeque<DomainMemory>,
    memory: u64,
}
impl DomainMemoryRecord {
    pub(crate) fn process_domain(
        &mut self,
        domain: Domain,
        system: &mut System,
        domain_count: usize,
        opt: &Opt,
    ) -> Fallible<()> {
        let name = domain.get_name()?;
        let state = domain.get_state()?.0;
        match state {
            VIR_DOMAIN_SHUTOFF | VIR_DOMAIN_SHUTDOWN => {
                self.records.clear();
                return Ok(());
            }
            _ => {}
        }
        let current_memory = domain.get_info()?.memory as i64;
        let max = domain.get_max_memory()?;
        domain.set_memory_stats_period(2, 0)?;
        let mut memoey_stats = [0; 13];
        unsafe {
            let mut pinfo: [sys::virDomainMemoryStats; 16] = Default::default();
            let ret =
                virDomainMemoryStats(domain.as_ptr().cast(), &mut pinfo[0], pinfo.len() as u32, 0);
            if ret == -1 {
                return Err(format_err!("virDomainMemoryStats failed"));
            }
            for i in &pinfo {
                if (i.tag as usize) < memoey_stats.len() {
                    memoey_stats[i.tag as usize] = i.val;
                }
            }
        }
        let usable = memoey_stats[8] as i64;
        debug!("guest available memory: {}", usable);
        let host_usable_memory = (system.get_total_memory() - system.get_used_memory()) as i64;
        debug!("host available memory: {}", host_usable_memory);
        let physical_memory_size = (i64::max(
            i64::min(
                current_memory + host_usable_memory - opt.host_reserved as i64 * 1024,
                current_memory
                    + (((host_usable_memory + usable) as f32
                        * (opt.reserved_percent / domain_count as f32))
                        as i64
                        - usable),
            ),
            current_memory - usable + opt.guest_reserved as i64 * 1024,
        )) as u64;
        let align = opt.align * 1024;
        let physical_memory_size_aligned = u64::min(
            max,
            physical_memory_size - physical_memory_size % align + align,
        );
        debug!(
            "expected guest available memory: {}",
            physical_memory_size_aligned - (current_memory - usable) as u64
        );
        if self.records.len() >= opt.history_count {
            self.records.pop_front();
        }
        self.records.push_back(DomainMemory {
            memory: physical_memory_size_aligned,
        });
        let memory = self
            .records
            .iter()
            .map(|r| r.memory)
            .max()
            .unwrap_or(physical_memory_size_aligned);
        if memory != self.memory {
            domain.set_memory(memory)?;
            info!(
                "virtual machine: {name} , change memory: {inc:+}MiB , memory: {mem}MiB ",
                name = name,
                mem = memory / 1024,
                inc = ((memory as i64) - self.memory as i64) / 1024
            );
            self.memory = memory;
        }
        Ok(())
    }
}
