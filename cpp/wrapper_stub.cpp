#include "wrapper.hpp"
#include <cstring>
#include <cstdlib>
#include <string>
#include <vector>
#include <map>

static void result_ok(tt_result* r) {
    if (r) {
        r->ok = 1;
        r->message[0] = '\0';
    }
}

static void result_err(tt_result* r, const char* msg) {
    if (r) {
        r->ok = 0;
        std::strncpy(r->message, msg, sizeof(r->message) - 1);
        r->message[sizeof(r->message) - 1] = '\0';
    }
}

struct tt_device {
    int id;
};

struct tt_program {
    std::vector<std::string> kernels;
};

struct tt_buffer {
    std::vector<uint8_t> data;
    uint32_t addr;
    static uint32_t next_addr;
};

uint32_t tt_buffer::next_addr = 0x10000000;

extern "C" {

size_t ttrs_get_num_available_devices(void) {
    return 0;
}

size_t ttrs_get_num_pcie_devices(void) {
    return 0;
}

int ttrs_is_mock_mode(void) {
    return 1;
}

int ttrs_is_simulator_mode(void) {
    return 0;
}

tt_device* ttrs_device_create(int device_id, uint8_t num_hw_cqs, tt_result* result) {
    auto* dev = new tt_device{device_id};
    result_ok(result);
    return dev;
}

void ttrs_device_close(tt_device* device) {
    delete device;
}

tt_program* ttrs_program_create(void) {
    return new tt_program();
}

void ttrs_program_destroy(tt_program* program) {
    delete program;
}

tt_kernel_handle ttrs_create_kernel_from_string(
    tt_program* program,
    const char* kernel_source,
    tt_core_range core_range,
    const tt_kernel_config* config,
    tt_result* result)
{
    program->kernels.push_back(std::string(kernel_source));
    result_ok(result);
    return static_cast<tt_kernel_handle>(program->kernels.size());
}

tt_kernel_handle ttrs_create_kernel(
    tt_program* program,
    const char* file_path,
    tt_core_range core_range,
    const tt_kernel_config* config,
    tt_result* result)
{
    program->kernels.push_back(std::string(file_path));
    result_ok(result);
    return static_cast<tt_kernel_handle>(program->kernels.size());
}

void ttrs_set_runtime_args(
    tt_program* program,
    tt_kernel_handle kernel,
    tt_core_coord core,
    const uint32_t* args,
    size_t args_len,
    tt_result* result)
{
    result_ok(result);
}

void ttrs_set_runtime_args_range(
    tt_program* program,
    tt_kernel_handle kernel,
    tt_core_range range,
    const uint32_t* args,
    size_t args_len,
    tt_result* result)
{
    result_ok(result);
}

void ttrs_set_common_runtime_args(
    tt_program* program,
    tt_kernel_handle kernel,
    const uint32_t* args,
    size_t args_len,
    tt_result* result)
{
    result_ok(result);
}

tt_cb_handle ttrs_create_circular_buffer(
    tt_program* program,
    tt_core_range core_range,
    const tt_cb_config* config,
    tt_result* result)
{
    result_ok(result);
    return 1;
}

tt_buffer* ttrs_buffer_create_interleaved(
    tt_device* device,
    uint64_t size,
    uint64_t page_size,
    tt_buffer_type buffer_type,
    tt_result* result)
{
    auto* buf = new tt_buffer();
    buf->data.resize(size);
    buf->addr = tt_buffer::next_addr;
    tt_buffer::next_addr += size;
    result_ok(result);
    return buf;
}

void ttrs_buffer_destroy(tt_buffer* buffer) {
    delete buffer;
}

uint32_t ttrs_buffer_address(const tt_buffer* buffer) {
    return buffer->addr;
}

uint64_t ttrs_buffer_size(const tt_buffer* buffer) {
    return buffer->data.size();
}

void ttrs_buffer_write(
    tt_buffer* buffer,
    const uint8_t* data,
    size_t data_len,
    tt_result* result)
{
    size_t to_copy = std::min(data_len, buffer->data.size());
    std::memcpy(buffer->data.data(), data, to_copy);
    result_ok(result);
}

void ttrs_buffer_read(
    tt_buffer* buffer,
    uint8_t* data_out,
    size_t data_len,
    tt_result* result)
{
    size_t to_copy = std::min(data_len, buffer->data.size());
    std::memcpy(data_out, buffer->data.data(), to_copy);
    result_ok(result);
}

void ttrs_compile_program(tt_device* device, tt_program* program, tt_result* result) {
    result_ok(result);
}

void ttrs_launch_program(tt_device* device, tt_program* program, int wait_for_completion, tt_result* result) {
    result_ok(result);
}

void ttrs_wait_program_done(tt_device* device, tt_program* program, tt_result* result) {
    result_ok(result);
}

}
