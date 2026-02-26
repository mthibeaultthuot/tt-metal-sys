#include "wrapper.hpp"

#include <tt-metalium/host_api.hpp>
#include <tt-metalium/tt_metal.hpp>
#include <tt-metalium/program.hpp>
#include <tt-metalium/buffer.hpp>
#include <tt-metalium/kernel_types.hpp>
#include <tt-metalium/circular_buffer_config.hpp>
#include <tt-metalium/core_coord.hpp>
#include <tt-metalium/data_types.hpp>
#include <tt-metalium/buffer_types.hpp>
#include <tt-metalium/tt_backend_api_types.hpp>
#include <cstdlib>

#include <cstring>
#include <memory>
#include <string>
#include <map>
#include <vector>
#include <variant>

using namespace tt::tt_metal;

struct tt_device {
    IDevice* ptr;
};

struct tt_program {
    Program inner;
};

struct tt_buffer {
    std::shared_ptr<Buffer> inner;
};

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

static void result_err_exception(tt_result* r, const std::exception& e) {
    result_err(r, e.what());
}

static CoreCoord to_core_coord(tt_core_coord c) {
    return CoreCoord(c.x, c.y);
}

static CoreRange to_core_range(tt_core_range r) {
    return CoreRange(to_core_coord(r.start), to_core_coord(r.end));
}

static tt::DataFormat to_data_format(tt_data_format df) {
    return static_cast<tt::DataFormat>(df);
}

static MathFidelity to_math_fidelity(tt_math_fidelity mf) {
    return static_cast<MathFidelity>(mf);
}

static DataMovementProcessor to_dm_processor(tt_data_movement_processor p) {
    return static_cast<DataMovementProcessor>(p);
}

static NOC to_noc(tt_noc n) {
    return static_cast<NOC>(n);
}

static NOC_MODE to_noc_mode(tt_noc_mode m) {
    return static_cast<NOC_MODE>(m);
}

static BufferType to_buffer_type(tt_buffer_type bt) {
    switch (bt) {
        case TT_BUFFER_TYPE_DRAM: return BufferType::DRAM;
        case TT_BUFFER_TYPE_L1: return BufferType::L1;
        case TT_BUFFER_TYPE_SYSTEM_MEMORY: return BufferType::SYSTEM_MEMORY;
        case TT_BUFFER_TYPE_L1_SMALL: return BufferType::L1_SMALL;
        case TT_BUFFER_TYPE_TRACE: return BufferType::TRACE;
        default: return BufferType::DRAM;
    }
}

static std::variant<DataMovementConfig, ComputeConfig, EthernetConfig>
build_kernel_config(const tt_kernel_config* cfg) {
    std::vector<uint32_t> compile_args;
    if (cfg->compile_args && cfg->compile_args_len > 0) {
        compile_args.assign(cfg->compile_args, cfg->compile_args + cfg->compile_args_len);
    }

    std::map<std::string, std::string> defines;
    if (cfg->define_keys && cfg->define_values && cfg->defines_len > 0) {
        for (size_t i = 0; i < cfg->defines_len; i++) {
            defines[cfg->define_keys[i]] = cfg->define_values[i];
        }
    }

    switch (cfg->kernel_type) {
        case TT_KERNEL_TYPE_COMPUTE: {
            ComputeConfig cc;
            cc.math_fidelity = to_math_fidelity(cfg->math_fidelity);
            cc.fp32_dest_acc_en = cfg->fp32_dest_acc_en != 0;
            cc.math_approx_mode = cfg->math_approx_mode != 0;
            cc.compile_args = std::move(compile_args);
            cc.defines = std::move(defines);
            return cc;
        }
        case TT_KERNEL_TYPE_ETHERNET: {
            EthernetConfig ec;
            ec.noc = to_noc(cfg->noc);
            ec.compile_args = std::move(compile_args);
            ec.defines = std::move(defines);
            return ec;
        }
        case TT_KERNEL_TYPE_DATA_MOVEMENT:
        default: {
            DataMovementConfig dmc;
            dmc.processor = to_dm_processor(cfg->processor);
            dmc.noc = to_noc(cfg->noc);
            dmc.noc_mode = to_noc_mode(cfg->noc_mode);
            dmc.compile_args = std::move(compile_args);
            dmc.defines = std::move(defines);
            return dmc;
        }
    }
}

