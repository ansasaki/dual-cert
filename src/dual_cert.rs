/*!
Dual Certificate Support - Rust Proof of Concept

This server demonstrates true dual certificate support (ECDSA + RSA) using
rust-openssl with multiple certificate calls.

The server automatically presents the appropriate certificate based on
client cipher preferences without requiring callbacks.
*/

use actix_web::{web, App, HttpResponse, HttpServer, Result, middleware::Logger};
use openssl::ssl::{SslAcceptor, SslMethod, SslFiletype};
use serde_json::json;
use std::io;

async fn hello() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body("Hello from Rust dual certificate server!"))
}

async fn info() -> Result<HttpResponse> {
    let info = json!({
        "message": "Rust dual certificate server",
        "framework": "actix-web + rust-openssl",
        "approach": "Multiple certificate calls",
        "certificates": ["ECDSA (P-256)", "RSA (2048-bit)"],
        "automatic_selection": true,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(info))
}

async fn cert_info() -> Result<HttpResponse> {
    // Note: In a real implementation, you could inspect the TLS connection
    // to determine which certificate was actually used for this request
    let cert_info = json!({
        "note": "Certificate selection is automatic based on client cipher preferences",
        "ecdsa_usage": "Presented when client supports ECDHE-ECDSA ciphers",
        "rsa_usage": "Presented when client prefers or only supports ECDHE-RSA ciphers",
        "test_commands": [
            "openssl s_client -connect localhost:8443 -cipher 'ECDHE-ECDSA-AES256-GCM-SHA384'",
            "openssl s_client -connect localhost:8443 -cipher 'ECDHE-RSA-AES256-GCM-SHA384'"
        ]
    });

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(cert_info))
}

/// Create SSL acceptor with dual certificate support
///
/// This function demonstrates the simple approach to dual certificates:
/// 1. Load ECDSA certificate and key
/// 2. Load RSA certificate and key
/// 3. OpenSSL automatically handles certificate selection during handshake
fn create_dual_cert_ssl_acceptor() -> io::Result<openssl::ssl::SslAcceptorBuilder> {
    println!("🔧 Creating SSL acceptor with dual certificate support...");

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;

    // Load ECDSA certificate first
    println!("📜 Loading ECDSA certificate...");
    builder.set_private_key_file("certs/ecdsa-key.pem", SslFiletype::PEM)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to load ECDSA private key: {}", e)))?;

    builder.set_certificate_chain_file("certs/ecdsa-cert.pem")
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to load ECDSA certificate: {}", e)))?;

    builder.check_private_key()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("ECDSA private key check failed: {}", e)))?;

    println!("✓ ECDSA certificate loaded and verified");

    // Load RSA certificate second
    println!("📜 Loading RSA certificate...");
    builder.set_private_key_file("certs/rsa-key.pem", SslFiletype::PEM)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to load RSA private key: {}", e)))?;

    builder.set_certificate_chain_file("certs/rsa-cert.pem")
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to load RSA certificate: {}", e)))?;

    builder.check_private_key()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("RSA private key check failed: {}", e)))?;

    println!("✓ RSA certificate loaded and verified");
    println!("✅ Dual certificate SSL acceptor created successfully");
    println!("🤖 OpenSSL will automatically select appropriate certificate based on client preferences");

    Ok(builder)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    println!("🔒 Rust Dual Certificate Server");
    println!("   Framework: actix-web + rust-openssl");
    println!("   Certificates: ECDSA + RSA");
    println!("   Automatic selection based on client cipher preferences");

    // Create SSL acceptor with dual certificate support
    let ssl_acceptor = match create_dual_cert_ssl_acceptor() {
        Ok(acceptor) => acceptor,
        Err(e) => {
            eprintln!("❌ Failed to create SSL acceptor: {}", e);
            eprintln!("💡 Make sure the certificate files exist:");
            eprintln!("   - certs/ecdsa-cert.pem & certs/ecdsa-key.pem");
            eprintln!("   - certs/rsa-cert.pem & certs/rsa-key.pem");
            return Err(e);
        }
    };

    let port = 8443;
    println!("\n🚀 Starting server on https://localhost:{}", port);
    println!("📋 Available endpoints:");
    println!("   GET /          - Hello message");
    println!("   GET /info      - Server information");
    println!("   GET /cert-info - Certificate selection info");

    println!("\n🧪 Test commands:");
    println!("   # Default connection (should use ECDSA)");
    println!("   curl -k https://localhost:{}", port);
    println!("   ");
    println!("   # Force RSA certificate");
    println!("   openssl s_client -connect localhost:{} -cipher 'ECDHE-RSA-AES256-GCM-SHA384'", port);
    println!("   ");
    println!("   # Force ECDSA certificate");
    println!("   openssl s_client -connect localhost:{} -cipher 'ECDHE-ECDSA-AES256-GCM-SHA384'", port);
    println!("   ");
    println!("   # Check certificate type");
    println!("   openssl s_client -connect localhost:{} -servername localhost </dev/null 2>/dev/null | openssl x509 -text -noout | grep 'Public Key Algorithm'", port);

    println!("\n⏹️  Press Ctrl+C to stop");

    // Start the server
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/", web::get().to(hello))
            .route("/info", web::get().to(info))
            .route("/cert-info", web::get().to(cert_info))
    })
    .bind_openssl(format!("127.0.0.1:{}", port), ssl_acceptor)?
    .run()
    .await?;

    Ok(())
}