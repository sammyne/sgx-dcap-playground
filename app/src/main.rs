extern crate sgx_types;
extern crate sgx_urts;

use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::{mem, ptr};

extern "C" {
    fn ecall_new_report(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        qe3_target: *const sgx_target_info_t,
        report: *mut sgx_report_t,
    ) -> sgx_status_t;
}

fn panic_if_not_success(status: sgx_status_t, tip: &str) {
    match status {
        sgx_status_t::SGX_SUCCESS => {}
        _ => panic!(format!("[-] {} {}!", tip, status.as_str())),
    }
}

fn init_enclave(enclave_path: &str) -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // [DEPRECATED since v2.6] Step 1: try to retrieve the launch token saved by last transaction
    // if there is no token, then create a new one.
    //

    // Step 2: call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    const DEBUG: i32 = 1;
    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };
    let enclave = SgxEnclave::create(
        enclave_path,
        DEBUG,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    )?;

    // [DEPRECATED since v2.6] Step 3: save the launch token if it is updated

    Ok(enclave)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("missing enclave path");
        std::process::exit(-1);
    }

    let enclave = match init_enclave(&args[1]) {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        }
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        }
    };

    let mut qe3_target = sgx_target_info_t::default();
    let err = unsafe { sgx_qe_get_target_info(&mut qe3_target) };
    if err != sgx_quote3_error_t::SGX_QL_SUCCESS {
        panic!("[-] ocall_sgx_qe_get_target_info failed: {:?}", err);
    }

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let mut report = sgx_report_t::default();
    let result =
        unsafe { ecall_new_report(enclave.geteid(), &mut retval, &qe3_target, &mut report) };
    panic_if_not_success(result, "ecall fail result");
    panic_if_not_success(retval, "ecall fail retval");

    //println!("report: {:?}", report);

    let quote_size = {
        let mut quote_size = 0u32;
        let err = unsafe { sgx_qe_get_quote_size(&mut quote_size) };
        if err != sgx_quote3_error_t::SGX_QL_SUCCESS {
            panic!("[-] sgx_qe_get_quote_size failed: {:?}", err);
        }

        quote_size
    };

    let quote = {
        let mut quote = vec![0u8; quote_size as usize];
        let err = unsafe { sgx_qe_get_quote(&report, quote_size, quote.as_mut_ptr()) };
        if err != sgx_quote3_error_t::SGX_QL_SUCCESS {
            panic!("[-] sgx_qe_get_quote failed: {:?}", err);
        }

        quote
    };

    let mut supplemental_data_size = 0u32;
    let mut supplemental_data = sgx_ql_qv_supplemental_t::default();

    // 2021/01/01 00:00:00 UTC as unix seconds 1609459200
    // @TODO: read from config
    let expiration_check_date = 1609459200i64;

    // call DCAP quote verify library for quote verification
    let mut collateral_expiration_status = 1u32;
    let mut quote_verification_result = sgx_ql_qv_result_t::SGX_QL_QV_RESULT_UNSPECIFIED;

    let qve_err = unsafe { sgx_qv_get_quote_supplemental_data_size(&mut supplemental_data_size) };
    if qve_err == sgx_quote3_error_t::SGX_QL_SUCCESS
        && supplemental_data_size == (mem::size_of::<sgx_ql_qv_supplemental_t>() as u32)
    {
        println!("[app+] sgx_qv_get_quote_supplemental_data_size ok");
    } else {
        supplemental_data_size = 0;
        println!("[app-] sgx_qv_get_quote_supplemental_data_size failed");
    }

    let qve_err = unsafe {
        sgx_qv_verify_quote(
            quote.as_ptr(),
            quote.len() as u32,
            ptr::null(),
            expiration_check_date,
            &mut collateral_expiration_status,
            &mut quote_verification_result,
            ptr::null_mut(),
            supplemental_data_size,
            &mut supplemental_data as *mut _ as *mut u8,
        )
    };

    if qve_err != sgx_quote3_error_t::SGX_QL_SUCCESS {
        panic!(
            "[app-] sgx_qv_verify_quote failed: got {:?}-{}, expect {:?}-{}",
            qve_err,
            qve_err as u32,
            sgx_quote3_error_t::SGX_QL_SUCCESS,
            sgx_quote3_error_t::SGX_QL_SUCCESS as u32
        );
    }

    match quote_verification_result {
        sgx_ql_qv_result_t::SGX_QL_QV_RESULT_OK => {
            println!("[app+] Verification completed successfully.")
        }
        sgx_ql_qv_result_t::SGX_QL_QV_RESULT_CONFIG_NEEDED
        | sgx_ql_qv_result_t::SGX_QL_QV_RESULT_OUT_OF_DATE
        | sgx_ql_qv_result_t::SGX_QL_QV_RESULT_OUT_OF_DATE_CONFIG_NEEDED => panic!(
            "[app+] Verification completed with Non-terminal result: {:?}",
            quote_verification_result,
        ),
        _ => panic!("{:?}", quote_verification_result),
    };

    enclave.destroy();
}
