#pragma once

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct tt_device tt_device;
typedef struct tt_program tt_program;
typedef struct tt_buffer tt_buffer;

typedef uint32_t tt_kernel_handle;
typedef uintptr_t tt_cb_handle;

typedef struct {
    int ok;
    char message[512];
} tt_result;

typedef enum {
    TT_BUFFER_TYPE_DRAM = 0,
    TT_BUFFER_TYPE_L1 = 1,
    TT_BUFFER_TYPE_SYSTEM_MEMORY = 2,
    TT_BUFFER_TYPE_L1_SMALL = 3,
    TT_BUFFER_TYPE_TRACE = 4,
} tt_buffer_type;

typedef enum {
    TT_DATA_FORMAT_FLOAT32 = 0,
    TT_DATA_FORMAT_FLOAT16 = 1,
    TT_DATA_FORMAT_BFP8 = 2,
    TT_DATA_FORMAT_BFP4 = 3,
    TT_DATA_FORMAT_FLOAT16_B = 5,
    TT_DATA_FORMAT_BFP8_B = 6,
    TT_DATA_FORMAT_BFP4_B = 7,
    TT_DATA_FORMAT_TF32 = 4,
    TT_DATA_FORMAT_UINT16 = 9,
    TT_DATA_FORMAT_INT32 = 8,
    TT_DATA_FORMAT_INT8 = 14,
    TT_DATA_FORMAT_BFP2_B = 15,
    TT_DATA_FORMAT_UINT8 = 30,
    TT_DATA_FORMAT_UINT32 = 24,
    TT_DATA_FORMAT_LF8 = 10,
    TT_DATA_FORMAT_BFP2 = 11,
    TT_DATA_FORMAT_FP8_E4M3 = 0x1A,
    TT_DATA_FORMAT_RAW_UINT8 = 0xf0,
    TT_DATA_FORMAT_RAW_UINT16 = 0xf1,
    TT_DATA_FORMAT_RAW_UINT32 = 0xf2,
} tt_data_format;

typedef enum {
    TT_MATH_FIDELITY_LOFI = 0,
    TT_MATH_FIDELITY_HIFI2 = 2,
    TT_MATH_FIDELITY_HIFI3 = 3,
    TT_MATH_FIDELITY_HIFI4 = 4,
} tt_math_fidelity;

typedef enum {
    TT_DM_PROCESSOR_RISCV_0 = 0,
    TT_DM_PROCESSOR_RISCV_1 = 1,
} tt_data_movement_processor;

typedef enum {
    TT_NOC_0 = 0,
    TT_NOC_1 = 1,
} tt_noc;

typedef enum {
    TT_NOC_MODE_DM_DEDICATED = 0,
    TT_NOC_MODE_DM_DYNAMIC = 1,
} tt_noc_mode;

typedef enum {
    TT_KERNEL_TYPE_DATA_MOVEMENT = 0,
    TT_KERNEL_TYPE_COMPUTE = 1,
    TT_KERNEL_TYPE_ETHERNET = 2,
} tt_kernel_type;

typedef struct {
    size_t x;
    size_t y;
} tt_core_coord;

typedef struct {
    tt_core_coord start;
    tt_core_coord end;
} tt_core_range;

typedef struct {
    tt_kernel_type kernel_type;

    tt_data_movement_processor processor;
    tt_noc noc;
    tt_noc_mode noc_mode;

    tt_math_fidelity math_fidelity;
    int fp32_dest_acc_en;
    int math_approx_mode;

    const uint32_t* compile_args;
    size_t compile_args_len;

    const char* const* define_keys;
    const char* const* define_values;
    size_t defines_len;
} tt_kernel_config;

typedef struct {
    uint8_t buffer_index;
    tt_data_format data_format;
    uint32_t page_size;
} tt_cb_data_format_entry;

typedef struct {
    uint32_t total_size;
    const tt_cb_data_format_entry* data_format_spec;
    size_t data_format_spec_len;
} tt_cb_config;

size_t ttrs_get_num_available_devices(void);
size_t ttrs_get_num_pcie_devices(void);
int ttrs_is_mock_mode(void);
int ttrs_is_simulator_mode(void);

tt_device* ttrs_device_create(int device_id, uint8_t num_hw_cqs, tt_result* result);
void ttrs_device_close(tt_device* device);

tt_program* ttrs_program_create(void);
void ttrs_program_destroy(tt_program* program);

tt_kernel_handle ttrs_create_kernel_from_string(
    tt_program* program,
    const char* kernel_source,
    tt_core_range core_range,
    const tt_kernel_config* config,
    tt_result* result);

tt_kernel_handle ttrs_create_kernel(
    tt_program* program,
    const char* file_path,
    tt_core_range core_range,
    const tt_kernel_config* config,
    tt_result* result);

void ttrs_set_runtime_args(
    tt_program* program,
    tt_kernel_handle kernel,
    tt_core_coord core,
    const uint32_t* args,
    size_t args_len,
    tt_result* result);

void ttrs_set_runtime_args_range(
    tt_program* program,
    tt_kernel_handle kernel,
    tt_core_range range,
    const uint32_t* args,
    size_t args_len,
    tt_result* result);

void ttrs_set_common_runtime_args(
    tt_program* program,
    tt_kernel_handle kernel,
    const uint32_t* args,
    size_t args_len,
    tt_result* result);

tt_cb_handle ttrs_create_circular_buffer(
    tt_program* program,
    tt_core_range core_range,
    const tt_cb_config* config,
    tt_result* result);

tt_buffer* ttrs_buffer_create_interleaved(
    tt_device* device,
    uint64_t size,
    uint64_t page_size,
    tt_buffer_type buffer_type,
    tt_result* result);

void ttrs_buffer_destroy(tt_buffer* buffer);

uint32_t ttrs_buffer_address(const tt_buffer* buffer);
uint64_t ttrs_buffer_size(const tt_buffer* buffer);

void ttrs_buffer_write(
    tt_buffer* buffer,
    const uint8_t* data,
    size_t data_len,
    tt_result* result);

void ttrs_buffer_read(
    tt_buffer* buffer,
    uint8_t* data_out,
    size_t data_len,
    tt_result* result);

void ttrs_compile_program(tt_device* device, tt_program* program, tt_result* result);
void ttrs_launch_program(tt_device* device, tt_program* program, int wait_for_completion, tt_result* result);
void ttrs_wait_program_done(tt_device* device, tt_program* program, tt_result* result);

#ifdef __cplusplus
}
#endif
