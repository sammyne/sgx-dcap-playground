# Example of DCAP-v1.3.101.3 

## environment 
- ubuntu 16.04
- SGX v2.7
- DCAP v1.3.101.3

## build 
```bash
mkdir build 
cd build 
cmake -DCMAKE_BUILD_TYPE=Prerelease ..

make 
```

## run
```bash
# in the build directory 
make run
```

## Problem
- `sgx_qv_verify_quote` returns `SGX_QL_TCBINFO_UNSUPPORTED_FORMAT`
    - the error code can be one of 
      - `STATUS_UNSUPPORTED_TCB_INFO_FORMAT`
      - `STATUS_TCB_NOT_SUPPORTED`
      - `STATUS_SGX_TCB_INFO_UNSUPPORTED_FORMAT`
      - `STATUS_SGX_TCB_INFO_INVALID`

    but I can't figure out exactly which one.