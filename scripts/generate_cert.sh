#!/bin/bash
# Script to generate self-signed certificates for localhost development

# Check if OpenSSL is installed
if ! command -v openssl &> /dev/null; then
    echo "Error: OpenSSL is not installed. Please install OpenSSL and try again."
    exit 1
fi

# Default values
CERT_DIR="certs"
DOMAIN="localhost"
DAYS=365

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dir)
            CERT_DIR="$2"
            shift 2
            ;;
        --domain)
            DOMAIN="$2"
            shift 2
            ;;
        --days)
            DAYS="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --dir DIR       Directory to store certificates (default: certs)"
            echo "  --domain DOMAIN Domain name for the certificate (default: localhost)"
            echo "  --days DAYS     Validity period in days (default: 365)"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Create certificate directory if it doesn't exist
mkdir -p "$CERT_DIR"

# Generate certificate
echo "Generating self-signed certificate for $DOMAIN (valid for $DAYS days)..."
openssl req -x509 \
    -out "$CERT_DIR/$DOMAIN.crt" \
    -keyout "$CERT_DIR/$DOMAIN.key" \
    -newkey rsa:2048 -nodes -sha256 \
    -subj "/CN=$DOMAIN" -extensions EXT -config <( \
    printf "[dn]\nCN=$DOMAIN\n[req]\ndistinguished_name = dn\n[EXT]\nsubjectAltName=DNS:$DOMAIN\nkeyUsage=digitalSignature\nextendedKeyUsage=serverAuth")

# Check if certificate generation was successful
if [ $? -eq 0 ]; then
    echo "Certificate generated successfully!"
    echo "Certificate: $CERT_DIR/$DOMAIN.crt"
    echo "Private key: $CERT_DIR/$DOMAIN.key"
    echo ""
    echo "To use these certificates with MCP Daemon, configure your application with:"
    echo "  - cert_path: \"$CERT_DIR/$DOMAIN.crt\""
    echo "  - key_path: \"$CERT_DIR/$DOMAIN.key\""
else
    echo "Failed to generate certificate."
    exit 1
fi

# Make the script executable
chmod +x "$0"
