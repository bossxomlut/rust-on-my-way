# RabbitMQ Topic Exchange - Wildcards Patterns

## 🎯 Tổng quan

Topic Exchange sử dụng **wildcards** để routing messages dựa trên **pattern matching**.

### 2 Wildcards chính:

| Wildcard | Ý nghĩa                      | Ví dụ                                                            |
| -------- | ---------------------------- | ---------------------------------------------------------------- |
| `*`      | Match **chính xác 1 word**   | `user.*` match `user.created` ✅                                 |
| `#`      | Match **0 hoặc nhiều words** | `user.#` match `user`, `user.created`, `user.profile.updated` ✅ |

**Word** = chuỗi ký tự phân tách bởi dấu chấm `.`

---

## 📝 Wildcard `*` - Match chính xác 1 word

### Quy tắc:

- `*` thay thế **ĐÚNG 1 word**
- Không nhiều hơn, không ít hơn

### Ví dụ với pattern `user.*`:

| Routing Key            | Match? | Giải thích                   |
| ---------------------- | ------ | ---------------------------- |
| `user.created`         | ✅     | 2 words: "user" + "created"  |
| `user.updated`         | ✅     | 2 words: "user" + "updated"  |
| `user.deleted`         | ✅     | 2 words: "user" + "deleted"  |
| `user`                 | ❌     | Chỉ 1 word, thiếu word thứ 2 |
| `user.profile.created` | ❌     | 3 words, thừa 1 word         |
| `order.created`        | ❌     | Word đầu không phải "user"   |

### Ví dụ với pattern `*.created`:

| Routing Key            | Match? | Giải thích                  |
| ---------------------- | ------ | --------------------------- |
| `user.created`         | ✅     | 2 words, word 2 = "created" |
| `order.created`        | ✅     | 2 words, word 2 = "created" |
| `product.created`      | ✅     | 2 words, word 2 = "created" |
| `created`              | ❌     | Chỉ 1 word, thiếu word đầu  |
| `user.profile.created` | ❌     | 3 words, không phải 2       |
| `user.updated`         | ❌     | Word 2 không phải "created" |

### Ví dụ với pattern `*.*.updated`:

| Routing Key                   | Match? | Giải thích                  |
| ----------------------------- | ------ | --------------------------- |
| `user.profile.updated`        | ✅     | 3 words, word 3 = "updated" |
| `order.status.updated`        | ✅     | 3 words, word 3 = "updated" |
| `user.updated`                | ❌     | Chỉ 2 words, thiếu 1 word   |
| `user.profile.status.updated` | ❌     | 4 words, thừa 1 word        |

### Nhiều `*` trong pattern:

```rust
Pattern: "*.*.created"
✅ user.profile.created    (3 words)
✅ order.item.created      (3 words)
❌ user.created            (2 words - thiếu)
❌ user.profile.item.created (4 words - thừa)

Pattern: "user.*.*.updated"
✅ user.profile.settings.updated   (4 words)
✅ user.account.email.updated      (4 words)
❌ user.profile.updated            (3 words - thiếu)
❌ user.updated                    (2 words - thiếu nhiều)
```

---

## 📝 Wildcard `#` - Match 0 hoặc nhiều words

### Quy tắc:

- `#` thay thế **0, 1, 2, 3, ... nhiều words**
- Rất linh hoạt!

### Ví dụ với pattern `user.#`:

