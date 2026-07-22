#!/bin/bash

##############################################################################
# Repository Consistency Checker
# Automated script to validate repository state against documented standards
#
# Usage: bash scripts/check-consistency.sh [--fix] [--verbose]
# Exit Code: 0 (all checks pass), 1 (failures found)
##############################################################################

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Flags
FIX_MODE=false
VERBOSE=false
FAILED_CHECKS=0

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --fix) FIX_MODE=true; shift ;;
    --verbose) VERBOSE=true; shift ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

print_header() {
  echo -e "${BLUE}================================${NC}"
  echo -e "${BLUE}$1${NC}"
  echo -e "${BLUE}================================${NC}"
}

print_success() {
  echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
  echo -e "${RED}❌ $1${NC}"
  FAILED_CHECKS=$((FAILED_CHECKS + 1))
}

print_warning() {
  echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
  if [ "$VERBOSE" = true ]; then
    echo -e "${BLUE}ℹ️  $1${NC}"
  fi
}

print_fix() {
  if [ "$FIX_MODE" = true ]; then
    echo -e "${GREEN}🔧 $1${NC}"
  fi
}

##############################################################################
# 1. VERSION CONSISTENCY CHECKS
##############################################################################

check_version_consistency() {
  print_header "1. Version Consistency"

  # Extract versions
  CARGO_VERSION=$(grep '^version' src-tauri/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
  PYTHON_VERSION=$(grep '^version' ingestion/pyproject.toml | sed 's/version = "\(.*\)"/\1/')
  TAURI_CONF_VERSION=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*"\([^"]*\)".*/\1/')
  
  print_info "Cargo version: $CARGO_VERSION"
  print_info "Python version: $PYTHON_VERSION"
  print_info "Tauri config version: $TAURI_CONF_VERSION"

  # Check Cargo vs Python
  if [ "$CARGO_VERSION" = "$PYTHON_VERSION" ]; then
    print_success "Cargo and Python versions match ($CARGO_VERSION)"
  else
    print_error "Version mismatch: Cargo=$CARGO_VERSION, Python=$PYTHON_VERSION"
    if [ "$FIX_MODE" = true ]; then
      print_fix "Updating Python version to match Cargo..."
      sed -i "s/^version = \".*\"/version = \"$CARGO_VERSION\"/" ingestion/pyproject.toml
    fi
  fi

  # Check Tauri config version
  if [ "$CARGO_VERSION" = "$TAURI_CONF_VERSION" ]; then
    print_success "Tauri config version matches release version ($CARGO_VERSION)"
  else
    print_error "Tauri config version ($TAURI_CONF_VERSION) differs from release version ($CARGO_VERSION)"
    if [ "$FIX_MODE" = true ]; then
      print_fix "Updating tauri.conf.json version..."
      sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$CARGO_VERSION\"/" src-tauri/tauri.conf.json
    fi
  fi

  # Check package.json version
  if grep -q '"version"' package.json; then
    PKG_VERSION=$(grep '"version"' package.json | head -1 | cut -d'"' -f4)
    if [ "$CARGO_VERSION" = "$PKG_VERSION" ]; then
      print_success "package.json version matches ($PKG_VERSION)"
    else
      print_error "package.json version ($PKG_VERSION) differs from release version ($CARGO_VERSION)"
      if [ "$FIX_MODE" = true ]; then
        print_fix "Updating package.json version..."
        sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$CARGO_VERSION\"/" package.json
      fi
    fi
  else
    print_warning "package.json missing version field"
    if [ "$FIX_MODE" = true ]; then
      print_fix "Adding version to package.json..."
      sed -i "s/\"name\": \"zs-note\",/\"name\": \"zs-note\",\n  \"version\": \"$CARGO_VERSION\",/" package.json
    fi
  fi
}

##############################################################################
# 2. ORGANIZATION REFERENCE CHECKS
##############################################################################

check_org_references() {
  print_header "2. Organization References"

  # Check for wrong org in markdown files
  WRONG_ORG_FILES=$(grep -r "zarishsphere" --include="*.md" . 2>/dev/null || true)
  
  if [ -z "$WRONG_ORG_FILES" ]; then
    print_success "No zarishsphere references found in documentation"
  else
    print_error "Found zarishsphere references (should be devopsariful):"
    echo "$WRONG_ORG_FILES" | while read -r line; do
      print_info "  $line"
    done

    if [ "$FIX_MODE" = true ]; then
      print_fix "Replacing zarishsphere with devopsariful..."
      find . -type f \( -name "*.md" -o -name "*.json" \) \
        -exec sed -i 's|zarishsphere/zs-note|devopsariful/zs-note|g' {} +
    fi
  fi

  # Check for wrong org in JSON config files
  WRONG_ORG_JSON=$(grep -r "zarishsphere" --include="*.json" . 2>/dev/null || true)
  
  if [ -z "$WRONG_ORG_JSON" ]; then
    print_success "No zarishsphere references found in JSON"
  else
    print_error "Found zarishsphere in JSON files:"
    echo "$WRONG_ORG_JSON" | while read -r line; do
      print_info "  $line"
    done
  fi
}

