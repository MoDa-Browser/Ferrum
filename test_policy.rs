use moda_security::{Capability, PolicyManager, SecurityPolicy};

fn main() {
    println!("Testing Security Policy Module...");

    let manager = PolicyManager::new();

    let policy = SecurityPolicy::new("test-resource", "Test resource policy")
        .with_allowed_capabilities(vec![Capability::NetworkAccess])
        .with_denied_capabilities(vec![Capability::FileSystemWrite]);

    match manager.add_policy(policy) {
        Ok(_) => println!("✓ Policy added successfully"),
        Err(e) => println!("✗ Failed to add policy: {:?}", e),
    }

    match manager.check_resource_capability("test-resource", &Capability::NetworkAccess) {
        Ok(true) => println!("✓ Network access allowed as expected"),
        Ok(false) => println!("✗ Network access unexpectedly denied"),
        Err(e) => println!("✗ Error checking network access: {:?}", e),
    }

    match manager.check_resource_capability("test-resource", &Capability::FileSystemWrite) {
        Ok(false) => println!("✓ File system write correctly denied"),
        Ok(true) => println!("✗ File system write unexpectedly allowed"),
        Err(e) => println!("✗ Error checking file system write: {:?}", e),
    }

    println!("Security Policy Module test completed!");
}