| Routing Key                     | Match? | Giải thích                               |
| ------------------------------- | ------ | ---------------------------------------- |
| `user`                          | ✅     | 1 word (# = 0 words)                     |
| `user.created`                  | ✅     | 2 words (# = 1 word: "created")          |
| `user.profile.updated`          | ✅     | 3 words (# = 2 words: "profile.updated") |
| `user.profile.settings.changed` | ✅     | 4 words (# = 3 words)                    |
| `user.a.b.c.d.e.f`              | ✅     | Bất kỳ số words nào sau "user"           |
| `order.created`                 | ❌     | Word đầu không phải "user"               |

### Ví dụ với pattern `#.created`:

| Routing Key            | Match? | Giải thích                     |
| ---------------------- | ------ | ------------------------------ |
| `created`              | ✅     | 1 word (# = 0 words)           |
| `user.created`         | ✅     | 2 words (# = 1 word)           |
| `order.created`        | ✅     | 2 words (# = 1 word)           |
| `user.profile.created` | ✅     | 3 words (# = 2 words)          |
| `a.b.c.created`        | ✅     | 4 words (# = 3 words)          |
| `user.updated`         | ❌     | Word cuối không phải "created" |

### Ví dụ với pattern `#`:

| Routing Key             | Match? | Giải thích              |
| ----------------------- | ------ | ----------------------- |
| `user`                  | ✅     | Bất kỳ                  |
| `user.created`          | ✅     | Bất kỳ                  |
| `order.payment.success` | ✅     | Bất kỳ                  |
| `a.b.c.d.e.f.g`         | ✅     | **TẤT CẢ** routing keys |

> ⚠️ Pattern `#` = **match TẤT CẢ messages** (giống Fanout Exchange)

---

## 🔄 Kết hợp `*` và `#`

### Pattern: `user.*.#`

| Routing Key                     | Match? | Giải thích                                 |
| ------------------------------- | ------ | ------------------------------------------ |
| `user.profile`                  | ✅     | user + 1 word (profile) + 0 words          |
| `user.profile.updated`          | ✅     | user + 1 word (profile) + 1 word (updated) |
| `user.profile.settings.changed` | ✅     | user + 1 word (profile) + 2 words          |
| `user`                          | ❌     | Thiếu 1 word sau "user" (vì có \*)         |
| `order.created`                 | ❌     | Không bắt đầu bằng "user"                  |

### Pattern: `#.payment.*`

| Routing Key                  | Match? | Giải thích                                  |
| ---------------------------- | ------ | ------------------------------------------- |
| `payment.success`            | ✅     | 0 words + payment + 1 word (success)        |
| `payment.failed`             | ✅     | 0 words + payment + 1 word (failed)         |
| `order.payment.success`      | ✅     | 1 word (order) + payment + 1 word (success) |
| `user.order.payment.success` | ✅     | 2 words + payment + 1 word                  |
| `payment`                    | ❌     | Thiếu 1 word sau "payment"                  |
| `order.payment`              | ❌     | Thiếu 1 word sau "payment"                  |
| `payment.credit.success`     | ❌     | Thừa word giữa "payment" và word cuối       |

### Pattern: `*.payment.#`

| Routing Key                   | Match? | Giải thích                   |
| ----------------------------- | ------ | ---------------------------- |
| `order.payment`               | ✅     | 1 word + payment + 0 words   |
| `order.payment.success`       | ✅     | 1 word + payment + 1 word    |
| `user.payment.credit.success` | ✅     | 1 word + payment + 2 words   |
| `payment.success`             | ❌     | Thiếu 1 word trước "payment" |
| `order.user.payment.success`  | ❌     | Thừa 1 word trước "payment"  |

---

## 🎓 Ví dụ thực tế

### Scenario 1: Event-Driven Microservices

**Events:**

```
user.created
user.updated
user.deleted
user.profile.created
user.profile.updated
user.profile.avatar.changed
order.created
order.updated
order.payment.pending
order.payment.success
order.payment.failed
order.shipping.dispatched
```

**Subscribers với patterns:**

#### 1. User Service (tất cả user events)

```rust
topic_exchange_subscriber("user.#", "user_service")
```

**Nhận:**

- ✅ `user.created`
- ✅ `user.updated`
- ✅ `user.deleted`
- ✅ `user.profile.created`
- ✅ `user.profile.updated`
- ✅ `user.profile.avatar.changed`
- ❌ `order.*` (không phải user)

#### 2. Order Service (tất cả order events)

```rust
topic_exchange_subscriber("order.#", "order_service")
```

**Nhận:**

- ✅ Tất cả order events
- ❌ user events

#### 3. Payment Service (chỉ payment events - level 3)

```rust
topic_exchange_subscriber("order.payment.*", "payment_service")
```

**Nhận:**

- ✅ `order.payment.pending`
- ✅ `order.payment.success`
- ✅ `order.payment.failed`
- ❌ `order.created` (không phải payment)
- ❌ `order.shipping.dispatched` (không phải payment)

#### 4. Notification Service (tất cả "created" events)

```rust
topic_exchange_subscriber("*.created", "notification_service")
```

**Nhận:**

- ✅ `user.created`
- ✅ `order.created`
- ❌ `user.profile.created` (3 words, không phải 2)

#### 5. Advanced Notification (tất cả created ở mọi level)

```rust
topic_exchange_subscriber("#.created", "advanced_notification")
```

**Nhận:**

- ✅ `user.created`
- ✅ `order.created`
- ✅ `user.profile.created`
- ✅ Bất kỳ _._.\*.created

#### 6. Audit Logger (TẤT CẢ events)

```rust
topic_exchange_subscriber("#", "audit_logger")
```

**Nhận:**

- ✅ **MỌI** events

---

## 📊 Bảng so sánh các patterns

| Pattern       | user.created | user.profile.updated | order.payment.success | payment.failed |
| ------------- | ------------ | -------------------- | --------------------- | -------------- |
| `user.*`      | ✅           | ❌                   | ❌                    | ❌             |
| `user.#`      | ✅           | ✅                   | ❌                    | ❌             |
| `*.created`   | ✅           | ❌                   | ❌                    | ❌             |
| `#.created`   | ✅           | ❌                   | ❌                    | ❌             |
| `*.*.updated` | ❌           | ✅                   | ❌                    | ❌             |
| `#.success`   | ❌           | ❌                   | ✅                    | ❌             |
| `order.#`     | ❌           | ❌                   | ✅                    | ❌             |
| `*.payment.*` | ❌           | ❌                   | ✅                    | ❌             |
| `#.payment.#` | ❌           | ❌                   | ✅                    | ❌             |
| `payment.*`   | ❌           | ❌                   | ❌                    | ✅             |
| `#`           | ✅           | ✅                   | ✅                    | ✅             |

---

## 🧪 Test Cases để hiểu rõ

### Test 1: `*` chỉ match ĐÚNG 1 word

```rust
Pattern: "user.*"

✅ user.created
✅ user.updated
✅ user.deleted
❌ user                    // 0 words sau "user"
❌ user.profile.updated    // 2 words sau "user"
```

### Test 2: `#` match 0 hoặc nhiều words

```rust
Pattern: "user.#"

✅ user                    // 0 words
✅ user.created            // 1 word
✅ user.profile.updated    // 2 words
✅ user.a.b.c.d            // 4 words
✅ user.x.y.z.a.b.c        // 6 words
```

### Test 3: Kết hợp `*` và `#`

```rust
Pattern: "order.*.#"

✅ order.created           // order + 1 word + 0 words
✅ order.payment.success   // order + 1 word (payment) + 1 word (success)
✅ order.shipping.tracking.updated  // order + 1 word + 2 words
❌ order                   // Thiếu 1 word sau "order"
❌ order.payment           // Cần ít nhất 2 words sau "order" (có thể 0 words sau payment)

Chờ... order.payment có ✅ đúng không?
→ ✅ ĐÚNG! order + 1 word (payment) + 0 words

Pattern breakdown:
- "order" = exact match
- ".*" = 1 word (payment)
- ".#" = 0+ words (có thể không có gì)
```

### Test 4: Multiple `#`

```rust
Pattern: "#.payment.#"

✅ payment                 // 0 + payment + 0
✅ payment.success         // 0 + payment + 1
✅ order.payment           // 1 + payment + 0
✅ order.payment.success   // 1 + payment + 1
✅ user.order.payment.credit.success  // 2 + payment + 2
✅ a.b.c.payment.x.y.z     // 3 + payment + 3
```

### Test 5: Edge Cases

```rust
Pattern: "*"
✅ user                    // Chỉ 1 word
✅ order
✅ payment
❌ user.created            // 2 words
❌ a.b.c                   // 3 words

Pattern: "*.*"
❌ user                    // Chỉ 1 word
✅ user.created            // 2 words
✅ order.updated
❌ user.profile.updated    // 3 words

Pattern: "*.#"
✅ user                    // 1 word + 0
✅ user.created            // 1 word + 1
✅ user.profile.updated    // 1 word + 2
❌ (empty)                 // Cần ít nhất 1 word

Pattern: "#.*"
✅ user                    // 0 + 1
✅ user.created            // 1 + 1
✅ user.profile.updated    // 2 + 1
❌ (empty)                 // Cần ít nhất 1 word
```

---

## 🎯 Quy tắc vàng

### 1. `*` = EXACTLY 1 word

```
user.*           → user.[1 word]
*.created        → [1 word].created
user.*.updated   → user.[1 word].updated
*.*.*            → [1].[1].[1] = đúng 3 words
```

### 2. `#` = 0 OR MORE words

```
user.#           → user.[0+ words]
#.created        → [0+ words].created
order.#.success  → order.[0+ words].success
#                → [0+ words] = TẤT CẢ
```

### 3. Kết hợp: Đếm words!

```
user.*.#         → user + 1 word + 0+ words = ≥ 2 words
#.payment.*      → 0+ words + payment + 1 word = ≥ 2 words
*.#.updated      → 1 word + 0+ words + updated = ≥ 2 words
```

---

## 🚫 Lỗi thường gặp

### ❌ Lỗi 1: Hiểu sai `#` nghĩa là "anything"

```rust
Pattern: "user.#.created"

// Tưởng:
user.created  ✅  // SAI! Cần: user + [0+ words] + created
                  // "created" là word riêng biệt, không phải part của #

// Thực tế:
user.profile.created  ✅  // user + 1 word + created
user.created          ❌  // Thiếu word giữa user và created
```

**Sửa:** Nếu muốn match cả `user.created`, dùng 2 patterns:

```rust
"user.created"  // Exact
"user.#.created"  // With words in between
```

Hoặc dùng:

```rust
"user.#"  // Match tất cả user events
```

### ❌ Lỗi 2: Nhầm `*` có thể là 0 words

```rust
Pattern: "user.*"

user          ❌  // * cần ĐÚNG 1 word
user.created  ✅
```

**Sửa:** Nếu muốn match cả `user`, dùng:

```rust
"user.#"  // Match user và user.[anything]
```

### ❌ Lỗi 3: Nghĩ `*.*` match "anything với 1 dấu chấm"

```rust
Pattern: "*.*"

user.created              ✅  // 2 words
user.profile.updated      ❌  // 3 words (không phải 2!)
```

---

## 💡 Tips & Best Practices

### 1. Bắt đầu đơn giản

```rust
// ✅ Tốt: Dễ hiểu
"user.#"          // Tất cả user events
"order.#"         // Tất cả order events

// ❌ Tránh: Phức tạp không cần thiết
"#.user.#.order.#"
```

### 2. Sử dụng naming convention

```rust
// ✅ Tốt: Consistent structure
entity.action
entity.subentity.action
entity.subentity.field.action

// Ví dụ:
user.created
user.profile.updated
user.profile.avatar.changed
order.payment.success
```

### 3. Document patterns của bạn

```rust
// ✅ Tốt
// Subscribe to all user events (user.*)
topic_exchange_subscriber("user.#", "user_service")

// Subscribe to all created events (*.created, *.*.created)
topic_exchange_subscriber("#.created", "notification_service")
```

### 4. Test patterns trước khi deploy

```rust
// Viết test cases
assert!(matches("user.created", "user.*"));
assert!(matches("user.created", "user.#"));
assert!(!matches("user.created", "order.*"));
```

---

## 🎓 Quiz

### Quiz 1: Pattern `user.*.updated`

Routing keys nào match?

1. `user.updated`
2. `user.profile.updated`
3. `user.account.email.updated`
4. `admin.user.profile.updated`

<details>
<summary>Đáp án</summary>

✅ **2. `user.profile.updated`** - Đúng 3 words: user + profile + updated

❌ 1. `user.updated` - Chỉ 2 words  
❌ 3. `user.account.email.updated` - 4 words  
❌ 4. `admin.user.profile.updated` - Không bắt đầu bằng "user"

</details>

### Quiz 2: Pattern `#.payment.#`

Routing keys nào match?

1. `payment`
2. `payment.success`
3. `order.payment`
4. `order.payment.success`
5. `user.order.payment.credit.success`

<details>
<summary>Đáp án</summary>

✅ **TẤT CẢ đều match!**

1. `payment` → 0 + payment + 0 ✅
2. `payment.success` → 0 + payment + 1 ✅
3. `order.payment` → 1 + payment + 0 ✅
4. `order.payment.success` → 1 + payment + 1 ✅
5. `user.order.payment.credit.success` → 2 + payment + 2 ✅
</details>

### Quiz 3: Tìm pattern match `user.created` và `user.profile.created`

Patterns nào match CẢ HAI?

1. `user.*`
2. `user.#`
3. `#.created`
4. `*.created`

<details>
<summary>Đáp án</summary>

✅ **2. `user.#`** - Match user.created (1 word) và user.profile.created (2 words)  
✅ **3. `#.created`** - Match bất kỳ \*.created

❌ 1. `user.*` - Chỉ match user.created (2 words), không match user.profile.created (3 words)  
❌ 4. `*.created` - Chỉ match user.created (2 words), không match user.profile.created (3 words)

</details>

---

## 🎉 Tổng kết

| Muốn                            | Pattern            | Ví dụ             |
| ------------------------------- | ------------------ | ----------------- |
| Match đúng 1 word sau prefix    | `prefix.*`         | `user.*`          |
| Match bất kỳ words sau prefix   | `prefix.#`         | `user.#`          |
| Match đúng 1 word trước suffix  | `*.suffix`         | `*.created`       |
| Match bất kỳ words trước suffix | `#.suffix`         | `#.created`       |
| Match TẤT CẢ                    | `#`                | `#`               |
| Match chính xác N words         | `*.*.*...` (N lần) | `*.*.*` (3 words) |

**Remember:**

- `*` = Exactly ONE word
- `#` = ZERO or MORE words
- Combine them for flexible routing!
