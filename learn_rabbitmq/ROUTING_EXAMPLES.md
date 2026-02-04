# RabbitMQ Routing Examples - Hướng dẫn chi tiết

## 📋 Tổng quan

Project này có **3 loại Routing Patterns** chính:

| Pattern       | Exchange Type | Cách routing             | Use Case                            |
| ------------- | ------------- | ------------------------ | ----------------------------------- |
| **Broadcast** | Fanout        | TẤT CẢ queues nhận       | Notifications, events               |
| **Direct**    | Direct        | Exact routing key match  | Log levels, task types              |
| **Topic**     | Topic         | Pattern matching (\*, #) | Complex routing, flexible filtering |

---

## 🎯 Example 4 & 5: FANOUT - Broadcast Pattern

### Cách hoạt động:

```
Publisher → [Exchange:Fanout] → Queue A
                    ↓
                 Queue B
                    ↓
                 Queue C

→ TẤT CẢ queues đều nhận message
```

### Test:

**Terminal 1-3: Start subscribers**

```rust
// Terminal 1
publish_subscribe_subscriber("subscriber_1").await?;

// Terminal 2
publish_subscribe_subscriber("subscriber_2").await?;

// Terminal 3
publish_subscribe_subscriber("subscriber_3").await?;
```

**Terminal 4: Publish**

```rust
publish_subscribe_publisher().await?;
```

**Kết quả:** CẢ 3 subscribers đều nhận được message! ✅

---

## 🎯 Example 6 & 6b: DIRECT - Exact Routing

### Cách hoạt động:

```
Publisher (routing_key="error") → [Exchange:Direct]
                                        ↓
                Queue bound với "error" ✅ Nhận
                Queue bound với "warning" ✗ KHÔNG nhận
                Queue bound với "error" + "warning" ✅ Nhận
```

### Use Case: Log System

**Scenario:** Hệ thống log với 3 levels: error, warning, info

**Terminal 1: Error Logger (chỉ nhận ERROR)**

```rust
direct_exchange_subscriber(vec!["error"], "error_logger").await?;
```

**Terminal 2: Important Logger (nhận ERROR + WARNING)**

```rust
direct_exchange_subscriber(vec!["error", "warning"], "important_logger").await?;
```

**Terminal 3: All Logger (nhận TẤT CẢ)**

```rust
direct_exchange_subscriber(vec!["error", "warning", "info"], "all_logger").await?;
```

**Terminal 4: Publisher - Gửi ERROR**

```rust
direct_exchange_publisher("error", "Database connection failed!").await?;
```

**Kết quả:**

```
✅ Terminal 1 (error_logger): Nhận
✅ Terminal 2 (important_logger): Nhận
✅ Terminal 3 (all_logger): Nhận
```

**Terminal 4: Publisher - Gửi INFO**

```rust
direct_exchange_publisher("info", "User logged in successfully").await?;
```

**Kết quả:**

```
✗ Terminal 1 (error_logger): KHÔNG nhận
✗ Terminal 2 (important_logger): KHÔNG nhận
✅ Terminal 3 (all_logger): Nhận
```

### Bảng routing:

| Routing Key | error_logger | important_logger | all_logger |
| ----------- | ------------ | ---------------- | ---------- |
| `error`     | ✅           | ✅               | ✅         |
| `warning`   | ❌           | ✅               | ✅         |
| `info`      | ❌           | ❌               | ✅         |

---

## 🎯 Example 7 & 7b: TOPIC - Pattern Matching

### Wildcards:

- `*` = match **chính xác 1 word**
- `#` = match **0 hoặc nhiều words**

### Cách hoạt động:

```
Routing Key: "user.profile.created"

Patterns:
  "user.*.*"        ✅ Match
  "user.#"          ✅ Match
  "*.profile.*"     ✅ Match
  "*.created"       ❌ NO match (3 words, không phải 2)
  "#.created"       ✅ Match
  "#"               ✅ Match all
  "user.profile"    ❌ NO match
  "order.#"         ❌ NO match
```

### Use Case: Event-Driven Architecture

**Events:**

- User: `user.created`, `user.updated`, `user.deleted`
- Order: `order.created`, `order.payment.success`, `order.payment.failed`

**Terminal 1: User Service (chỉ user events)**

```rust
topic_exchange_subscriber("user.*", "user_service").await?;
```

→ Nhận: `user.created`, `user.updated`, `user.deleted` ✅  
→ KHÔNG nhận: `order.*` ❌

**Terminal 2: Audit Logger (tất cả events)**

```rust
topic_exchange_subscriber("#", "audit_logger").await?;
```

→ Nhận: TẤT CẢ events ✅

**Terminal 3: Payment Service (chỉ payment events)**

```rust
topic_exchange_subscriber("order.payment.*", "payment_service").await?;
```

→ Nhận: `order.payment.success`, `order.payment.failed` ✅  
→ KHÔNG nhận: `order.created`, `user.*` ❌

**Terminal 4: Order Service (tất cả order events)**

```rust
topic_exchange_subscriber("order.#", "order_service").await?;
```

→ Nhận: `order.created`, `order.payment.success`, `order.payment.failed` ✅  
→ KHÔNG nhận: `user.*` ❌

**Terminal 5: Notification Service (tất cả "created" events)**

```rust
topic_exchange_subscriber("*.created", "notification_service").await?;
```

→ Nhận: `user.created`, `order.created` ✅  
→ KHÔNG nhận: `user.updated`, `order.payment.success` ❌

**Terminal 6: Publisher - Gửi events**

```rust
// User events
topic_exchange_publisher("user.created", "New user registered").await?;
topic_exchange_publisher("user.updated", "User profile updated").await?;

// Order events
topic_exchange_publisher("order.created", "New order placed").await?;
topic_exchange_publisher("order.payment.success", "Payment completed").await?;
```

### Bảng routing chi tiết:

| Routing Key             | user.\* | #   | order.payment.\* | order.# | \*.created |
| ----------------------- | ------- | --- | ---------------- | ------- | ---------- |
| `user.created`          | ✅      | ✅  | ❌               | ❌      | ✅         |
| `user.updated`          | ✅      | ✅  | ❌               | ❌      | ❌         |
| `user.deleted`          | ✅      | ✅  | ❌               | ❌      | ❌         |
| `order.created`         | ❌      | ✅  | ❌               | ✅      | ✅         |
| `order.payment.success` | ❌      | ✅  | ✅               | ✅      | ❌         |
| `order.payment.failed`  | ❌      | ✅  | ✅               | ✅      | ❌         |

---

## 🆚 So sánh 3 Patterns

### 1. **FANOUT** (Example 4-5)

```rust
exchange_declare("notifications", Fanout)
basic_publish("notifications", "", message)  // Routing key bỏ qua
```

**Đặc điểm:**

- ✅ Đơn giản nhất
- ✅ Broadcast đến TẤT CẢ
- ❌ Không selective
- **Use case:** Notifications, global events

### 2. **DIRECT** (Example 6-6b)

```rust
exchange_declare("logs", Direct)
basic_publish("logs", "error", message)  // Exact match

queue.bind("logs", "error")        // Chỉ nhận "error"
queue.bind("logs", "warning")      // Chỉ nhận "warning"
```

**Đặc điểm:**

- ✅ Exact matching
- ✅ Một queue có thể bind nhiều keys
- ❌ Không flexible như Topic
- **Use case:** Log levels, task types, priority

### 3. **TOPIC** (Example 7-7b)

```rust
exchange_declare("events", Topic)
basic_publish("events", "user.profile.created", message)

queue.bind("events", "user.*")      // Tất cả user events
queue.bind("events", "*.created")   // Tất cả created events
queue.bind("events", "#")           // TẤT CẢ events
```

**Đặc điểm:**

- ✅ Flexible nhất
- ✅ Pattern matching với \* và #
- ❌ Phức tạp hơn
- **Use case:** Event-driven, microservices, complex routing

---

## 🧪 Các Scenarios để Test

### Scenario 1: Log System với Direct Exchange

**Goal:** Các services khác nhau nhận log levels khác nhau

```bash
# Terminal 1: Critical service (chỉ errors)
direct_exchange_subscriber(vec!["error"], "critical_service")

# Terminal 2: Monitoring (errors + warnings)
direct_exchange_subscriber(vec!["error", "warning"], "monitoring")

# Terminal 3: Debug service (all levels)
direct_exchange_subscriber(vec!["error", "warning", "info"], "debug_service")

# Terminal 4: Publish logs
direct_exchange_publisher("error", "Critical error!")
direct_exchange_publisher("warning", "Warning message")
direct_exchange_publisher("info", "Info message")
```

### Scenario 2: Microservices với Topic Exchange

**Goal:** Services nhận events liên quan đến domain của họ

```bash
# Terminal 1: User Service
topic_exchange_subscriber("user.#", "user_service")

# Terminal 2: Order Service
topic_exchange_subscriber("order.#", "order_service")

# Terminal 3: Payment Service
topic_exchange_subscriber("*.payment.*", "payment_service")

# Terminal 4: Notification Service (all created events)
topic_exchange_subscriber("#.created", "notification_service")

# Terminal 5: Audit (everything)
topic_exchange_subscriber("#", "audit_service")

# Terminal 6: Publish events
topic_exchange_publisher("user.created", "New user")
topic_exchange_publisher("order.created", "New order")
topic_exchange_publisher("order.payment.success", "Payment OK")
```

### Scenario 3: Multi-level Pattern Matching

**Pattern Examples:**

```rust
// Simple wildcards
"user.*"              // user.created, user.updated (1 level)
"*.created"           // user.created, order.created (1 level)

// Multi-level
"user.*.*"            // user.profile.created (2 levels)
"order.payment.*"     // order.payment.success

// Hash wildcards
"user.#"              // user.created, user.profile.updated, user.x.y.z
"#.failed"            // payment.failed, order.payment.failed
"#"                   // TẤT CẢ

// Complex
"*.*.created"         // user.profile.created
"order.#.failed"      // order.payment.failed, order.x.y.failed
```

---

## 📝 Lưu ý quan trọng

### 1. Routing Key Format

```rust
// ✅ Valid
"user.created"
"order.payment.success"
"log.error"
"service.user.profile.updated"

// ❌ Invalid (không nên dùng)
"user-created"        // Dùng . không phải -
"UserCreated"         // Dùng lowercase
"user.created.now!"   // Không có ký tự đặc biệt
```

### 2. Binding Keys với Topic Exchange

```rust
// * matches exactly 1 word
"user.*"              // ✅ user.created
                      // ❌ user
                      // ❌ user.profile.created

// # matches 0 or more words
"user.#"              // ✅ user.created
                      // ✅ user.profile.created
                      // ✅ user (0 words)

"#"                   // ✅ Matches EVERYTHING
```

### 3. Performance

- **Fanout**: Nhanh nhất (không cần routing logic)
- **Direct**: Nhanh (hash table lookup)
- **Topic**: Chậm hơn (pattern matching)

---

## 🚀 Quick Start

1. **Fanout (Broadcast):**

```bash
# Terminal 1-3: Subscribers
cargo run  # uncomment: publish_subscribe_subscriber("sub_X")

# Terminal 4: Publisher
cargo run  # uncomment: publish_subscribe_publisher()
```

2. **Direct (Exact Routing):**

```bash
# Terminal 1-3: Subscribers với routing keys khác nhau
cargo run  # uncomment: direct_exchange_subscriber(vec!["error"], ...)

# Terminal 4: Publisher
cargo run  # uncomment: direct_exchange_publisher("error", ...)
```

3. **Topic (Pattern Matching):**

```bash
# Terminal 1-5: Subscribers với patterns khác nhau
cargo run  # uncomment: topic_exchange_subscriber("user.*", ...)

# Terminal 6: Publisher
cargo run  # uncomment: topic_exchange_publisher("user.created", ...)
```

---

## 🎓 Summary

| Cần                          | Dùng       | Ví dụ                                   |
| ---------------------------- | ---------- | --------------------------------------- |
| Broadcast đến tất cả         | **Fanout** | Notifications                           |
| Route theo category cụ thể   | **Direct** | Log levels (error, warning)             |
| Route linh hoạt với patterns | **Topic**  | Microservices events (user.\*, order.#) |

Happy routing! 🎉
