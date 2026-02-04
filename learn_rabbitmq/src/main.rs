use lapin::{
    options::*, types::FieldTable, Connection, ConnectionProperties,
    Channel, Result as LapinResult,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// Global RabbitMQ configuration
static RABBITMQ_CONFIG: Lazy<Mutex<RabbitMQConfig>> = Lazy::new(|| {
    Mutex::new(RabbitMQConfig {
        url: "amqp://services:services@10.90.96.52/sos".to_string(),
        queue_name: "hello_queue".to_string(),
        exchange_name: "hello_exchange".to_string(),
    })
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitMQConfig {
    pub url: String,
    pub queue_name: String,
    pub exchange_name: String,
}

// Message structure for serialization
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: u32,
    pub content: String,
}

async fn create_connection() -> LapinResult<Connection> {
    let config = RABBITMQ_CONFIG.lock().unwrap().clone();
    println!("Connecting to RabbitMQ at: {}", config.url);
    
    Connection::connect(
        &config.url,
        ConnectionProperties::default(),
    ).await
}

async fn create_channel(conn: &Connection) -> LapinResult<Channel> {
    conn.create_channel().await
}

// Example 1: Simple producer - sends a message to a queue
// ⚠️  Sử dụng DEFAULT EXCHANGE (empty string "")
// 🔴 LƯU Ý: KHÔNG THỂ không có exchange! "" = DEFAULT EXCHANGE (type: direct)
// Default exchange tự động bind đến TẤT CẢ queues với routing key = tên queue
async fn simple_producer() -> LapinResult<()> {
    println!("\n=== Example 1: Simple Producer ===");
    
    let conn = create_connection().await?;
    let channel = create_channel(&conn).await?;
    
    let config = RABBITMQ_CONFIG.lock().unwrap().clone();
    
    // Declare a queue
    let _queue = channel
        .queue_declare(
            &config.queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    // Send a message
    let message = Message {
        id: 1,
        content: "Hello from RabbitMQ!".to_string(),
    };
    
    let payload = serde_json::to_string(&message).unwrap();
    
    channel
        .basic_publish(
            "",  // ← EMPTY = Default Exchange (type: direct)
            &config.queue_name,  // ← Routing key = tên queue (gửi thẳng đến queue)
            BasicPublishOptions::default(),
            payload.as_bytes(),
            lapin::BasicProperties::default(),
        )
        .await?;
    
    println!("✓ Sent message: {:?}", message);
    println!("ℹ️  Gửi qua DEFAULT EXCHANGE → trực tiếp đến queue '{}'", config.queue_name);
    
    Ok(())
}

// Example 2: Simple consumer - receives messages from a queue
async fn simple_consumer() -> LapinResult<()> {
    println!("\n=== Example 2: Simple Consumer ===");
    
    let conn = create_connection().await?;
    let channel = create_channel(&conn).await?;
    
    let config = RABBITMQ_CONFIG.lock().unwrap().clone();
    
    // Declare a queue
    let _queue = channel
        .queue_declare(
            &config.queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    println!("Waiting for messages. Press Ctrl+C to exit.");
    
    // Create consumer
    let mut consumer = channel
        .basic_consume(
            &config.queue_name,
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    // Process messages
    use futures::StreamExt;
    
    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let message_str = String::from_utf8_lossy(&delivery.data);
            
            match serde_json::from_str::<Message>(&message_str) {
                Ok(msg) => {
                    println!("✓ Received message: {:?}", msg);
                    
                    // Acknowledge the message
                    delivery
                        .ack(BasicAckOptions::default())
                        .await
                        .expect("Failed to ack");
                }
                Err(e) => {
                    println!("✗ Failed to parse message: {}", e);
                }
            }
        }
    }
    
    Ok(())
}

// Example 3: Work queue - multiple workers sharing tasks
async fn work_queue_producer() -> LapinResult<()> {
    println!("\n=== Example 3: Work Queue Producer ===");
    
    let conn = create_connection().await?;
    let channel = create_channel(&conn).await?;
    
    let queue_name = "task_queue";
    
    // Declare a durable queue
    let _queue = channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    
    // Send multiple tasks
    for i in 1..=5 {
        let message = Message {
            id: i,
            content: format!("Task {}", i),
        };
        
        let payload = serde_json::to_string(&message).unwrap();
        
        channel
            .basic_publish(
                "",
                queue_name,
                BasicPublishOptions::default(),
                payload.as_bytes(),
                lapin::BasicProperties::default()
                    .with_delivery_mode(2), // Persistent message
            )
            .await?;
        
        println!("✓ Sent task: {:?}", message);
    }
    
    Ok(())
}

// Example 4: Publish/Subscribe pattern with exchange
// ✅ Sử dụng CUSTOM EXCHANGE (hello_exchange) - type FANOUT
// MỖI consumer sẽ nhận được TẤT CẢ messages
async fn publish_subscribe_publisher() -> LapinResult<()> {
    println!("\n=== Example 4: Publish/Subscribe Publisher ===");
    println!("⚠️  Chạy publish_subscribe_subscriber() ở các terminal khác trước!");
    
    let conn = create_connection().await?;
    let channel = create_channel(&conn).await?;
    
    let config = RABBITMQ_CONFIG.lock().unwrap().clone();
    
    // BƯỚC 1: Tạo FANOUT exchange
    // FANOUT = Broadcast message đến TẤT CẢ queues đã bind vào exchange này
    channel
        .exchange_declare(
            &config.exchange_name,  // "hello_exchange"
            lapin::ExchangeKind::Fanout,  // Type: FANOUT = broadcast
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    println!("✓ Exchange '{}' (type: FANOUT) ready", config.exchange_name);
    
    // Publish message to exchange
    let message = Message {
        id: 100,
        content: "Broadcast message to all subscribers!".to_string(),
    };
    
    let payload = serde_json::to_string(&message).unwrap();
    
    // BƯỚC 2: Publish message VÀO EXCHANGE (không phải queue!)
    channel
        .basic_publish(
            &config.exchange_name,  // ← Gửi VÀO EXCHANGE "hello_exchange"
            "",  // ← Routing key (fanout không dùng, để empty)
            BasicPublishOptions::default(),
            payload.as_bytes(),
            lapin::BasicProperties::default(),
        )
        .await?;
    
    println!("✓ Published message: {:?}", message);
    println!("✓ Exchange '{}' sẽ BROADCAST đến TẤT CẢ queues đã bind!", config.exchange_name);
    println!("ℹ️  Luồng: Publisher → [{}:FANOUT] → All Bound Queues → Consumers", config.exchange_name);
    
    Ok(())
}

// Example 5: Publish/Subscribe subscriber
// ✅ Mỗi subscriber tạo QUEUE RIÊNG và BIND vào EXCHANGE
// → TẤT CẢ đều nhận message từ exchange
async fn publish_subscribe_subscriber(subscriber_name: &str) -> LapinResult<()> {
    println!("\n=== Example 5: Publish/Subscribe Subscriber [{}] ===", subscriber_name);
    
    let conn = create_connection().await?;
    let channel = create_channel(&conn).await?;
    
    let config = RABBITMQ_CONFIG.lock().unwrap().clone();
    
    // BƯỚC 1: Đảm bảo exchange tồn tại
    channel
        .exchange_declare(
            &config.exchange_name,  // "hello_exchange"
            lapin::ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    // BƯỚC 2: Tạo queue TẠM (exclusive) - MỖI subscriber có queue RIÊNG
    // ⚠️  Đây là key point: Mỗi terminal tạo 1 queue khác nhau!
    let queue = channel
        .queue_declare(
            "",  // ← Empty name = RabbitMQ tự tạo tên RANDOM (vd: amq.gen-xyz123)
            QueueDeclareOptions {
                exclusive: true,  // Queue này CHỈ cho connection này, không share
                auto_delete: true,  // Tự xóa khi subscriber disconnect
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    
    let queue_name = queue.name().as_str();
    println!("✓ Created exclusive queue: {} (chỉ cho subscriber này)", queue_name);
    
    // BƯỚC 3: BIND queue vào exchange
    // Đây là bước QUAN TRỌNG: Kết nối queue của mình với exchange
    channel
        .queue_bind(
            queue_name,  // ← Queue của mình
            &config.exchange_name,  // ← Kết nối đến "hello_exchange"
            "",  // ← Routing key (fanout không cần)
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    println!("✓ Queue '{}' BOUND to exchange '{}'", queue_name, config.exchange_name);
    println!("ℹ️  Khi có message → Exchange broadcast → Queue này nhận được!");
    
    println!("✓ [{}] Waiting for broadcast messages...", subscriber_name);
    
    // Create consumer
    let mut consumer = channel
        .basic_consume(
            queue_name,
            subscriber_name,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    // Process messages
    use futures::StreamExt;
    
    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let message_str = String::from_utf8_lossy(&delivery.data);
            
            match serde_json::from_str::<Message>(&message_str) {
                Ok(msg) => {
                    println!("✓ [{}] Received broadcast: {:?}", subscriber_name, msg);
                    
                    delivery
                        .ack(BasicAckOptions::default())
                        .await
                        .expect("Failed to ack");
                }
                Err(e) => {
                    println!("✗ [{}] Failed to parse message: {}", subscriber_name, e);
                }
            }
        }
    }
    
    Ok(())
}

// Example 6: Direct Exchange - Routing by exact key
// Gửi message đến queues CỤ THỂ dựa trên routing key CHÍNH XÁC
async fn direct_exchange_publisher(routing_key: &str, message_content: &str) -> LapinResult<()> {
    println!("\n=== Example 6: Direct Exchange Publisher ===");
    println!("Publishing with routing_key: '{}'", routing_key);
    
    let conn = create_connection().await?;
    let channel = create_channel(&conn).await?;
    
    let exchange_name = "logs_direct";
    
    // Tạo DIRECT exchange
    channel
        .exchange_declare(
            exchange_name,
            lapin::ExchangeKind::Direct,  // Type: DIRECT
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    println!("✓ Exchange '{}' (type: DIRECT) ready", exchange_name);
    
    let message = Message {
        id: 200,
        content: message_content.to_string(),
    };
    
    let payload = serde_json::to_string(&message).unwrap();
    
    // Publish với routing key CỤ THỂ
    channel
        .basic_publish(
            exchange_name,
            routing_key,  // ← Routing key: "error", "warning", "info"
            BasicPublishOptions::default(),
            payload.as_bytes(),
            lapin::BasicProperties::default(),
        )
        .await?;
    
    println!("✓ Published: {:?} with routing_key='{}'", message, routing_key);
    println!("ℹ️  Chỉ queues bind với routing_key='{}' mới nhận!", routing_key);
    
    Ok(())
}

// Example 6b: Direct Exchange Subscriber
// Subscribe với routing key CỤ THỂ
async fn direct_exchange_subscriber(routing_keys: Vec<&str>, subscriber_name: &str) -> LapinResult<()> {
    println!("\n=== Example 6: Direct Exchange Subscriber [{}] ===", subscriber_name);
    println!("Subscribing to routing keys: {:?}", routing_keys);
    
    let conn = create_connection().await?;
    let channel = create_channel(&conn).await?;
    
    let exchange_name = "logs_direct";
    
    // Declare exchange
    channel
        .exchange_declare(
            exchange_name,
            lapin::ExchangeKind::Direct,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    // Tạo queue exclusive
    let queue = channel
        .queue_declare(
            "",
            QueueDeclareOptions {
                exclusive: true,
                auto_delete: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    
    let queue_name = queue.name().as_str();
    println!("✓ Created exclusive queue: {}", queue_name);
    
    // BIND queue với NHIỀU routing keys
    for routing_key in &routing_keys {
        channel
            .queue_bind(
                queue_name,
                exchange_name,
                routing_key,  // ← Bind với routing key cụ thể
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
        
        println!("✓ Bound to routing_key: '{}'", routing_key);
    }
    
    println!("✓ [{}] Waiting for messages with routing keys: {:?}...", subscriber_name, routing_keys);
    
    let mut consumer = channel
        .basic_consume(
            queue_name,
            subscriber_name,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    use futures::StreamExt;
    
    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let routing_key = delivery.routing_key.as_str();
            let message_str = String::from_utf8_lossy(&delivery.data);
            
            match serde_json::from_str::<Message>(&message_str) {
                Ok(msg) => {
                    println!("✓ [{}] Received [{}]: {:?}", subscriber_name, routing_key, msg);
                    
                    delivery
                        .ack(BasicAckOptions::default())
                        .await
                        .expect("Failed to ack");
                }
                Err(e) => {
                    println!("✗ [{}] Failed to parse: {}", subscriber_name, e);
                }
            }
        }
    }
    
    Ok(())
}

// Example 7: Topic Exchange - Pattern matching routing
// Routing dựa trên PATTERN (wildcards: * và #)
async fn topic_exchange_publisher(routing_key: &str, message_content: &str) -> LapinResult<()> {
    println!("\n=== Example 7: Topic Exchange Publisher ===");
    println!("Publishing with routing_key: '{}'", routing_key);
    
    let conn = create_connection().await?;
    let channel = create_channel(&conn).await?;
    
    let exchange_name = "logs_topic";
    
    // Tạo TOPIC exchange
    channel
        .exchange_declare(
            exchange_name,
            lapin::ExchangeKind::Topic,  // Type: TOPIC
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    println!("✓ Exchange '{}' (type: TOPIC) ready", exchange_name);
    
    let message = Message {
        id: 300,
        content: message_content.to_string(),
    };
    
    let payload = serde_json::to_string(&message).unwrap();
    
    // Publish với routing key (dạng: word.word.word)
    channel
        .basic_publish(
            exchange_name,
            routing_key,  // ← "user.created", "order.payment.success", etc.
            BasicPublishOptions::default(),
            payload.as_bytes(),
            lapin::BasicProperties::default(),
        )
        .await?;
    
    println!("✓ Published: {:?} with routing_key='{}'", message, routing_key);
    println!("ℹ️  Queues với pattern matching '{}' sẽ nhận!", routing_key);
    
    Ok(())
}

// Example 7b: Topic Exchange Subscriber
// Subscribe với PATTERN (*, #)
async fn topic_exchange_subscriber(binding_key: &str, subscriber_name: &str) -> LapinResult<()> {
    println!("\n=== Example 7: Topic Exchange Subscriber [{}] ===", subscriber_name);
    println!("Subscribing to pattern: '{}'", binding_key);
    println!("  * = match exactly 1 word");
    println!("  # = match 0 or more words");
    
    let conn = create_connection().await?;
    let channel = create_channel(&conn).await?;
    
    let exchange_name = "logs_topic";
    
    // Declare exchange
    channel
        .exchange_declare(
            exchange_name,
            lapin::ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    // Tạo queue exclusive
    let queue = channel
        .queue_declare(
            "",
            QueueDeclareOptions {
                exclusive: true,
                auto_delete: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    
    let queue_name = queue.name().as_str();
    println!("✓ Created exclusive queue: {}", queue_name);
    
    // BIND với PATTERN
    channel
        .queue_bind(
            queue_name,
            exchange_name,
            binding_key,  // ← Pattern: "user.*", "order.#", "*.created", etc.
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    println!("✓ Bound with pattern: '{}'", binding_key);
    println!("✓ [{}] Waiting for messages matching pattern...", subscriber_name);
    
    let mut consumer = channel
        .basic_consume(
            queue_name,
            subscriber_name,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    use futures::StreamExt;
    
    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let routing_key = delivery.routing_key.as_str();
            let message_str = String::from_utf8_lossy(&delivery.data);
            
            match serde_json::from_str::<Message>(&message_str) {
                Ok(msg) => {
                    println!("✓ [{}] Matched! routing_key='{}': {:?}", 
                        subscriber_name, routing_key, msg);
                    
                    delivery
                        .ack(BasicAckOptions::default())
                        .await
                        .expect("Failed to ack");
                }
                Err(e) => {
                    println!("✗ [{}] Failed to parse: {}", subscriber_name, e);
                }
            }
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> LapinResult<()> {
    println!("🐰 RabbitMQ Learning Examples\n");
    
    // You can modify the global config if needed
    {
        let mut config = RABBITMQ_CONFIG.lock().unwrap();
        println!("Current RabbitMQ Config:");
        println!("  URL: {}", config.url);
        println!("  Queue: {}", config.queue_name);
        println!("  Exchange: {}", config.exchange_name);
    }
    
    // Uncomment the example you want to run:
    
    // ==========================================
    // QUEUE PATTERN (chỉ 1 consumer nhận message)
    // ==========================================
    
    // Example 1: Send a simple message
    // simple_producer().await?;
    
    // Example 2: Receive messages (this will block waiting for messages)
    // ⚠️  Chạy ở nhiều terminal -> chỉ 1 consumer nhận được mỗi message (load balancing)
    // simple_consumer().await?;
    
    // Example 3: Send work queue tasks
    // work_queue_producer().await?;
    
    // ==========================================
    // PUBLISH/SUBSCRIBE PATTERN (TẤT CẢ subscribers nhận message)
    // ==========================================
    
    // Example 4: Publish message to all subscribers
    // ⚠️  Chạy Example 5 ở các terminal khác TRƯỚC, sau đó chạy cái này
    // publish_subscribe_publisher().await?;
    
    // Example 5: Subscribe to receive ALL messages
    // ⚠️  Chạy ở nhiều terminal -> TẤT CẢ đều nhận được message
    // Đổi tên subscriber cho mỗi terminal: "subscriber_1", "subscriber_2", etc.
    // publish_subscribe_subscriber("subscriber_1").await?;
    
    // ==========================================
    // ROUTING PATTERN - DIRECT EXCHANGE
    // ==========================================
    
    // Example 6: Direct Exchange - Routing by exact key
    // Publish message với routing key cụ thể
    // ⚠️  Chạy Example 6b (subscribers) ở các terminal khác TRƯỚC
    
    // Gửi ERROR log
    // direct_exchange_publisher("error", "Database connection failed!").await?;
    
    // Gửi WARNING log
    // direct_exchange_publisher("warning", "High memory usage detected").await?;
    
    // Gửi INFO log
    // direct_exchange_publisher("info", "User logged in successfully").await?;
    
    // Example 6b: Subscribe với routing keys CỤ THỂ
    // Terminal 1: Chỉ nhận ERROR
    // direct_exchange_subscriber(vec!["error"], "error_logger").await?;
    
    // Terminal 2: Nhận cả ERROR và WARNING
    // direct_exchange_subscriber(vec!["error", "warning"], "important_logger").await?;
    
    // Terminal 3: Nhận TẤT CẢ (error, warning, info)
    // direct_exchange_subscriber(vec!["error", "warning", "info"], "all_logger").await?;
    
    // ==========================================
    // ROUTING PATTERN - TOPIC EXCHANGE
    // ==========================================
    
    // Example 7: Topic Exchange - Pattern matching
    // Publish với routing key phức tạp (word.word.word)
    // ⚠️  Chạy Example 7b (subscribers) ở các terminal khác TRƯỚC
    
    // Publish events
    topic_exchange_publisher("user.created", "New user registered").await?;
    // topic_exchange_publisher("user.updated", "User profile updated").await?;
    // topic_exchange_publisher("user.deleted", "User account deleted").await?;
    // topic_exchange_publisher("order.created", "New order placed").await?;
    // topic_exchange_publisher("order.payment.success", "Payment completed").await?;
    // topic_exchange_publisher("order.payment.failed", "Payment failed").await?;
    
    // Example 7b: Subscribe với PATTERN
    // Terminal 1: Tất cả user events (user.*)
    // topic_exchange_subscriber("user.*", "user_service").await?;
    
    // Terminal 2: Tất cả events (bất kỳ level nào) (#)
    // topic_exchange_subscriber("#", "audit_logger").await?;
    
    // Terminal 3: Tất cả payment events (order.payment.*)
    // topic_exchange_subscriber("order.payment.*", "payment_service").await?;
    
    // Terminal 4: Tất cả order events (order.#)
    // topic_exchange_subscriber("order.#", "order_service").await?;
    
    // Terminal 5: Tất cả "created" events (*.created)
    // topic_exchange_subscriber("*.created", "notification_service").await?;

    println!("\n✓ Done!");
    
    Ok(())
}

