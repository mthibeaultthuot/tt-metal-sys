use tt_metal::*;

const KERNEL_SOURCE: &str = r#"
#include "api/dataflow/dataflow_api.h"

void kernel_main() {
    uint32_t src0_dram_addr = get_arg_val<uint32_t>(0);
    uint32_t src1_dram_addr = get_arg_val<uint32_t>(1);
    uint32_t dst_dram_addr  = get_arg_val<uint32_t>(2);
    uint32_t src0_l1_addr   = get_arg_val<uint32_t>(3);
    uint32_t src1_l1_addr   = get_arg_val<uint32_t>(4);
    uint32_t dst_l1_addr    = get_arg_val<uint32_t>(5);

    constexpr uint32_t page_size = sizeof(uint32_t);
    InterleavedAddrGen<true> src0 = { .bank_base_address = src0_dram_addr, .page_size = page_size };
    InterleavedAddrGen<true> src1 = { .bank_base_address = src1_dram_addr, .page_size = page_size };
    InterleavedAddrGen<true> dst  = { .bank_base_address = dst_dram_addr,  .page_size = page_size };

    uint64_t src0_noc_addr = get_noc_addr(0, src0);
    uint64_t src1_noc_addr = get_noc_addr(0, src1);
    noc_async_read(src0_noc_addr, src0_l1_addr, page_size);
    noc_async_read(src1_noc_addr, src1_l1_addr, page_size);
    noc_async_read_barrier();  // Wait for both reads to complete.

    uint32_t* a   = (uint32_t*)src0_l1_addr;
    uint32_t* b   = (uint32_t*)src1_l1_addr;
    uint32_t* out = (uint32_t*)dst_l1_addr;
    *out = *a + *b;

    uint64_t dst_noc_addr = get_noc_addr(0, dst);
    noc_async_write(dst_l1_addr, dst_noc_addr, page_size);
    noc_async_write_barrier();  // Wait for the write to complete.
}
"#;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let num_devices = Device::num_available();
    println!("Available Tenstorrent devices: {num_devices}");
    if num_devices == 0 {
        eprintln!("No Tenstorrent devices found. Cannot run this example.");
        std::process::exit(1);
    }

    println!("Opening device 0...");
    let device = Device::open(0, 1)?;

    let mut program = Program::new();

    let core = CoreRange::single(0, 0);

    let buf_size: u64 = std::mem::size_of::<u32>() as u64;

    let mut src0_dram = Buffer::new(&device, buf_size, buf_size, BufferType::Dram)?;
    let mut src1_dram = Buffer::new(&device, buf_size, buf_size, BufferType::Dram)?;
    let dst_dram = Buffer::new(&device, buf_size, buf_size, BufferType::Dram)?;
    let src0_l1 = Buffer::new(&device, buf_size, buf_size, BufferType::L1)?;
    let src1_l1 = Buffer::new(&device, buf_size, buf_size, BufferType::L1)?;
    let dst_l1 = Buffer::new(&device, buf_size, buf_size, BufferType::L1)?;

    let a: u32 = 14;
    let b: u32 = 7;
    println!("Writing inputs: {a} + {b}");
    src0_dram.write_typed(&[a])?;
    src1_dram.write_typed(&[b])?;

    println!("Compiling kernel from source string...");
    let kernel = program.create_kernel_from_string(
        KERNEL_SOURCE,
        core,
        KernelConfig::data_movement(),
    )?;

    program.set_runtime_args(
        kernel,
        CoreCoord::new(0, 0),
        &[
            src0_dram.address(),
            src1_dram.address(),
            dst_dram.address(),
            src0_l1.address(),
            src1_l1.address(),
            dst_l1.address(),
        ],
    )?;

    println!("Launching program...");
    device.launch_program(&mut program, true)?;

    let mut result = [0u32; 1];
    dst_dram.read_typed(&mut result)?;

    println!("Result: {} + {} = {}", a, b, result[0]);

    if result[0] == a + b {
        println!("SUCCESS!");
    } else {
        eprintln!("FAILURE: expected {}, got {}", a + b, result[0]);
        std::process::exit(1);
    }

    Ok(())
}
