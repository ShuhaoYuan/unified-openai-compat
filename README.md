# Unified OpenAI Compatible Server

A unified OpenAI compatible interface server based on Rust and Actix-Web, supporting unified access to multiple model providers with optional API key authentication.

## Features

- 🚀 **High Performance** - Built on Rust and Actix-Web framework
- 🔌 **Multi-Provider Support** - Unified interface for multiple model providers
- 🔐 **API Key Authentication** - Optional server-level API key authentication
- 🔄 **Dynamic Model Discovery** - Automatically fetch available models from providers
- 📋 **Static Model Configuration** - Configure static models when provider's /models endpoint is unavailable
- ⚡ **Streaming Response** - Full support for OpenAI streaming API
- 🎯 **Smart Routing** - Automatic routing to providers based on model names
- 📊 **Priority Management** - Configuration order determines model priority
- 🛡️ **Full Compatibility** - 100% compatible with OpenAI API format
- 📝 **Detailed Logging** - Comprehensive request and authentication logging

## Quick Start

### Requirements

- Rust 1.70+
- Cargo

### Installation

1. Clone the project
```bash
git clone <repository-url>
cd unified-openai-compat
```

2. Configure providers
Edit the `config.toml` file to add your model providers and optional server API key:

```toml
# Optional API key for the unified server
# If not set, the server will not require authentication
# server_api_key = "your-unified-server-api-key"

# First provider (higher priority)
[[providers]]
base_url = "http://localhost:8000/v1"
api_key = ""
models = ["glm-4.6", "glm-4"]  # Static models when /models endpoint is unavailable

# Second provider (lower priority)
[[providers]]
base_url = "https://api.openai.com/v1"
api_key = "your-openai-api-key"

# Third provider
[[providers]]
base_url = "https://api.anthropic.com"
api_key = "your-anthropic-api-key"
```

3. Run the server
```bash
cargo run
```

The server will start on `http://127.0.0.1:8080`. You'll see output indicating:
- Whether API key authentication is enabled
- List of configured providers with their priority order

## API Endpoints

### Get Models List

```bash
curl http://127.0.0.1:8080/v1/models
```

Returns available models from all providers, with fields completely from original providers.

### Chat Completion (Non-streaming)

```bash
curl -X POST http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-server-api-key" \
  -d '{
    "model": "your-model-name",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ],
    "max_tokens": 100
  }'
```

### Chat Completion (Streaming)

```bash
curl -X POST http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-server-api-key" \
  -d '{
    "model": "your-model-name",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ],
    "max_tokens": 100,
    "stream": true
  }'
```

**Note**: The `Authorization` header is only required if `server_api_key` is configured in `config.toml`. If not set, the server runs in development mode without authentication.

## Configuration

### Configuration File Structure

`config.toml` supports the following configuration:

```toml
# Optional API key for the unified server
# If not set, the server will not require authentication
server_api_key = "your-unified-server-api-key"

# Multiple providers can be configured
[[providers]]
base_url = "http://localhost:8000/v1"  # Provider API URL
api_key = ""                           # API key (optional)
models = ["glm-4.6", "glm-4"]  # Static models when /models endpoint is unavailabl

[[providers]]
base_url = "https://api.openai.com/v1"
api_key = "sk-..."
```

### Static Model Configuration

When a provider's `/models` endpoint is unavailable or unreliable, you can configure static models:

```toml
[[providers]]
base_url = "https://example-provider.com/v1"
api_key = "your-api-key"
models = ["model-1", "model-2", "model-3"]
```

**Benefits of Static Models:**
- **Reliability**: Works even when provider's `/models` endpoint is down
- **Performance**: Faster startup since no HTTP requests needed for model discovery
- **Control**: Explicitly define which models are available for each provider

**Behavior:**
- If `models` is configured, the system will use the static list instead of fetching from the provider
- If `models` is not configured, the system will fetch models from the provider's `/models` endpoint
- Static models are formatted as standard OpenAI model objects with `id`, `object`, `created`, and `owned_by` fields

### Authentication

- **Server API Key**: Optional authentication for the unified server itself
  - If `server_api_key` is set, clients must include `Authorization: Bearer <server_api_key>` header
  - If not set, the server runs in development mode with no authentication required
- **Provider API Keys**: Each provider can have its own API key for authentication with the upstream service
- **Models Endpoint**: The `/v1/models` endpoint bypasses authentication for easy model discovery
- **Chat Completions**: Requires authentication when `server_api_key` is configured

### Priority Rules

- The order **from top to bottom** in the configuration file determines priority
- If multiple providers have models with the same name, the provider **higher** in the configuration is used
- Model list is automatically deduplicated, keeping the highest priority version

### Supported Providers

