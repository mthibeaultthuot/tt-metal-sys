use tt_metal::*;

const KERNEL_SOURCE: &str = r#"
#include "api/dataflow/dataflow_api.h"

void kernel_main() {
}
"#;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {

    let num_devices = Device::num_available();
    println!("Available devices: {num_devices}");
    if num_devices == 0 {
        eprintln!("No Tenstorrent devices found.");
        std::process::exit(1);
    }

    if Device::is_mock_mode() {
        println!("WARNING: Running in MOCK mode. Kernel compilation and execution are not supported in mock mode.");
        println!("To run on real hardware, unset TT_METAL_MOCK_CLUSTER_DESC_PATH.");
        return Ok(());
    }

    if Device::is_simulator_mode() {
        println!("WARNING: Running in SIMULATOR mode.");
    }

    let device = Device::open(0, 1)?;
    let mut program = Program::new();

    let core = CoreRange::single(0, 0);

    let kernel = program.create_kernel_from_string(
        KERNEL_SOURCE,
        core,
        KernelConfig::data_movement(),
    )?;
    println!("Kernel compiled, handle = {kernel:?}");

    device.launch_program(&mut program, true)?;
    println!("Program launched and completed.");
    Ok(())
}
