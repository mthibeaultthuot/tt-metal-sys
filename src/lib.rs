mod ffi;

use std::ffi::{CString, NulError};
use std::fmt;
use std::os::raw::c_char;
use std::ptr;

pub use ffi::{
    TtBufferType as BufferType,
    TtDataFormat as DataFormat,
    TtDataMovementProcessor as DataMovementProcessor,
    TtKernelType as KernelType,
    TtMathFidelity as MathFidelity,
    TtNoc as Noc,
    TtNocMode as NocMode,
};

#[derive(Debug)]
pub enum Error {
    TtMetal(String),
    NulByte(NulError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::TtMetal(msg) => write!(f, "tt-metal error: {msg}"),
            Error::NulByte(e) => write!(f, "null byte in string: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<NulError> for Error {
    fn from(e: NulError) -> Self {
        Error::NulByte(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn check_result(r: &ffi::TtResult) -> Result<()> {
    if r.is_ok() {
        Ok(())
    } else {
        Err(Error::TtMetal(r.error_message()))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CoreCoord {
    pub x: usize,
    pub y: usize,
}

impl CoreCoord {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    fn to_ffi(self) -> ffi::TtCoreCoord {
        ffi::TtCoreCoord {
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CoreRange {
    pub start: CoreCoord,
    pub end: CoreCoord,
}

impl CoreRange {
    pub fn new(start: CoreCoord, end: CoreCoord) -> Self {
        Self { start, end }
    }

    pub fn single(x: usize, y: usize) -> Self {
        let c = CoreCoord::new(x, y);
        Self { start: c, end: c }
    }

    fn to_ffi(self) -> ffi::TtCoreRange {
        ffi::TtCoreRange {
            start: self.start.to_ffi(),
            end: self.end.to_ffi(),
        }
    }
}

pub struct KernelConfig {
    kernel_type: KernelType,
    processor: DataMovementProcessor,
    noc: Noc,
    noc_mode: NocMode,
    math_fidelity: MathFidelity,
    fp32_dest_acc_en: bool,
    math_approx_mode: bool,
    compile_args: Vec<u32>,
    define_keys: Vec<CString>,
    define_values: Vec<CString>,
}

impl KernelConfig {
    pub fn data_movement() -> Self {
        Self {
            kernel_type: KernelType::DataMovement,
            processor: DataMovementProcessor::Riscv0,
            noc: Noc::Noc0,
            noc_mode: NocMode::DmDedicated,
            math_fidelity: MathFidelity::HiFi4,
            fp32_dest_acc_en: false,
            math_approx_mode: false,
            compile_args: Vec::new(),
            define_keys: Vec::new(),
            define_values: Vec::new(),
        }
    }

    pub fn reader() -> Self {
        Self::data_movement()
    }

    pub fn writer() -> Self {
        Self {
            processor: DataMovementProcessor::Riscv1,
            noc: Noc::Noc1,
            ..Self::data_movement()
        }
    }

    pub fn compute() -> Self {
        Self {
            kernel_type: KernelType::Compute,
            ..Self::data_movement()
        }
    }

    pub fn processor(mut self, p: DataMovementProcessor) -> Self {
        self.processor = p;
        self
    }

    pub fn noc(mut self, n: Noc) -> Self {
        self.noc = n;
        self
    }

    pub fn noc_mode(mut self, m: NocMode) -> Self {
        self.noc_mode = m;
        self
    }

    pub fn math_fidelity(mut self, mf: MathFidelity) -> Self {
        self.math_fidelity = mf;
        self
    }

    pub fn fp32_dest_acc(mut self, enable: bool) -> Self {
        self.fp32_dest_acc_en = enable;
        self
    }

    pub fn math_approx(mut self, enable: bool) -> Self {
        self.math_approx_mode = enable;
        self
    }

    pub fn compile_args(mut self, args: Vec<u32>) -> Self {
        self.compile_args = args;
        self
    }

    pub fn define(mut self, key: &str, value: &str) -> Result<Self> {
        self.define_keys.push(CString::new(key)?);
        self.define_values.push(CString::new(value)?);
        Ok(self)
    }

    fn to_ffi(
        &self,
        key_ptrs: &mut Vec<*const c_char>,
        val_ptrs: &mut Vec<*const c_char>,
    ) -> ffi::TtKernelConfig {
        key_ptrs.clear();
        val_ptrs.clear();
        for k in &self.define_keys {
            key_ptrs.push(k.as_ptr());
        }
        for v in &self.define_values {
            val_ptrs.push(v.as_ptr());
        }

        ffi::TtKernelConfig {
            kernel_type: self.kernel_type,
            processor: self.processor,
            noc: self.noc,
            noc_mode: self.noc_mode,
            math_fidelity: self.math_fidelity,
            fp32_dest_acc_en: self.fp32_dest_acc_en as i32,
            math_approx_mode: self.math_approx_mode as i32,
            compile_args: if self.compile_args.is_empty() {
                ptr::null()
            } else {
                self.compile_args.as_ptr()
            },
            compile_args_len: self.compile_args.len(),
            define_keys: if key_ptrs.is_empty() {
                ptr::null()
            } else {
                key_ptrs.as_ptr()
            },
            define_values: if val_ptrs.is_empty() {
                ptr::null()
            } else {
                val_ptrs.as_ptr()
            },
            defines_len: self.define_keys.len(),
        }
    }
}

pub struct CbDataFormatSpec {
    entries: Vec<ffi::TtCbDataFormatEntry>,
}

impl CbDataFormatSpec {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add(mut self, buffer_index: u8, format: DataFormat, page_size: u32) -> Self {
        self.entries.push(ffi::TtCbDataFormatEntry {
            buffer_index,
            data_format: format,
            page_size,
        });
        self
    }
}

pub struct CircularBufferConfig {
    pub total_size: u32,
    pub data_format_spec: CbDataFormatSpec,
}

impl CircularBufferConfig {
    pub fn new(total_size: u32, spec: CbDataFormatSpec) -> Self {
        Self {
            total_size,
            data_format_spec: spec,
        }
    }
}

pub struct Device {
    ptr: *mut ffi::TtDevice,
}

unsafe impl Send for Device {}

impl Device {
    pub fn num_available() -> usize {
        unsafe { ffi::ttrs_get_num_available_devices() }
    }

    pub fn num_pcie() -> usize {
        unsafe { ffi::ttrs_get_num_pcie_devices() }
    }

    pub fn is_mock_mode() -> bool {
        unsafe { ffi::ttrs_is_mock_mode() != 0 }
    }

    pub fn is_simulator_mode() -> bool {
        unsafe { ffi::ttrs_is_simulator_mode() != 0 }
    }

    pub fn open(device_id: i32, num_hw_cqs: u8) -> Result<Self> {
        let mut result = ffi::TtResult::new();
        let ptr = unsafe { ffi::ttrs_device_create(device_id, num_hw_cqs, &mut result) };
        check_result(&result)?;
        Ok(Self { ptr })
    }

    pub fn compile_program(&self, program: &mut Program) -> Result<()> {
        let mut result = ffi::TtResult::new();
        unsafe { ffi::ttrs_compile_program(self.ptr, program.ptr, &mut result) };
        check_result(&result)
    }

    pub fn launch_program(&self, program: &mut Program, wait: bool) -> Result<()> {
        let mut result = ffi::TtResult::new();
        unsafe {
            ffi::ttrs_launch_program(self.ptr, program.ptr, wait as i32, &mut result);
        }
        check_result(&result)
    }

    pub fn wait_program_done(&self, program: &mut Program) -> Result<()> {
        let mut result = ffi::TtResult::new();
        unsafe { ffi::ttrs_wait_program_done(self.ptr, program.ptr, &mut result) };
        check_result(&result)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            ffi::ttrs_device_close(self.ptr);
        }
    }
}

pub struct Program {
    ptr: *mut ffi::TtProgram,
}

unsafe impl Send for Program {}

impl Program {
    pub fn new() -> Self {
        Self {
            ptr: unsafe { ffi::ttrs_program_create() },
        }
    }

    pub fn create_kernel_from_string(
        &mut self,
        source: &str,
        core_range: CoreRange,
        config: KernelConfig,
    ) -> Result<KernelHandle> {
        let c_source = CString::new(source)?;
        let mut key_ptrs = Vec::new();
        let mut val_ptrs = Vec::new();
        let ffi_config = config.to_ffi(&mut key_ptrs, &mut val_ptrs);
        let mut result = ffi::TtResult::new();
        let handle = unsafe {
            ffi::ttrs_create_kernel_from_string(
                self.ptr,
                c_source.as_ptr(),
                core_range.to_ffi(),
                &ffi_config,
                &mut result,
            )
        };
        check_result(&result)?;
        Ok(KernelHandle(handle))
    }

    pub fn create_kernel(
        &mut self,
        file_path: &str,
        core_range: CoreRange,
        config: KernelConfig,
    ) -> Result<KernelHandle> {
        let c_path = CString::new(file_path)?;
        let mut key_ptrs = Vec::new();
        let mut val_ptrs = Vec::new();
        let ffi_config = config.to_ffi(&mut key_ptrs, &mut val_ptrs);
        let mut result = ffi::TtResult::new();
        let handle = unsafe {
            ffi::ttrs_create_kernel(
                self.ptr,
                c_path.as_ptr(),
                core_range.to_ffi(),
                &ffi_config,
                &mut result,
            )
        };
        check_result(&result)?;
        Ok(KernelHandle(handle))
    }

    pub fn set_runtime_args(
        &self,
        kernel: KernelHandle,
        core: CoreCoord,
        args: &[u32],
    ) -> Result<()> {
        let mut result = ffi::TtResult::new();
        unsafe {
            ffi::ttrs_set_runtime_args(
                self.ptr,
                kernel.0,
                core.to_ffi(),
                args.as_ptr(),
                args.len(),
                &mut result,
            );
        }
        check_result(&result)
    }

    pub fn set_runtime_args_range(
        &self,
        kernel: KernelHandle,
        range: CoreRange,
        args: &[u32],
    ) -> Result<()> {
        let mut result = ffi::TtResult::new();
        unsafe {
            ffi::ttrs_set_runtime_args_range(
                self.ptr,
                kernel.0,
                range.to_ffi(),
                args.as_ptr(),
                args.len(),
                &mut result,
            );
        }
        check_result(&result)
    }

    pub fn set_common_runtime_args(
        &self,
        kernel: KernelHandle,
        args: &[u32],
    ) -> Result<()> {
        let mut result = ffi::TtResult::new();
        unsafe {
            ffi::ttrs_set_common_runtime_args(
                self.ptr,
                kernel.0,
                args.as_ptr(),
                args.len(),
                &mut result,
            );
        }
        check_result(&result)
    }

    pub fn create_circular_buffer(
        &mut self,
        core_range: CoreRange,
        config: CircularBufferConfig,
    ) -> Result<CbHandle> {
        let ffi_config = ffi::TtCbConfig {
            total_size: config.total_size,
            data_format_spec: if config.data_format_spec.entries.is_empty() {
                ptr::null()
            } else {
                config.data_format_spec.entries.as_ptr()
            },
            data_format_spec_len: config.data_format_spec.entries.len(),
        };
        let mut result = ffi::TtResult::new();
        let handle = unsafe {
            ffi::ttrs_create_circular_buffer(self.ptr, core_range.to_ffi(), &ffi_config, &mut result)
        };
        check_result(&result)?;
        Ok(CbHandle(handle))
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            ffi::ttrs_program_destroy(self.ptr);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KernelHandle(u32);

#[derive(Debug, Clone, Copy)]
pub struct CbHandle(usize);

pub struct Buffer {
    ptr: *mut ffi::TtBuffer,
}

unsafe impl Send for Buffer {}

impl Buffer {
    pub fn new(
        device: &Device,
        size: u64,
        page_size: u64,
        buffer_type: BufferType,
    ) -> Result<Self> {
        let mut result = ffi::TtResult::new();
        let ptr = unsafe {
            ffi::ttrs_buffer_create_interleaved(device.ptr, size, page_size, buffer_type, &mut result)
        };
        check_result(&result)?;
        Ok(Self { ptr })
    }

    pub fn address(&self) -> u32 {
        unsafe { ffi::ttrs_buffer_address(self.ptr) }
    }

    pub fn size(&self) -> u64 {
        unsafe { ffi::ttrs_buffer_size(self.ptr) }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        let mut result = ffi::TtResult::new();
        unsafe {
            ffi::ttrs_buffer_write(self.ptr, data.as_ptr(), data.len(), &mut result);
        }
        check_result(&result)
    }

    pub fn read(&self, data: &mut [u8]) -> Result<()> {
        let mut result = ffi::TtResult::new();
        unsafe {
            ffi::ttrs_buffer_read(self.ptr as *mut _, data.as_mut_ptr(), data.len(), &mut result);
        }
        check_result(&result)
    }

    pub fn write_typed<T: Copy>(&mut self, data: &[T]) -> Result<()> {
        let bytes = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
        };
        self.write(bytes)
    }

    pub fn read_typed<T: Copy>(&self, data: &mut [T]) -> Result<()> {
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                data.as_mut_ptr() as *mut u8,
                std::mem::size_of_val(data),
            )
        };
        self.read(bytes)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            ffi::ttrs_buffer_destroy(self.ptr);
        }
    }
}
