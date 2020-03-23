#![no_std]

extern crate sgx_types;
#[macro_use]
extern crate sgx_tstd as std;

use std::prelude::v1::*;

//use sgx_tse::*;
use sgx_types::*;

#[no_mangle]
pub extern "C" fn ecall_new_report(
    qe3_target: *const sgx_target_info_t,
    report: *mut sgx_report_t,
) -> sgx_status_t {
    let data: sgx_report_data_t = sgx_report_data_t::default();

    unsafe { sgx_create_report(qe3_target, &data, report) }
}
