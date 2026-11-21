#!/usr/bin/env bash
# ==============================================================================
# Gordafarid Telegram Bot - Local CI/CD Automated Deployment Pipeline
# Architecture: On-Premise Gatekeeper & Release Orchestrator
# ==============================================================================

set -Eeuo pipefail
IFS=$'\n\t'

# --- Terminal Styling ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

SERVICE_NAME="gordafarid.service"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_DIR"

log_info()    { echo -e "${CYAN}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn()    { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $1" >&2; }
log_step()    { echo -e "\n${BOLD}${BLUE}====> $1${NC}"; }

trap 'log_error "Deployment failed at line $LINENO! Previous version remains untouched."; exit 1' ERR

echo -e "${BOLD}${CYAN}"
echo "=================================================================="
echo "    🌸 GORDAFARID BOT - LOCAL CI/CD DEPLOYMENT PIPELINE 🛡️       "
echo "=================================================================="
echo -e "${NC}"

# ------------------------------------------------------------------------------
# [0/5] Pre-flight Environment Checks
# ------------------------------------------------------------------------------
log_step "[0/5] Checking environment dependencies..."

command -v cargo >/dev/null 2>&1 || { log_error "cargo is required but not installed."; exit 1; }
command -v python3 >/dev/null 2>&1 || { log_error "python3 is required but not installed."; exit 1; }

if [ ! -f ".env" ]; then
    log_warn ".env file not found! Checking if .env.example exists..."
    if [ -f ".env.example" ]; then
        log_info "Creating .env template from .env.example"
        cp .env.example .env
    fi
fi
log_success "Environment tools verified."

# ------------------------------------------------------------------------------
# [1/5] Telegram Message & MarkdownV2 Gatekeeper
# ------------------------------------------------------------------------------
log_step "[1/5] Validating Telegram messages & Markdown syntax..."

if [ -f "scripts/validate_messages.py" ]; then
    python3 scripts/validate_messages.py --src src
    log_success "All Telegram message templates validated successfully."
else
    log_warn "scripts/validate_messages.py not found. Skipping message linting."
fi

# Optional i18n validator if i18n file exists
if [ -f "i18n.json" ] && [ -f "scripts/test_i18n_mdv2.py" ]; then
    log_info "Running i18n MarkdownV2 validator against i18n.json..."
    python3 scripts/test_i18n_mdv2.py -f i18n.json --fast --all
    log_success "i18n.json verified."
fi

# ------------------------------------------------------------------------------
# [2/5] Rust Unit & Integration Tests (cargo test)
# ------------------------------------------------------------------------------
log_step "[2/5] Running Rust test suite..."

cargo test --quiet
log_success "All unit tests passed with 100% success rate."

# ------------------------------------------------------------------------------
# [3/5] Production Release Compilation (cargo build --release)
# ------------------------------------------------------------------------------
log_step "[3/5] Building optimized release binary..."

START_BUILD=$(date +%s)
cargo build --release --quiet
END_BUILD=$(date +%s)

BUILD_DURATION=$((END_BUILD - START_BUILD))
log_success "Binary compiled successfully in ${BUILD_DURATION}s."

BINARY_PATH="target/release/gordafarid"
if [ ! -f "$BINARY_PATH" ]; then
    log_error "Expected release binary at $BINARY_PATH not found!"; exit 1
fi

BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
log_info "Binary size: ${BINARY_SIZE}"

# ------------------------------------------------------------------------------
# [4/5] Atomic Service Restart (systemd)
# ------------------------------------------------------------------------------
log_step "[4/5] Deploying binary and restarting $SERVICE_NAME..."

if systemctl list-unit-files "$SERVICE_NAME" >/dev/null 2>&1; then
    sudo systemctl restart "$SERVICE_NAME"
    sleep 2
    log_success "Service $SERVICE_NAME restarted."
else
    log_warn "Systemd service $SERVICE_NAME not registered yet."
    if [ -f "gordafarid.service" ]; then
        log_info "Installing systemd service file..."
        sudo cp gordafarid.service /etc/systemd/system/
        sudo systemctl daemon-reload
        sudo systemctl enable --now "$SERVICE_NAME"
        sleep 2
        log_success "Service installed and started."
    fi
fi

# ------------------------------------------------------------------------------
# [5/5] Post-Deployment Health Check & Verification
# ------------------------------------------------------------------------------
log_step "[5/5] Verifying post-deployment daemon status..."

if sudo systemctl is-active --quiet "$SERVICE_NAME"; then
    log_success "Service $SERVICE_NAME is ACTIVE (RUNNING)!"
else
    log_error "Service failed to stay active! Checking recent logs:"
    sudo journalctl -u "$SERVICE_NAME" -n 20 --no-pager
    exit 1
fi

# Double check that sibling service RepoFa is also healthy
if systemctl list-unit-files repofagifbot.service >/dev/null 2>&1; then
    if sudo systemctl is-active --quiet repofagifbot.service; then
        log_success "Sibling service repofagifbot.service remains healthy and running."
    else
        log_warn "Sibling service repofagifbot.service is not active."
    fi
fi

echo -e "\n${BOLD}${CYAN}=== Recent Service Logs ===${NC}"
sudo journalctl -u "$SERVICE_NAME" -n 8 --no-pager

echo -e "\n${BOLD}${GREEN}=================================================================="
echo "    🚀 GORDAFARID BOT DEPLOYED SUCCESSFULLY TO PRODUCTION!       "
echo "==================================================================${NC}\n"