##############################################################################
# 3. DEPENDENCY VERSION CHECKS
##############################################################################

check_dependency_versions() {
  print_header "3. Dependency Versions"

  # Check Node dependencies
  print_info "Checking Node dependencies..."
  
  MILKDOWN_MATH=$(grep '@milkdown/plugin-math' package.json | grep -oP '\^\d+\.\d+\.\d+' | head -1)
  MILKDOWN_TABLE=$(grep '@milkdown/plugin-table' package.json | grep -oP '\^\d+\.\d+\.\d+' | head -1)
  MILKDOWN_CORE=$(grep '@milkdown/core' package.json | grep -oP '\^\d+\.\d+\.\d+' | head -1)

  if [ -n "$MILKDOWN_MATH" ] && [ -n "$MILKDOWN_TABLE" ] && [ -n "$MILKDOWN_CORE" ]; then
    if [[ "$MILKDOWN_MATH" == *"7.5"* ]] && [[ "$MILKDOWN_CORE" == *"7.21"* ]]; then
      print_error "Milkdown plugin-math version ($MILKDOWN_MATH) out of sync with core ($MILKDOWN_CORE)"
      if [ "$FIX_MODE" = true ]; then
        print_fix "Updating Milkdown plugins..."
        sed -i 's/"@milkdown\/plugin-math": "[^"]*"/"@milkdown\/plugin-math": "^7.21.2"/' package.json
      fi
    fi
    
    if [[ "$MILKDOWN_TABLE" == *"5.3"* ]] && [[ "$MILKDOWN_CORE" == *"7.21"* ]]; then
      print_error "Milkdown plugin-table version ($MILKDOWN_TABLE) out of sync with core ($MILKDOWN_CORE)"
      if [ "$FIX_MODE" = true ]; then
        print_fix "Updating Milkdown plugin-table..."
        sed -i 's/"@milkdown\/plugin-table": "[^"]*"/"@milkdown\/plugin-table": "^7.21.2"/' package.json
      fi
    fi
  fi

  # Check Tauri versions
  TAURI_API=$(grep '@tauri-apps/api' package.json | grep -oP '\^\d+\.\d+\.\d+' | head -1)
  TAURI_CLI=$(grep '@tauri-apps/cli' package.json | grep -oP '\^\d+\.\d+\.\d+' | head -1)

  if [ -n "$TAURI_API" ] && [ -n "$TAURI_CLI" ]; then
    if [ "$TAURI_API" != "$TAURI_CLI" ]; then
      print_error "Tauri API ($TAURI_API) and CLI ($TAURI_CLI) versions mismatch"
      if [ "$FIX_MODE" = true ]; then
        print_fix "Aligning Tauri versions..."
        sed -i 's/"@tauri-apps\/api": "[^"]*"/"@tauri-apps\/api": "^2.11.4"/' package.json
        sed -i 's/"@tauri-apps\/cli": "[^"]*"/"@tauri-apps\/cli": "^2.11.4"/' package.json
      fi
    else
      print_success "Tauri API and CLI versions match ($TAURI_API)"
    fi
  fi
}

##############################################################################
# 4. CARGO TOML CHECKS
##############################################################################

check_cargo_dependencies() {
  print_header "4. Cargo Dependencies"

  # Check if whisper-rs is in voice feature gate
  if grep -q 'whisper-rs' src-tauri/Cargo.toml; then
    if grep -A5 'voice = \[' src-tauri/Cargo.toml | grep -q 'whisper-rs'; then
      print_success "whisper-rs is properly gated in voice feature"
    else
      print_error "whisper-rs is not included in voice feature gate"
      if [ "$FIX_MODE" = true ]; then
        print_fix "Adding whisper-rs to voice feature..."
        sed -i 's/voice = \[\(.*\)\]/voice = [\1, "dep:whisper-rs"]/' src-tauri/Cargo.toml
      fi
    fi
  fi
}

##############################################################################
# 5. CONFIGURATION CHECKS
##############################################################################

check_configuration() {
  print_header "5. Configuration Files"

  # Check CSP policy
  if grep -q "'unsafe-inline'" src-tauri/tauri.conf.json; then
    print_error "CSP policy contains 'unsafe-inline' (security risk)"
    if [ "$FIX_MODE" = true ]; then
      print_fix "Removing unsafe-inline from CSP..."
      sed -i "s/style-src 'self' 'unsafe-inline'/style-src 'self'/" src-tauri/tauri.conf.json
    fi
  else
    print_success "CSP policy does not use 'unsafe-inline'"
  fi

  # Check for ESLint config
  if grep -q "eslint src" package.json; then
    if [ -f ".eslintrc.js" ] || [ -f ".eslintrc.json" ] || [ -f ".eslintrc.cjs" ]; then
      print_success "ESLint configuration found"
    else
      print_warning "ESLint is in npm scripts but no .eslintrc config found"
    fi
  else
    print_success "ESLint not configured in package.json scripts"
  fi
}

