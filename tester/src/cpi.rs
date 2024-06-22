use std::sync::Arc;

use solana_rbpf::{memory_region::MemoryMapping, program::SBPFVersion, vm::{Config, EbpfVm, TestContextObject}};

use crate::config::create_program_runtime_environment_v1;

 // The Murmur3 hash value (used by RBPF) of the string "entrypoint"
 const ENTRYPOINT_KEY: u32 = 0x71E3CF81;


 #[test]
 fn testing_cpi() {
    let mut result = create_program_runtime_environment_v1(false).unwrap();

    let function= result.get_function_registry().lookup_by_key(ENTRYPOINT_KEY).map(|(_name, function)| function).unwrap();

    let mock_config = Config::default();
    let empty_memory_mapping =
            MemoryMapping::new(Vec::new(), &mock_config, &SBPFVersion::V1).unwrap();
    
            let mut context_object: TestContextObject = TestContextObject::new(15000000000);

    let mut vm = EbpfVm::new(
        Arc::new(result),
        &SBPFVersion::V1,
        // Removes lifetime tracking
        &mut context_object,
        empty_memory_mapping,
        0,
    );
    
    vm.invoke_function(function);

 }
