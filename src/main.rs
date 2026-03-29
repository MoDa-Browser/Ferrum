use moda_security::{CapabilityToken, Capability, PermissionManager, PermissionPolicy};
use moda_sandbox::{Sandbox, NamespaceConfig};
use moda_ipc::{IpcChannel, IpcMessage, IpcSecurity};
use moda_network::{HttpClient, TlsConfig, TlsVersion};
use moda_storage::SecureStorage;
use moda_platform::PlatformInfo;
use moda_render::{LayoutEngine, DOMNode, NodeType, Rect};

fn main() {
    println!("MoDa Browser Core - Prototype");
    println!("==============================");
    println!();

    println!("1. Platform Detection");
    println!("--------------------");
    let platform_info = PlatformInfo::new();
    println!("Platform: {}", platform_info.platform().name());
    println!("Architecture: {}", platform_info.arch());
    println!("Sandbox supported: {}", platform_info.platform().supports_sandbox());
    println!();

    println!("2. Security Framework");
    println!("--------------------");
    let token = CapabilityToken::new("test-token")
        .with_capability(Capability::NetworkAccess)
        .with_capability(Capability::FileSystemRead);
    
    println!("Token ID: {}", token.id());
    println!("Capabilities: {:?}", token.capabilities());
    println!("Is expired: {}", token.is_expired());
    println!();

    println!("3. Permission Management");
    println!("-----------------------");
    let permission_manager = PermissionManager::new();
    let policy = PermissionPolicy {
        allowed_capabilities: vec![Capability::NetworkAccess],
        denied_capabilities: vec![Capability::FileSystemWrite],
    };
    let _ = permission_manager.add_policy("test-resource", policy);
    
    match permission_manager.check_permission("test-resource", &Capability::NetworkAccess) {
        Ok(allowed) => println!("Network access allowed: {}", allowed),
        Err(e) => println!("Error: {}", e),
    }
    println!();

    println!("4. Sandbox Management");
    println!("--------------------");
    let namespace_config = NamespaceConfig::new()
        .with_pid(true)
        .with_network(true);
    
    let _sandbox = Sandbox::new()
        .with_namespace(namespace_config);
    
    println!("Sandbox configured with PID and network namespaces");
    println!();

    println!("5. IPC Communication");
    println!("-------------------");
    let channel = IpcChannel::new();
    let security = IpcSecurity::new().with_authentication(true);
    
    let message = IpcMessage::new("sender", "receiver", vec![1, 2, 3, 4, 5]);
    
    match security.validate_message(&message) {
        Ok(_) => println!("Message validated successfully"),
        Err(e) => println!("Validation error: {}", e),
    }
    
    match channel.send(message) {
        Ok(_) => println!("Message sent successfully"),
        Err(e) => println!("Send error: {}", e),
    }
    
    match channel.receive() {
        Ok(received) => println!("Message received: {} -> {}", received.source, received.target),
        Err(e) => println!("Receive error: {}", e),
    }
    println!();

    println!("6. Network Stack");
    println!("---------------");
    let tls_config = TlsConfig::new()
        .with_min_tls_version(TlsVersion::Tls1_2)
        .with_max_tls_version(TlsVersion::Tls1_3);
    
    match tls_config.validate() {
        Ok(_) => println!("TLS configuration valid"),
        Err(e) => println!("TLS config error: {}", e),
    }
    
    let _http_client = HttpClient::new();
    println!("HTTP client initialized");
    println!();

    println!("7. Secure Storage");
    println!("----------------");
    let key = [0u8; 32];
    let storage = SecureStorage::new(&key);
    
    let plaintext = b"Hello, MoDa Browser!";
    let encrypted = storage.encrypt(plaintext);
    
    match encrypted {
        Ok(data) => {
            println!("Encrypted data size: {} bytes", data.ciphertext.len());
            
            let decrypted = storage.decrypt(&data);
            match decrypted {
                Ok(decrypted_data) => println!("Decrypted: {}", String::from_utf8_lossy(&decrypted_data)),
                Err(e) => println!("Decryption error: {}", e),
            }
        }
        Err(e) => println!("Encryption error: {}", e),
    }
    println!();

    println!("8. Render Engine");
    println!("----------------");
    let mut layout_engine = LayoutEngine::new();
    layout_engine.parse_html("<html><body><h1>Hello, MoDa!</h1></body></html>");
    layout_engine.calculate_layout();
    
    println!("HTML parsed and layout calculated");
    
    if let Some(bounds) = layout_engine.get_element_bounds("root") {
        println!("Root element bounds: x={}, y={}, width={}, height={}", 
            bounds.x, bounds.y, bounds.width, bounds.height);
    }
    
    let div_node = DOMNode::new("test-div", NodeType::Element)
        .with_tag_name("div")
        .with_text("Test content")
        .with_bounds(Rect::new(10.0, 10.0, 200.0, 100.0));
    
    println!("Created DOM node: {} with bounds ({}, {}, {}, {})", 
        div_node.tag_name, div_node.bounds.x, div_node.bounds.y, 
        div_node.bounds.width, div_node.bounds.height);
    println!();

    println!("==============================");
    println!("Prototype execution complete!");
    println!("All core components initialized successfully.");
}