##############################################################################
# 6. DOCUMENTATION CHECKS
##############################################################################

check_documentation() {
  print_header "6. Documentation Structure"

  # Check for required README files
  if [ -f "README.md" ]; then
    print_success "Root README.md exists"
  else
    print_error "Root README.md not found"
  fi

  if [ -f "docs/README.md" ]; then
    print_success "docs/README.md exists"
  else
    print_error "docs/README.md not found"
  fi

  # Check for empty spec directories
  EMPTY_DIRS=0
  for dir in docs/001-concept docs/002-specifications docs/003-architecture; do
    if [ -d "$dir" ]; then
      FILE_COUNT=$(find "$dir" -type f | wc -l)
      if [ "$FILE_COUNT" -eq 0 ]; then
        print_warning "$dir is empty"
        EMPTY_DIRS=$((EMPTY_DIRS + 1))
      fi
    fi
  done

  if [ "$EMPTY_DIRS" -gt 0 ]; then
    print_error "Found $EMPTY_DIRS empty documentation directories"
  fi
}

##############################################################################
# 7. ENGINE CONSTRAINTS CHECK
##############################################################################

check_engine_constraints() {
  print_header "7. Engine Constraints"

  # Check Node.js constraints
  if grep -q '"node": "\^24.15.0"' package.json; then
    print_warning "Node.js version constraint is too strict (^24.15.0)"
    print_info "  Should be: >=24.15.0"
    if [ "$FIX_MODE" = true ]; then
      print_fix "Relaxing Node.js version constraint..."
      sed -i 's/"node": "\^24\.15\.0"/"node": ">=24.15.0"/' package.json
    fi
  else
    print_success "Node.js version constraint is reasonable"
  fi

  # Check pnpm constraints
  if grep -q '"pnpm": "\^11.5.1"' package.json; then
    print_warning "pnpm version constraint is too strict (^11.5.1)"
    print_info "  Should be: >=11.5.0"
    if [ "$FIX_MODE" = true ]; then
      print_fix "Relaxing pnpm version constraint..."
      sed -i 's/"pnpm": "\^11\.5\.1"/"pnpm": ">=11.5.0"/' package.json
    fi
  else
    print_success "pnpm version constraint is reasonable"
  fi
}

##############################################################################
# 8. BUILD COMMAND CHECKS
##############################################################################

check_build_commands() {
  print_header "8. Build Commands"

  # Check if tauri dev script exists
  if grep -q '"dev":' package.json; then
    DEV_CMD=$(grep '"dev":' package.json | head -1 | cut -d'"' -f4)
    if [ "$DEV_CMD" = "vite" ]; then
      if ! grep -q '"dev:tauri"' package.json; then
        print_warning "No tauri dev script found (only vite dev)"
        if [ "$FIX_MODE" = true ]; then
          print_fix "Adding dev:tauri script..."
          sed -i '/"dev": "vite",/a\    "dev:tauri": "tauri dev",' package.json
        fi
      else
        print_success "tauri dev script exists"
      fi
    fi
  fi
}

##############################################################################
# 9. GITIGNORE CHECKS
##############################################################################

check_gitignore() {
  print_header "9. .gitignore Files"

  if [ -f ".gitignore" ]; then
    print_success "Root .gitignore exists"
  else
    print_error "Root .gitignore not found"
  fi

  # Check for per-directory gitignore files
  for dir in src src-tauri ingestion; do
    if [ -d "$dir" ]; then
      if [ -f "$dir/.gitignore" ]; then
        print_success "$dir/.gitignore exists"
      else
        print_warning "$dir/.gitignore missing"
      fi
    fi
  done
}

##############################################################################
# MAIN EXECUTION
##############################################################################

main() {
  print_header "🔍 Repository Consistency Checker"

  # Run all checks
  check_version_consistency
  check_org_references
  check_dependency_versions
  check_cargo_dependencies
  check_configuration
  check_documentation
  check_engine_constraints
  check_build_commands
  check_gitignore

  # Summary
  print_header "📊 Summary"
  
  if [ "$FAILED_CHECKS" -eq 0 ]; then
    print_success "All consistency checks passed! ✨"
    exit 0
  else
    print_error "Found $FAILED_CHECKS issue(s)"
    
    if [ "$FIX_MODE" = false ]; then
      echo ""
      echo -e "${YELLOW}💡 Tip: Run with --fix flag to auto-correct issues${NC}"
      echo "   bash scripts/check-consistency.sh --fix"
    fi
    
    exit 1
  fi
}

main
