pub const LLVM_GLOBAL_CTORS: &str = "llvm.global_ctors";

pub const COV_INIT: &str = "__cov_init";
pub const COV_INIT_ENTRY: &str = "__cov_init_entry";
pub const COV_MODULE_INIT: &str = "__cov_module_init";

pub const COV_HIT_BATCH: &str = "__cov_hit_batch";
pub const COV_MAPPING_SRC: &str = "__cov_mapping_src";

pub const COV_HIT_LINES_ARR: &str = "__cov_hit_lines_arr";
pub const COV_HIT_LINES_ARR_PTR: &str = "__cov_hit_lines_arr__ptr";

pub const COV_SRC_MAPPING_FUNC_LINES: &str = "__cov_src_mapping_funcs_lines";
pub const COV_SRC_MAPPING_FUNC_LINES_PTR: &str = "__cov_src_mapping_funcs_lines_ptr";
pub const COV_SRC_MAPPING_BRS_LINES: &str = "__cov_src_mapping_brs_lines";
pub const COV_SRC_MAPPING_BRS_LINES_PTR: &str = "__cov_src_mapping_brs_lines_ptr";
pub const COV_SRC_MAPPING_LINES_LINES: &str = "__cov_src_mapping_lines_lines";
pub const COV_SRC_MAPPING_LINES_LINES_PTR: &str = "__cov_src_mapping_lines_lines_ptr";

pub const ASAN_MEM_CHECK: &str = "__asan_mem_check";
pub const ASAN_MEM_INIT_REDZONE: &str = "__asan_init_redzone";

pub const FUZZER_MODULE_INIT: &str = "__fuzzer_module_init";
pub const FUZZER_INIT_ENTRY: &str = "__fuzzer_init_entry";
pub const FUZZER_FORKSERVER_INIT: &str = "__fuzzer_forkserver_init";
pub const FUZZER_TRACE_EDGE: &str = "__fuzzer_trace_edge";

pub const SYMBOLIC_MAKE_VAR: &str = "__make_symbolic";
pub const SYMBOLIC_MODULE_ADD_SYM: &str = "__symbolic_module_add_sym";
pub const SYMBOLIC_INIT_ENTRY: &str = "__symbolic_init_entry";
pub const SYMBOLIC_INIT: &str = "__symbolic_init";
pub const SYMBOLIC_MAKE_PREPARE: &str = "__symbolic_make_prepare";

pub const PTHREAD_MUTEX_LOCK: &str = "pthread_mutex_lock";
pub const PTHREAD_MUTEX_UNLOCK: &str = "pthread_mutex_unlock";
pub const PTHREAD_SELF: &str = "pthread_self";

pub const RACE_MODULE_INIT: &str = "__race_module_init";
pub const RACE_INIT_ENTRY: &str = "__race_init_entry";
pub const RACE_INIT_CANDIDATE_LOCKSET_GLOBAL_VAR: &str = "__race_init_candidate_lockset_global_var";
pub const RACE_INIT_CANDIDATE_LOCKSET_LOCK_VAR: &str = "__race_init_candidate_lockset_lock_var";
pub const RACE_LOCK_HELD: &str = "__race_update_lock_held";
pub const RACE_UPDATE_SHARED_MEM: &str = "__race_update_shared_mem";
pub const RACE_GLOBAL_PREFIX: &str = "__race.global.";
