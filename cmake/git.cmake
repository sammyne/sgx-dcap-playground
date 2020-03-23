cmake_minimum_required(VERSION 3.10)

ExternalProject_Add(teaclave-sgx-sdk
    GIT_REPOSITORY https://github.com/apache/incubator-teaclave-sgx-sdk
    GIT_TAG d6b0fd0d4a5f28d652612c98de3755e109929cbc
    GIT_PROGRESS true
    SOURCE_DIR ${PROJECT_SOURCE_DIR}/vendor/teaclave-sgx-sdk
    UPDATE_DISCONNECTED true
    CONFIGURE_COMMAND echo "skip configure for teaclave-sgx-sdk"
    BUILD_COMMAND echo "skip build for teaclave-sgx-sdk"
    INSTALL_COMMAND echo "skip install for teaclave-sgx-sdk"
)