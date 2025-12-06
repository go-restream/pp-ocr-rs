#!/bin/bash

# PP-OCR-RS API service test script
# Test API endpoints using test images from docs/img directory

# Configuration
API_BASE_URL="http://localhost:8080"
IMG_DIR="docs/img"
TIMEOUT=30

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if service is running
check_service() {
    log_info "Checking API service status..."
    response=$(curl -s -w "%{http_code}" -o /dev/null "${API_BASE_URL}/v1/health")

    if [ "$response" = "200" ]; then
        log_info "API service is running normally"
        return 0
    else
        log_error "API service is not running or inaccessible (HTTP $response)"
        log_warn "Please start the API service first: ./target/release/ocr serve"
        exit 1
    fi
}

# Get model list
test_models() {
    log_info "\nTesting get model list..."
    response=$(curl -s -w "\n%{http_code}" "${API_BASE_URL}/v1/models")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')

    if [ "$http_code" = "200" ]; then
        log_info "Successfully retrieved model list"
        echo "$body" | python3 -m json.tool 2>/dev/null || echo "$body"
    else
        log_error "Failed to get model list (HTTP $http_code)"
        echo "$body"
    fi
}

# Test single image OCR
test_ocr_image() {
    local image_path=$1
    local model=$2
    local image_name=$(basename "$image_path")

    log_info "\nTesting image: $image_name (model: $model)"

    # Check if image file exists
    if [ ! -f "$image_path" ]; then
        log_error "Image file does not exist: $image_path"
        return 1
    fi

    # Encode image to base64
    base64_image=$(base64 -i "$image_path")

    # Build request JSON
    json_payload=$(cat <<EOF
{
  "model": "$model",
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "ocr"
        },
        {
          "type": "image_url",
          "image_url": {
            "url": "data:image/png;base64,$base64_image"
          }
        }
      ]
    }
  ]
}
EOF
)

    # Send request
    response=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$json_payload" \
        "${API_BASE_URL}/v1/chat/completions")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')

    if [ "$http_code" = "200" ]; then
        log_info "OCR recognition successful"
        # Extract and display recognition results
        echo "$body" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    if 'choices' in data and len(data['choices']) > 0:
        content = data['choices'][0]['message']['content']
        print('Recognition Result:')
        print(content)
    else:
        print('Invalid response format')
        print(json.dumps(data, indent=2, ensure_ascii=False))
except:
    print('Failed to parse response')
    print(body)
" 2>/dev/null || echo "$body"
    else
        log_error "OCR recognition failed (HTTP $http_code)"
        echo "$body"
    fi
}

# Batch test all images
test_all_images() {
    local model=$1

    log_info "\nStarting batch test for all images..."

    # Find all PNG images
    images=($(find "$IMG_DIR" -name "*.png" -type f | sort))

    if [ ${#images[@]} -eq 0 ]; then
        log_error "No PNG images found in $IMG_DIR directory"
        return 1
    fi

    log_info "Found ${#images[@]} test images"

    success_count=0
    total_count=${#images[@]}

    for image in "${images[@]}"; do
        if test_ocr_image "$image" "$model"; then
            ((success_count++))
        fi

        # Add delay to avoid rapid requests
        sleep 1
    done

    log_info "\nTest completed: Success $success_count/$total_count"
}

# Performance test
test_performance() {
    local image_path=$1
    local model=$2
    local iterations=${3:-5}

    log_info "\nPerformance test: $iterations requests"
    log_info "Test image: $(basename "$image_path")"
    log_info "Using model: $model"

    total_time=0

    for i in $(seq 1 $iterations); do
        log_info "Test $i..."

        start_time=$(date +%s.%N)

        # Execute OCR request (silent mode)
        base64_image=$(base64 -i "$image_path")
        json_payload=$(cat <<EOF
{
  "model": "$model",
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "ocr"
        },
        {
          "type": "image_url",
          "image_url": {
            "url": "data:image/png;base64,$base64_image"
          }
        }
      ]
    }
  ]
}
EOF
)

        curl -s -X POST \
            -H "Content-Type: application/json" \
            -d "$json_payload" \
            "${API_BASE_URL}/v1/chat/completions" > /dev/null

        end_time=$(date +%s.%N)
        elapsed=$(echo "$end_time - $start_time" | bc)
        total_time=$(echo "$total_time + $elapsed" | bc)

        printf "  Time taken: %.2f seconds\n" "$elapsed"
    done

    avg_time=$(echo "scale=2; $total_time / $iterations" | bc)
    printf "\nAverage time: %.2f seconds\n" "$avg_time"
}

# Show help information
show_help() {
    echo "PP-OCR-RS API Service Test Script"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -h, --help         Show help information"
    echo "  -c, --check        Only check service status"
    echo "  -m, --models       Test get model list"
    echo "  -a, --all          Batch test all images (using mobile model)"
    echo "  -s, --server       Batch test all images (using server model)"
    echo "  -i, --image FILE   Test specified image"
    echo "  -p, --perf FILE    Perform performance test on specified image"
    echo "  -n, --num NUM      Number of iterations for performance test (default: 5)"
    echo ""
    echo "Examples:"
    echo "  $0 --all                           # Test all images using mobile model"
    echo "  $0 --server                        # Test all images using server model"
    echo "  $0 --image docs/img/test_1.png     # Test single image"
    echo "  $0 --perf docs/img/test_1.png -n 10 # Performance test 10 times"
}

# Main function
main() {
    local check_only=false
    local test_models_only=false
    local test_all_mobile=false
    local test_all_server=false
    local test_image_path=""
    local test_perf_path=""
    local perf_iterations=5

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -c|--check)
                check_only=true
                shift
                ;;
            -m|--models)
                test_models_only=true
                shift
                ;;
            -a|--all)
                test_all_mobile=true
                shift
                ;;
            -s|--server)
                test_all_server=true
                shift
                ;;
            -i|--image)
                test_image_path="$2"
                shift 2
                ;;
            -p|--perf)
                test_perf_path="$2"
                shift 2
                ;;
            -n|--num)
                perf_iterations="$2"
                shift 2
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    # Check service status
    check_service

    # Execute corresponding tests
    if [ "$check_only" = true ]; then
        exit 0
    fi

    if [ "$test_models_only" = true ]; then
        test_models
        exit 0
    fi

    if [ "$test_all_mobile" = true ]; then
        test_models
        test_all_images "ch_pp_ocr_v5_mobile"
        exit 0
    fi

    if [ "$test_all_server" = true ]; then
        test_models
        test_all_images "ch_pp_ocr_v5_server"
        exit 0
    fi

    if [ -n "$test_image_path" ]; then
        test_models
        test_ocr_image "$test_image_path" "ch_pp_ocr_v5_mobile"
        exit 0
    fi

    if [ -n "$test_perf_path" ]; then
        test_models
        test_performance "$test_perf_path" "ch_pp_ocr_v5_mobile" "$perf_iterations"
        exit 0
    fi

    # Default behavior: show help
    show_help
}

# Check dependencies
check_dependencies() {
    local missing_deps=()

    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi

    if ! command -v base64 &> /dev/null; then
        missing_deps+=("base64")
    fi

    if ! command -v python3 &> /dev/null; then
        log_warn "It's recommended to install python3 for better JSON formatting output"
    fi

    if ! command -v bc &> /dev/null; then
        log_warn "It's recommended to install bc for time calculation in performance tests"
    fi

    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        exit 1
    fi
}

# Script entry point
check_dependencies
main "$@"