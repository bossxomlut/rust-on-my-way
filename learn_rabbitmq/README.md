# RabbitMQ Learning Project

This project demonstrates RabbitMQ usage in Rust using the `lapin` library.

## Prerequisites

1. Install RabbitMQ:

   ```bash
   # On macOS
   brew install rabbitmq
   brew services start rabbitmq
   ```

2. RabbitMQ will be available at:
   - URL: `amqp://guest:guest@localhost:5672`
   - Management UI: `http://localhost:15672` (guest/guest)

## Examples Included

### 1. Simple Producer

Sends a single message to a queue.

### 2. Simple Consumer

Receives and processes messages from a queue (blocking).

### 3. Work Queue Producer

Sends multiple tasks to a durable queue for distributed processing.

### 4. Publish/Subscribe Publisher

Broadcasts messages to multiple subscribers using a fanout exchange.

## Running Examples

Edit `main.rs` and uncomment the example you want to run:

```rust
// Example 1: Send a simple message
simple_producer().await?;

// Example 2: Receive messages (this will block)
// simple_consumer().await?;

// Example 3: Send work queue tasks
// work_queue_producer().await?;

// Example 4: Publish/subscribe pattern
// publish_subscribe_publisher().await?;
```

Then run:

```bash
cargo run
```

## Global Configuration

The RabbitMQ configuration is stored in a global variable using `once_cell::Lazy`:

```rust
static RABBITMQ_CONFIG: Lazy<Mutex<RabbitMQConfig>> = Lazy::new(|| {
    Mutex::new(RabbitMQConfig {
        url: "amqp://guest:guest@localhost:5672".to_string(),
        queue_name: "hello_queue".to_string(),
        exchange_name: "hello_exchange".to_string(),
    })
});
```

You can modify this configuration at runtime if needed.
