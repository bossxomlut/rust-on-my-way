# Exchange trong RabbitMQ - Giải thích chi tiết

## Exchange là gì?

**Exchange** là "bưu điện trung tâm" của RabbitMQ. Nó nhận messages từ producers và quyết định gửi messages đến queue(s) nào dựa trên:

- **Type** của exchange (fanout, direct, topic, headers)
- **Routing key**
- **Bindings** (liên kết giữa exchange và queues)

## Luồng hoạt động

```
Producer → Exchange → Queue(s) → Consumer(s)
              ↓
         [Routing Logic]
```

**QUAN TRỌNG:** Producer KHÔNG BAO GIỜ gửi message trực tiếp đến queue. Luôn luôn phải qua Exchange!

## Ví dụ với `hello_exchange` trong code

### 1. Tạo Exchange (Fanout Type)

```rust
channel.exchange_declare(
    "hello_exchange",           // Tên exchange
    lapin::ExchangeKind::Fanout, // Type: Fanout = broadcast
    ExchangeDeclareOptions::default(),
    FieldTable::default(),
)
```

### 2. Publisher gửi message VÀO EXCHANGE

```rust
channel.basic_publish(
    "hello_exchange",  // ← Gửi vào EXCHANGE, không phải queue!
    "",               // ← Routing key (fanout không cần)
    BasicPublishOptions::default(),
    payload.as_bytes(),
    lapin::BasicProperties::default(),
)
```

### 3. Subscriber tạo queue và BIND vào exchange

```rust
// Bước 1: Tạo queue
let queue = channel.queue_declare(
    "",  // Tên random
    QueueDeclareOptions { exclusive: true, ... },
    FieldTable::default(),
).await?;

// Bước 2: BIND queue vào exchange
channel.queue_bind(
    queue_name,         // Queue của mình
    "hello_exchange",   // ← Kết nối đến exchange này
    "",                // Routing key (fanout không cần)
    QueueBindOptions::default(),
    FieldTable::default(),
)
```

## Các loại Exchange

### 1. **FANOUT** (như `hello_exchange`)

- **Chức năng:** Broadcast - gửi message đến **TẤT CẢ** queues đã bind
- **Routing key:** Bỏ qua, không quan tâm
- **Use case:** Notifications, broadcast events

```
Publisher → [hello_exchange:fanout] → Queue A → Consumer A
                    ↓
                 Queue B → Consumer B
                    ↓
                 Queue C → Consumer C

➡️ TẤT CẢ consumers đều nhận được message!
```

### 2. **DIRECT**

- **Chức năng:** Gửi đến queue có **routing key khớp chính xác**
- **Routing key:** Bắt buộc và phải match chính xác
- **Use case:** Task distribution by type

```
Publisher
  → [logs_exchange:direct] với routing_key="error"
       → Queue_Errors (bind với "error") ✓
       → Queue_Warnings (bind với "warning") ✗

➡️ Chỉ Queue_Errors nhận message!
```

### 3. **TOPIC**

- **Chức năng:** Pattern matching với routing key (wildcards: \* và #)
- **Routing key:** Pattern như "user.\*.created", "order.#"
- **Use case:** Flexible routing, log filtering

```
routing_key: "user.profile.updated"

Bindings:
  Queue A: "user.*.*"        ✓ Match
  Queue B: "user.#"          ✓ Match
  Queue C: "order.#"         ✗ No match
  Queue D: "*.profile.*"     ✓ Match
```

### 4. **HEADERS**

- **Chức năng:** Route dựa trên message headers, không dùng routing key
- **Use case:** Complex routing logic

## So sánh: Có Exchange vs Không Exchange

### ❌ Gửi trực tiếp đến Queue (như Example 1 & 2)

```rust
// Publisher
channel.basic_publish(
    "",              // ← Empty string = "default exchange"
    "hello_queue",   // ← Routing key = tên queue
    ...
)
```

- Sử dụng **default exchange** (exchange có sẵn, type: direct)
- Routing key = tên queue
- Message đi thẳng đến 1 queue cụ thể
- **Hạn chế:** Không flexible, không thể broadcast

### ✅ Gửi qua Exchange (như Example 4 & 5)

```rust
// Publisher
channel.basic_publish(
    "hello_exchange",  // ← Gửi vào custom exchange
    "",               // ← Routing key (fanout không cần)
    ...
)
```

- Sử dụng **custom exchange**
- Exchange điều phối đến nhiều queues
- **Ưu điểm:** Flexible, có thể broadcast, route theo điều kiện

## Tại sao cần Exchange?

### 1. **Decoupling**

Producer không cần biết có bao nhiêu consumers hay queues nào tồn tại.

### 2. **Flexibility**

Thay đổi routing logic mà không đổi code producer.

### 3. **Scalability**

Dễ dàng thêm/bớt consumers mà không ảnh hưởng producer.

### 4. **Broadcast**

Một message có thể đến nhiều destinations.

## Ví dụ thực tế với `hello_exchange`

### Scenario: Notification System

```rust
// Exchange
exchange_declare("hello_exchange", Fanout)

// Services bind queues của họ:
- Email Service    → queue "email_queue" → bind "hello_exchange"
- SMS Service      → queue "sms_queue"   → bind "hello_exchange"
- Push Service     → queue "push_queue"  → bind "hello_exchange"
- Logger Service   → queue "log_queue"   → bind "hello_exchange"

// Publisher gửi 1 message:
basic_publish("hello_exchange", "", "User registered!")

// Kết quả:
✓ Email Service  → Gửi email chào mừng
✓ SMS Service    → Gửi SMS xác nhận
✓ Push Service   → Gửi push notification
✓ Logger Service → Log event
```

**Tất cả 4 services nhận cùng 1 message** nhờ exchange fanout!

## Tóm tắt vai trò của `hello_exchange`

| Thành phần         | Vai trò                                     |
| ------------------ | ------------------------------------------- |
| **hello_exchange** | Trung tâm phân phối (fanout type)           |
| **Publisher**      | Gửi message VÀO exchange                    |
| **Exchange**       | Broadcast message đến TẤT CẢ queues đã bind |
| **Queues**         | Lưu trữ message cho từng consumer           |
| **Bindings**       | Liên kết queue ↔ exchange                   |
| **Consumers**      | Nhận message từ queue của họ                |

## Khi nào dùng Exchange nào?

| Use Case                  | Exchange Type                   |
| ------------------------- | ------------------------------- |
| Broadcast đến tất cả      | **Fanout**                      |
| Route theo loại chính xác | **Direct**                      |
| Route theo pattern        | **Topic**                       |
| Gửi đến 1 queue cụ thể    | Default Exchange (empty string) |
| Complex routing logic     | **Headers**                     |