Theoretically supports any provider compatible with OpenAI API format:

- ✅ VLLM
- ✅ OpenAI
- ✅ Anthropic Claude
- ✅ LocalAI
- ✅ Ollama (requires compatible mode configuration)
- ✅ Other OpenAI compatible services

## How It Works

### Model Discovery

1. Server checks each provider for static models configuration
2. If static models are configured, uses them directly
3. Otherwise, fetches `/models` list from the provider's endpoint
4. Processes in configuration order, removing duplicate models
5. Keeps the highest priority model information

**Static vs Dynamic Discovery:**
- **Static Models**: Use configured `models` array when provider's `/models` endpoint is unavailable
- **Dynamic Models**: Automatically fetch from provider's `/models` endpoint when no static configuration exists
- **Fallback**: If dynamic fetching fails, the provider contributes no models (but doesn't break the system)

### Request Routing

1. Receives chat completion request
2. Finds corresponding provider based on `model` field in request
3. Forwards request to target provider
4. Returns original response (supports both streaming and non-streaming)

### Data Transparency

- `/v1/models` endpoint returns **completely original** provider data
- Preserves all fields: `id`, `object`, `created`, `owned_by`, `max_model_len`, etc.
- Does not modify or filter any metadata

## Project Structure

```
unified-openai-compat/
├── src/
│   ├── main.rs          # Server entry point and startup logic
│   ├── config.rs        # Configuration management and model discovery
│   ├── handlers.rs      # HTTP request handlers for models and chat completions
│   └── middleware.rs    # API key authentication middleware
├── config.toml          # Provider and server configuration file
├── Cargo.toml           # Rust project configuration and dependencies
└── README.md           # Project documentation
```

## Development

### Build Project

```bash
cargo build --release
```

### Run Tests

```bash
cargo test
```

### Code Check

```bash
cargo check
cargo clippy
```

## Usage Examples

### Python Client

```python
import openai

# Configure client to point to unified interface
client = openai.OpenAI(
    base_url="http://127.0.0.1:8080/v1",
    api_key="any-key"  # Can be any value
)

# Use models from different providers
response = client.chat.completions.create(
    model="gpt-4",  # Automatically routed to OpenAI
    messages=[{"role": "user", "content": "Hello!"}]
)

response = client.chat.completions.create(
    model="local-model",  # Automatically routed to local VLLM
    messages=[{"role": "user", "content": "Hello!"}]
)
```

### JavaScript Client

```javascript
import OpenAI from 'openai';

const openai = new OpenAI({
  baseURL: 'http://127.0.0.1:8080/v1',
  apiKey: 'any-key', // Can be any value
  dangerouslyAllowBrowser: true
});

const completion = await openai.chat.completions.create({
  model: 'your-model',
  messages: [{ role: 'user', content: 'Hello!' }],
  stream: true
});

for await (const chunk of completion) {
  process.stdout.write(chunk.choices[0]?.delta?.content || '');
}
```

## Performance Features

- **Async Processing** - Based on Tokio async runtime
- **Connection Reuse** - HTTP client connection pool
- **Memory Safety** - Rust memory safety guarantees
- **Zero Copy** - Avoid data copying when possible
- **Error Recovery** - Single provider failure doesn't affect other providers

## Troubleshooting

### Common Issues

1. **Empty Model List**
   - Check if provider URLs are correct
   - Verify API keys are valid
   - Confirm provider services are running

2. **Request Failures**
   - Check server logs for detailed error information
   - Verify model names are correct
   - Confirm target provider is available

3. **Authentication Issues**
   - Ensure `Authorization: Bearer <server_api_key>` header is included when `server_api_key` is configured
   - Check that the API key matches exactly with the one in `config.toml`
   - Verify the header format is correct (Bearer prefix required)

4. **Streaming Response Issues**
   - Confirm client supports Server-Sent Events
   - Check network connection stability

### Log Viewing

The server outputs detailed logs when running:

```bash
RUST_LOG=debug cargo run
```

## Contributing

Issues and Pull Requests are welcome!

### Development Workflow

1. Fork the project
2. Create feature branch
3. Commit changes
4. Create Pull Request

## License

MIT License

## Changelog

### v0.1.1
- **Static Model Configuration**: Added support for configuring static models when provider's `/models` endpoint is unavailable
- **Improved Reliability**: System now gracefully handles provider endpoint failures
- **Enhanced Performance**: Faster startup when using static models configuration
- **Flexible Configuration**: Support for both static and dynamic model discovery per provider

### v0.1.0
- Initial release
- Multi-provider unified interface support
- Full OpenAI API compatibility
- Streaming response support
- Dynamic model discovery
- Priority routing
- Optional API key authentication
- Comprehensive logging and error handling