extern "C" {

size_t ttrs_get_num_available_devices(void) {
    return GetNumAvailableDevices();
}

size_t ttrs_get_num_pcie_devices(void) {
    return GetNumPCIeDevices();
}

int ttrs_is_mock_mode(void) {
    // Check if TT_METAL_MOCK_CLUSTER_DESC_PATH env var is set
    const char* mock_path = std::getenv("TT_METAL_MOCK_CLUSTER_DESC_PATH");
    return (mock_path != nullptr && mock_path[0] != '\0') ? 1 : 0;
}

int ttrs_is_simulator_mode(void) {
    // Check if TT_METAL_SIMULATOR env var is set
    const char* sim_path = std::getenv("TT_METAL_SIMULATOR");
    return (sim_path != nullptr && sim_path[0] != '\0') ? 1 : 0;
}

tt_device* ttrs_device_create(int device_id, uint8_t num_hw_cqs, tt_result* result) {
    try {
        IDevice* dev = CreateDevice(device_id, num_hw_cqs);
        auto* wrapper = new tt_device{dev};
        result_ok(result);
        return wrapper;
    } catch (const std::exception& e) {
        result_err_exception(result, e);
        return nullptr;
    }
}

void ttrs_device_close(tt_device* device) {
    if (device) {
        CloseDevice(device->ptr);
        delete device;
    }
}

tt_program* ttrs_program_create(void) {
    auto* p = new tt_program{CreateProgram()};
    return p;
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
    try {
        auto cr = to_core_range(core_range);
        auto kcfg = build_kernel_config(config);
        auto handle = CreateKernelFromString(
            program->inner,
            std::string(kernel_source),
            cr,
            kcfg);
        result_ok(result);
        return handle;
    } catch (const std::exception& e) {
        result_err_exception(result, e);
        return 0;
    }
}

tt_kernel_handle ttrs_create_kernel(
    tt_program* program,
    const char* file_path,
    tt_core_range core_range,
    const tt_kernel_config* config,
    tt_result* result)
{
    try {
        auto cr = to_core_range(core_range);
        auto kcfg = build_kernel_config(config);
        auto handle = CreateKernel(
            program->inner,
            std::string(file_path),
            cr,
            kcfg);
        result_ok(result);
        return handle;
    } catch (const std::exception& e) {
        result_err_exception(result, e);
        return 0;
    }
}

void ttrs_set_runtime_args(
    tt_program* program,
    tt_kernel_handle kernel,
    tt_core_coord core,
    const uint32_t* args,
    size_t args_len,
    tt_result* result)
{
    try {
        auto cc = to_core_coord(core);
        tt::stl::Span<const uint32_t> span(args, args_len);
        SetRuntimeArgs(program->inner, kernel, cc, span);
        result_ok(result);
    } catch (const std::exception& e) {
        result_err_exception(result, e);
    }
}

void ttrs_set_runtime_args_range(
    tt_program* program,
    tt_kernel_handle kernel,
    tt_core_range range,
    const uint32_t* args,
    size_t args_len,
    tt_result* result)
{
    try {
        auto cr = to_core_range(range);
        tt::stl::Span<const uint32_t> span(args, args_len);
        SetRuntimeArgs(program->inner, kernel, cr, span);
        result_ok(result);
    } catch (const std::exception& e) {
        result_err_exception(result, e);
    }
}

void ttrs_set_common_runtime_args(
    tt_program* program,
    tt_kernel_handle kernel,
    const uint32_t* args,
    size_t args_len,
    tt_result* result)
{
    try {
        tt::stl::Span<const uint32_t> span(args, args_len);
        SetCommonRuntimeArgs(program->inner, kernel, span);
        result_ok(result);
    } catch (const std::exception& e) {
        result_err_exception(result, e);
    }
}

tt_cb_handle ttrs_create_circular_buffer(
    tt_program* program,
    tt_core_range core_range,
    const tt_cb_config* config,
    tt_result* result)
{
    try {
        std::map<uint8_t, tt::DataFormat> data_format_spec;
        for (size_t i = 0; i < config->data_format_spec_len; i++) {
            data_format_spec[config->data_format_spec[i].buffer_index] =
                to_data_format(config->data_format_spec[i].data_format);
        }

        CircularBufferConfig cb_config(config->total_size, data_format_spec);
        for (size_t i = 0; i < config->data_format_spec_len; i++) {
            cb_config.set_page_size(config->data_format_spec[i].buffer_index, config->data_format_spec[i].page_size);
        }
        auto cr = to_core_range(core_range);
        auto handle = CreateCircularBuffer(program->inner, cr, cb_config);
        result_ok(result);
        return handle;
    } catch (const std::exception& e) {
        result_err_exception(result, e);
        return 0;
    }
}

tt_buffer* ttrs_buffer_create_interleaved(
    tt_device* device,
    uint64_t size,
    uint64_t page_size,
    tt_buffer_type buffer_type,
    tt_result* result)
{
    try {
        InterleavedBufferConfig cfg{
            .device = device->ptr,
            .size = size,
            .page_size = page_size,
            .buffer_type = to_buffer_type(buffer_type),
        };
        auto buf = CreateBuffer(cfg);
        auto* wrapper = new tt_buffer{std::move(buf)};
        result_ok(result);
        return wrapper;
    } catch (const std::exception& e) {
        result_err_exception(result, e);
        return nullptr;
    }
}

void ttrs_buffer_destroy(tt_buffer* buffer) {
    if (buffer) {
        delete buffer;
    }
}

uint32_t ttrs_buffer_address(const tt_buffer* buffer) {
    return buffer->inner->address();
}

uint64_t ttrs_buffer_size(const tt_buffer* buffer) {
    return buffer->inner->size();
}

void ttrs_buffer_write(
    tt_buffer* buffer,
    const uint8_t* data,
    size_t data_len,
    tt_result* result)
{
    try {
        tt::stl::Span<const uint8_t> span(data, data_len);
        detail::WriteToBuffer(*buffer->inner, span);
        result_ok(result);
    } catch (const std::exception& e) {
        result_err_exception(result, e);
    }
}

void ttrs_buffer_read(
    tt_buffer* buffer,
    uint8_t* data_out,
    size_t data_len,
    tt_result* result)
{
    try {
        detail::ReadFromBuffer(*buffer->inner, data_out);
        result_ok(result);
    } catch (const std::exception& e) {
        result_err_exception(result, e);
    }
}

void ttrs_compile_program(tt_device* device, tt_program* program, tt_result* result) {
    try {
        detail::CompileProgram(device->ptr, program->inner);
        result_ok(result);
    } catch (const std::exception& e) {
        result_err_exception(result, e);
    }
}

void ttrs_launch_program(tt_device* device, tt_program* program, int wait_for_completion, tt_result* result) {
    try {
        detail::LaunchProgram(device->ptr, program->inner, wait_for_completion != 0, true);
        result_ok(result);
    } catch (const std::exception& e) {
        result_err_exception(result, e);
    }
}

void ttrs_wait_program_done(tt_device* device, tt_program* program, tt_result* result) {
    try {
        detail::WaitProgramDone(device->ptr, program->inner);
        result_ok(result);
    } catch (const std::exception& e) {
        result_err_exception(result, e);
    }
}

}
