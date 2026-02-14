use std::os::raw::c_char;

#[repr(C)]
pub struct TtResult {
    pub ok: i32,
    pub message: [c_char; 512],
}

impl TtResult {
    pub fn new() -> Self {
        Self {
            ok: 0,
            message: [0; 512],
        }
    }

    pub fn is_ok(&self) -> bool {
        self.ok == 1
    }

    pub fn error_message(&self) -> String {
        if self.is_ok() {
            return String::new();
        }
        let bytes: Vec<u8> = self.message.iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8)
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

#[repr(C)]
pub struct TtDevice {
    _private: [u8; 0],
}

#[repr(C)]
pub struct TtProgram {
    _private: [u8; 0],
}

#[repr(C)]
pub struct TtBuffer {
    _private: [u8; 0],
}

pub type TtKernelHandle = u32;
pub type TtCbHandle = usize;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtBufferType {
    Dram = 0,
    L1 = 1,
    SystemMemory = 2,
    L1Small = 3,
    Trace = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtDataFormat {
    Float32 = 0,
    Float16 = 1,
    Bfp8 = 2,
    Bfp4 = 3,
    Tf32 = 4,
    Float16B = 5,
    Bfp8B = 6,
    Bfp4B = 7,
    Int32 = 8,
    UInt16 = 9,
    Lf8 = 10,
    Bfp2 = 11,
    Int8 = 14,
    Bfp2B = 15,
    UInt32 = 24,
    Fp8E4m3 = 0x1A,
    UInt8 = 30,
    RawUInt8 = 0xf0,
    RawUInt16 = 0xf1,
    RawUInt32 = 0xf2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtMathFidelity {
    LoFi = 0,
    HiFi2 = 2,
    HiFi3 = 3,
    HiFi4 = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtDataMovementProcessor {
    Riscv0 = 0,
    Riscv1 = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtNoc {
    Noc0 = 0,
    Noc1 = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtNocMode {
    DmDedicated = 0,
    DmDynamic = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtKernelType {
    DataMovement = 0,
    Compute = 1,
    Ethernet = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TtCoreCoord {
    pub x: usize,
    pub y: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TtCoreRange {
    pub start: TtCoreCoord,
    pub end: TtCoreCoord,
}

#[repr(C)]
pub struct TtKernelConfig {
    pub kernel_type: TtKernelType,
    pub processor: TtDataMovementProcessor,
    pub noc: TtNoc,
    pub noc_mode: TtNocMode,
    pub math_fidelity: TtMathFidelity,
    pub fp32_dest_acc_en: i32,
    pub math_approx_mode: i32,
    pub compile_args: *const u32,
    pub compile_args_len: usize,
    pub define_keys: *const *const c_char,
    pub define_values: *const *const c_char,
    pub defines_len: usize,
}

#[repr(C)]
pub struct TtCbDataFormatEntry {
    pub buffer_index: u8,
    pub data_format: TtDataFormat,
}

#[repr(C)]
pub struct TtCbConfig {
    pub total_size: u32,
    pub data_format_spec: *const TtCbDataFormatEntry,
    pub data_format_spec_len: usize,
}

unsafe extern "C" {
    pub fn ttrs_get_num_available_devices() -> usize;
    pub fn ttrs_get_num_pcie_devices() -> usize;
    pub fn ttrs_is_mock_mode() -> i32;
    pub fn ttrs_is_simulator_mode() -> i32;
    pub fn ttrs_device_create(device_id: i32, num_hw_cqs: u8, result: *mut TtResult) -> *mut TtDevice;
    pub fn ttrs_device_close(device: *mut TtDevice);

    pub fn ttrs_program_create() -> *mut TtProgram;
    pub fn ttrs_program_destroy(program: *mut TtProgram);

    pub fn ttrs_create_kernel_from_string(
        program: *mut TtProgram,
        kernel_source: *const c_char,
        core_range: TtCoreRange,
        config: *const TtKernelConfig,
        result: *mut TtResult,
    ) -> TtKernelHandle;

    pub fn ttrs_create_kernel(
        program: *mut TtProgram,
        file_path: *const c_char,
        core_range: TtCoreRange,
        config: *const TtKernelConfig,
        result: *mut TtResult,
    ) -> TtKernelHandle;

    pub fn ttrs_set_runtime_args(
        program: *mut TtProgram,
        kernel: TtKernelHandle,
        core: TtCoreCoord,
        args: *const u32,
        args_len: usize,
        result: *mut TtResult,
    );

    pub fn ttrs_set_runtime_args_range(
        program: *mut TtProgram,
        kernel: TtKernelHandle,
        range: TtCoreRange,
        args: *const u32,
        args_len: usize,
        result: *mut TtResult,
    );

    pub fn ttrs_set_common_runtime_args(
        program: *mut TtProgram,
        kernel: TtKernelHandle,
        args: *const u32,
        args_len: usize,
        result: *mut TtResult,
    );

    pub fn ttrs_create_circular_buffer(
        program: *mut TtProgram,
        core_range: TtCoreRange,
        config: *const TtCbConfig,
        result: *mut TtResult,
    ) -> TtCbHandle;

    pub fn ttrs_buffer_create_interleaved(
        device: *mut TtDevice,
        size: u64,
        page_size: u64,
        buffer_type: TtBufferType,
        result: *mut TtResult,
    ) -> *mut TtBuffer;

    pub fn ttrs_buffer_destroy(buffer: *mut TtBuffer);
    pub fn ttrs_buffer_address(buffer: *const TtBuffer) -> u32;
    pub fn ttrs_buffer_size(buffer: *const TtBuffer) -> u64;

    pub fn ttrs_buffer_write(
        buffer: *mut TtBuffer,
        data: *const u8,
        data_len: usize,
        result: *mut TtResult,
    );

    pub fn ttrs_buffer_read(
        buffer: *mut TtBuffer,
        data_out: *mut u8,
        data_len: usize,
        result: *mut TtResult,
    );

    pub fn ttrs_compile_program(device: *mut TtDevice, program: *mut TtProgram, result: *mut TtResult);
    pub fn ttrs_launch_program(
        device: *mut TtDevice,
        program: *mut TtProgram,
        wait_for_completion: i32,
        result: *mut TtResult,
    );
    pub fn ttrs_wait_program_done(device: *mut TtDevice, program: *mut TtProgram, result: *mut TtResult);
}
