use crate::internal::{init_forkserver_fd, init_shm, read_u128, read_u64, write_u128, write_u64};
use libc::{c_void, kill, read, waitpid, write, SIGKILL, WEXITSTATUS};
use memmap2::MmapMut;
use std::sync::mpsc;
use std::thread;

use std::time::Duration;
use std::{
    mem,
    sync::{Arc, Mutex},
};

const PREV_LOC_IDX: usize = 0; // 16 byte [0..15]
pub const NEW_COVERAGES: usize = PREV_LOC_IDX + 16; // 8 byte [16..23]
pub const VISIT_EDGES: usize = NEW_COVERAGES + 8; // 8 bytes [24..31]
pub const VISIT_MARK: usize = VISIT_EDGES + 8; // 8 bytes [32..39]
pub const VISIT_EDGES_INDICIES: usize = VISIT_MARK + 8; // n bytes [40..]
pub const VISIT_EDGES_INDEX_SIZE: usize = 8;
pub const PROCESS_EXIT_NORMAL: u64 = 1;

lazy_static::lazy_static! {
    pub static ref EDGE_COVERAGE: Arc<Mutex<EdgeCoverage>> = Arc::new(Mutex::new(EdgeCoverage::new()));
}

pub struct EdgeCoverage {
    shm: Option<MmapMut>,
    shm_size: Option<usize>,
    aux: Option<MmapMut>,
    fork_server_host: Option<i32>,
    fork_server_runtime: Option<i32>,
}

impl Default for EdgeCoverage {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeCoverage {
    pub fn new() -> Self {
        let (shm_mmap, shm_size) = init_shm("SHM_ID", "SHM_SIZE");
        let (shm_aux_mmap, _) = init_shm("SHM_AUX_ID", "SHM_AUX_SIZE");
        Self {
            shm: shm_mmap,
            shm_size,
            aux: shm_aux_mmap,
            fork_server_host: init_forkserver_fd("FORK_SERVER_HOST"),
            fork_server_runtime: init_forkserver_fd("FORK_SERVER_RUNTIME"),
        }
    }

    /// only used for test
    pub fn init(&mut self) {
        let new_edge_cov = Self::new();
        self.shm = new_edge_cov.shm;
        self.shm_size = new_edge_cov.shm_size;
        self.aux = new_edge_cov.aux;
        self.fork_server_host = new_edge_cov.fork_server_host;
        self.fork_server_runtime = new_edge_cov.fork_server_runtime;
    }

    fn notify_process_exit(&self, status: u64) {
        if let Some(fd) = self.fork_server_host {
            // do not send zero value
            let status = if status == 0 {
                PROCESS_EXIT_NORMAL
            } else {
                status
            };
            let bytes = status.to_ne_bytes();
            unsafe { write(fd, bytes.as_ptr() as *const _, bytes.len()) };
        }
    }

    pub fn read_wakeup(&self) {
        if let Some(fd) = self.fork_server_runtime {
            loop {
                let mut host_sent: u64 = 0;
                unsafe {
                    read(
                        fd,
                        &mut host_sent as *mut _ as *mut c_void,
                        mem::size_of::<u64>(),
                    );
                }
                if host_sent == 99999999999 {
                    // exit signal
                    unsafe {
                        libc::exit(0);
                    }
                }
                let pid = unsafe { libc::fork() };
                if pid == 0 {
                    // child process
                    return; // run target program's main logic
                } else {
                    // parent process
                    let (tx, rx) = mpsc::channel();
                    thread::spawn(
                        move || match rx.recv_timeout(Duration::from_secs(host_sent)) {
                            Ok(_) => {}
                            Err(mpsc::RecvTimeoutError::Timeout) => unsafe {
                                kill(pid, SIGKILL);
                            },
                            _ => {}
                        },
                    );
                    let mut status: i32 = 0;
                    unsafe {
                        waitpid(pid, &mut status, 0);
                        tx.send(()).ok();
                    }
                    status = if status != SIGKILL {
                        WEXITSTATUS(status)
                    } else {
                        status
                    };
                    self.notify_process_exit(status.try_into().unwrap());
                }
            }
        }
    }

    pub fn trace_edge(&mut self, cur_loc: i64) {
        if let (Some(ref mut shm), Some(shm_size), Some(ref mut shm_aux)) =
            (&mut self.shm, self.shm_size, &mut self.aux)
        {
            let cur_loc = cur_loc as u128;
            let prev_loc = read_u128(shm_aux, PREV_LOC_IDX);
            let edge = (cur_loc ^ prev_loc) as usize % shm_size;
            write_u128(shm_aux, PREV_LOC_IDX, cur_loc >> 1);

            let visited_cnt = read_u64(shm_aux, VISIT_MARK) as usize;
            let already_visited = (0..visited_cnt).any(|i| {
                let stored_edge =
                    read_u64(shm_aux, VISIT_EDGES_INDICIES + (i * VISIT_EDGES_INDEX_SIZE));
                stored_edge as usize == edge
            });
            if !already_visited {
                write_u64(
                    shm_aux,
                    VISIT_EDGES_INDICIES + (visited_cnt * VISIT_EDGES_INDEX_SIZE),
                    edge,
                );
                write_u64(shm_aux, VISIT_MARK, visited_cnt + 1);
            }
            if shm[edge] == 0 {
                write_u64(shm_aux, NEW_COVERAGES, 1);
                let edges = read_u64(shm_aux, VISIT_EDGES) as usize;
                write_u64(shm_aux, VISIT_EDGES, edges + 1);
            }
            shm[edge] = shm[edge].saturating_add(1);
        }
    }
}
