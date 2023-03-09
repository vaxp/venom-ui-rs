# 🎯 VenomUI State Management

دليل شامل لنظام إدارة الحالة في VenomUI، مستوحى من Flutter's BLoC Pattern.

---

## 📋 جدول المحتويات

1. [مقدمة](#مقدمة)
2. [المفاهيم الأساسية](#المفاهيم-الأساسية)
3. [الـ Cubit - البداية السريعة](#الـ-cubit---البداية-السريعة)
4. [الـ Bloc - إدارة الأحداث](#الـ-bloc---إدارة-الأحداث)
5. [الـ Selectors - الحالة المشتقة](#الـ-selectors---الحالة-المشتقة)
6. [الـ Subscriptions - الاشتراكات](#الـ-subscriptions---الاشتراكات)
7. [Thread-Safe State - الحالة الآمنة للخيوط](#thread-safe-state---الحالة-الآمنة-للخيوط)
8. [Widget Integration - دمج الـ Widgets](#widget-integration---دمج-الـ-widgets)
9. [DevTools - أدوات التطوير](#devtools---أدوات-التطوير)
10. [Async Operations - العمليات غير المتزامنة](#async-operations---العمليات-غير-المتزامنة)
11. [Hydration - حفظ الحالة](#hydration---حفظ-الحالة)
12. [أفضل الممارسات](#أفضل-الممارسات)
13. [أمثلة عملية](#أمثلة-عملية)

---

## مقدمة

VenomUI State Management هو نظام قوي ومرن لإدارة حالة التطبيق، يوفر:

- **بساطة**: API سهل ومباشر
- **مرونة**: من التطبيقات الصغيرة إلى الكبيرة
- **أداء**: تحسينات للأداء مع الـ memoization
- **أمان**: دعم كامل للـ thread safety
- **debugging**: أدوات متقدمة للتتبع والتصحيح

### متى تستخدم كل نوع؟

| النوع | الاستخدام | المثال |
|-------|----------|--------|
| `CubitCore` | حالة بسيطة، عمليات متزامنة | Counter, Toggle |
| `BlocCore` | حالة معقدة، أحداث متعددة | Auth, Cart |
| `SyncCubitCore` | multi-threading, Widget integration | UI State |
| `Selector` | حالة مشتقة، تحسين الأداء | Filtered List |

---

## المفاهيم الأساسية

### 1. State (الحالة)

الحالة هي البيانات التي تصف تطبيقك في لحظة معينة:

```rust
#[derive(Clone, PartialEq, Debug)]
struct AppState {
    user: Option<User>,
    items: Vec<Item>,
    is_loading: bool,
}
```

> **متطلبات الحالة:**
> - `Clone` - للنسخ
> - `PartialEq` - للمقارنة (هل تغيرت؟)

### 2. Cubit

حاوية للحالة مع دوال لتحديثها:

```rust
use venom_core::CubitCore;

let counter = CubitCore::new(0);
counter.emit(1);  // تحديث الحالة
println!("{}", counter.state());  // 1
```

### 3. Bloc

مثل Cubit لكن مع event-driven pattern:

```rust
use venom_core::BlocCore;

enum CounterEvent { Increment, Decrement }

let bloc = BlocCore::new(0, |event, state, emit| {
    match event {
        CounterEvent::Increment => emit(state + 1),
        CounterEvent::Decrement => emit(state - 1),
    }
});

bloc.add(CounterEvent::Increment);
```

### 4. Subscription

الاشتراك في تغييرات الحالة:

```rust
let counter = CubitCore::new(0);

let _sub = counter.subscribe(|old, new| {
    println!("Changed: {} -> {}", old, new);
});

counter.emit(1);  // يطبع: Changed: 0 -> 1
```

---

## الـ Cubit - البداية السريعة

### إنشاء Cubit بسيط

```rust
use venom_core::{CubitCore, Cubit};

// الطريقة الأولى: CubitCore مباشرة
let counter = CubitCore::new(0);
counter.emit(counter.state() + 1);

// الطريقة الثانية: SimpleCubit
use venom_core::SimpleCubit;
let counter = SimpleCubit::new(0);
```

### إنشاء Cubit مخصص

```rust
use venom_core::{CubitCore, Cubit};

struct CounterCubit {
    core: CubitCore<i32>,
}

impl CounterCubit {
    fn new() -> Self {
        Self { core: CubitCore::new(0) }
    }

    fn increment(&self) {
        self.core.emit(self.core.state() + 1);
    }

    fn decrement(&self) {
        self.core.emit(self.core.state() - 1);
    }

    fn reset(&self) {
        self.core.emit(0);
    }

    fn state(&self) -> i32 {
        self.core.state()
    }
}

// الاستخدام
let counter = CounterCubit::new();
counter.increment();
counter.increment();
println!("Count: {}", counter.state());  // 2
```

### الـ Update Pattern

```rust
let counter = CubitCore::new(0);

// بدلاً من
counter.emit(counter.state() + 1);

// استخدم update
counter.update(|n| n + 1);
counter.update(|n| n * 2);
```

### حالة معقدة

```rust
#[derive(Clone, PartialEq, Debug)]
struct TodoState {
    items: Vec<Todo>,
    filter: Filter,
}

#[derive(Clone, PartialEq, Debug)]
struct Todo {
    id: u32,
    text: String,
    completed: bool,
}

#[derive(Clone, PartialEq, Debug)]
enum Filter { All, Active, Completed }

struct TodoCubit {
    core: CubitCore<TodoState>,
}

impl TodoCubit {
    fn new() -> Self {
        Self {
            core: CubitCore::new(TodoState {
                items: vec![],
                filter: Filter::All,
            })
        }
    }

    fn add_todo(&self, text: &str) {
        self.core.update(|state| {
            let mut new_state = state.clone();
            new_state.items.push(Todo {
                id: new_state.items.len() as u32,
                text: text.to_string(),
                completed: false,
            });
            new_state
        });
    }

    fn toggle_todo(&self, id: u32) {
        self.core.update(|state| {
            let mut new_state = state.clone();
            if let Some(todo) = new_state.items.iter_mut().find(|t| t.id == id) {
                todo.completed = !todo.completed;
            }
            new_state
        });
    }

    fn set_filter(&self, filter: Filter) {
        self.core.update(|state| {
            let mut new_state = state.clone();
            new_state.filter = filter;
            new_state
        });
    }
}
```

---

## الـ Bloc - إدارة الأحداث

### متى تستخدم Bloc بدلاً من Cubit؟

- عندما يكون لديك **أحداث متعددة** تؤثر على الحالة
- عندما تحتاج **تحويل الأحداث** (debounce, throttle)
- عندما تريد **فصل المنطق** عن الـ UI بشكل أوضح

### إنشاء Bloc

```rust
use venom_core::BlocCore;

// تعريف الأحداث
#[derive(Clone)]
enum AuthEvent {
    Login { email: String, password: String },
    Logout,
    CheckSession,
}

// تعريف الحالة
#[derive(Clone, PartialEq, Debug)]
enum AuthState {
    Initial,
    Loading,
    Authenticated(User),
    Unauthenticated,
    Error(String),
}

#[derive(Clone, PartialEq, Debug)]
struct User {
    id: u32,
    name: String,
}

// إنشاء الـ Bloc
let auth_bloc = BlocCore::new(AuthState::Initial, |event, state, emit| {
    match event {
        AuthEvent::Login { email, password } => {
            emit(AuthState::Loading);
            
            // في الواقع: اتصال بالـ API
            if email == "admin" && password == "1234" {
                emit(AuthState::Authenticated(User {
                    id: 1,
                    name: "Admin".to_string(),
                }));
            } else {
                emit(AuthState::Error("Invalid credentials".to_string()));
            }
        }
        AuthEvent::Logout => {
            emit(AuthState::Unauthenticated);
        }
        AuthEvent::CheckSession => {
            // التحقق من الجلسة
            emit(AuthState::Unauthenticated);
        }
    }
});

// الاستخدام
auth_bloc.add(AuthEvent::Login {
    email: "admin".to_string(),
    password: "1234".to_string(),
});
```

### StatefulBloc مع حالة مخصصة

```rust
use venom_core::StatefulBloc;

struct CartBloc {
    core: StatefulBloc<CartEvent, CartState>,
}

#[derive(Clone)]
enum CartEvent {
    AddItem(Product),
    RemoveItem(u32),
    UpdateQuantity(u32, i32),
    Clear,
}

#[derive(Clone, PartialEq, Debug)]
struct CartState {
    items: Vec<CartItem>,
}

#[derive(Clone, PartialEq, Debug)]
struct CartItem {
    product: Product,
    quantity: i32,
}

impl CartBloc {
    fn new() -> Self {
        Self {
            core: StatefulBloc::new(CartState { items: vec![] }, |event, state, emit| {
                match event {
                    CartEvent::AddItem(product) => {
                        let mut new_state = state.clone();
                        if let Some(item) = new_state.items.iter_mut()
                            .find(|i| i.product.id == product.id) {
                            item.quantity += 1;
                        } else {
                            new_state.items.push(CartItem {
                                product,
                                quantity: 1,
                            });
                        }
                        emit(new_state);
                    }
                    CartEvent::RemoveItem(id) => {
                        let mut new_state = state.clone();
                        new_state.items.retain(|i| i.product.id != id);
                        emit(new_state);
                    }
                    CartEvent::Clear => {
                        emit(CartState { items: vec![] });
                    }
                    _ => {}
                }
            }),
        }
    }

    fn add(&self, event: CartEvent) {
        self.core.add(event);
    }

    fn state(&self) -> CartState {
        self.core.state()
    }

    fn total(&self) -> f64 {
        self.state().items.iter()
            .map(|item| item.product.price * item.quantity as f64)
            .sum()
    }
}
```

---

## الـ Selectors - الحالة المشتقة

الـ Selectors تحسب قيم مشتقة من الحالة مع **memoization**:

### Selector بسيط

```rust
use venom_core::Selector;

let counter = CubitCore::new(0);

// Selector يحسب الضعف
let doubled = Selector::new(&counter, |n| n * 2);

counter.emit(5);
assert_eq!(doubled.value(), 10);

// الـ Selector لا يُعاد حسابه إلا عند تغير الحالة
```

### Selector مع مقارنة مخصصة

```rust
use venom_core::SelectorEq;

#[derive(Clone, PartialEq)]
struct AppState {
    items: Vec<Item>,
    selected_id: Option<u32>,
}

let state = CubitCore::new(AppState { items: vec![], selected_id: None });

// Selector يعيد فقط الـ items المحددة
let selected_item = SelectorEq::new(&state, |s| {
    s.selected_id.and_then(|id| {
        s.items.iter().find(|item| item.id == id).cloned()
    })
});
```

### Selector2 و Selector3 - الجمع بين مصادر

```rust
use venom_core::{Selector2, Selector3};

let user = CubitCore::new(None::<User>);
let cart = CubitCore::new(CartState { items: vec![] });

// Selector يجمع بين الـ user والـ cart
let can_checkout = Selector2::new(&user, &cart, |user, cart| {
    user.is_some() && !cart.items.is_empty()
});

// ثلاثة مصادر
let price = CubitCore::new(100.0);
let discount = CubitCore::new(0.1);
let tax = CubitCore::new(0.15);

let final_price = Selector3::new(&price, &discount, &tax, |p, d, t| {
    let discounted = p * (1.0 - d);
    discounted * (1.0 + t)
});
```

---

## الـ Subscriptions - الاشتراكات

### الاشتراك الأساسي

```rust
use venom_core::CubitCore;

let counter = CubitCore::new(0);

// الاشتراك يعود handle - عند drop يتم الإلغاء تلقائياً
let handle = counter.subscribe(|old, new| {
    println!("Changed from {} to {}", old, new);
});

counter.emit(1);  // يطبع
counter.emit(2);  // يطبع

drop(handle);     // إلغاء الاشتراك

counter.emit(3);  // لا يطبع
```

### الاشتراك في القيمة الجديدة فقط

```rust
let counter = CubitCore::new(0);

let _handle = counter.subscribe_new(|new| {
    println!("New value: {}", new);
});
```

### الاشتراك مع شرط

```rust
let counter = CubitCore::new(0);

// فقط عندما تتجاوز القيمة 10
let _handle = counter.subscribe(|old, new| {
    if *new > 10 && *old <= 10 {
        println!("Crossed threshold!");
    }
});
```

### إدارة متعددة اشتراكات

```rust
use venom_core::SubscriptionHandle;

struct MyWidget {
    subscriptions: Vec<SubscriptionHandle>,
}

impl MyWidget {
    fn subscribe_to(&mut self, counter: &CubitCore<i32>) {
        let handle = counter.subscribe_new(|value| {
            // تحديث الـ UI
        });
        self.subscriptions.push(handle);
    }
}

impl Drop for MyWidget {
    fn drop(&mut self) {
        // الاشتراكات تُلغى تلقائياً عند drop
    }
}
```

---

## Thread-Safe State - الحالة الآمنة للخيوط

عندما تحتاج مشاركة الحالة بين threads أو دمجها مع الـ Widget system:

### SyncCubitCore

```rust
use venom_core::SyncCubitCore;
use std::thread;

let counter = SyncCubitCore::new(0);
let counter_clone = counter.clone();

// آمن للاستخدام في thread آخر
let handle = thread::spawn(move || {
    for _ in 0..100 {
        counter_clone.update(|n| n + 1);
    }
});

handle.join().unwrap();
println!("Final: {}", counter.state());  // 100
```

### SyncBloc

```rust
use venom_core::SyncBloc;

#[derive(Clone)]
enum Event { Increment, Decrement }

let bloc = SyncBloc::new(0, |event, state, emit| {
    match event {
        Event::Increment => emit(state + 1),
        Event::Decrement => emit(state - 1),
    }
});

// آمن للاستخدام من أي thread
let bloc_clone = bloc.clone();
thread::spawn(move || {
    bloc_clone.add(Event::Increment);
});
```

### متى تستخدم Sync vs Regular؟

| النوع | الأداء | Thread-Safe | استخدام |
|-------|-------|-------------|---------|
| `CubitCore` | ⚡ أسرع | ❌ | Single-thread logic |
| `SyncCubitCore` | معتدل | ✅ | Multi-thread, Widgets |

---

## Widget Integration - دمج الـ Widgets

### SyncBlocBuilder

يعيد بناء الـ widget عند تغير الحالة:

```rust
use venom_widgets::{SyncBlocBuilder, Container, Text};
use venom_core::SyncCubitCore;

let counter = SyncCubitCore::new(0);

let widget = SyncBlocBuilder::new(counter.clone(), |state| {
    Box::new(Text::new(format!("Count: {}", state)))
});
```

### SyncBlocListener

ينفذ side effects عند تغير الحالة:

```rust
use venom_widgets::{SyncBlocListener, Container};
use venom_core::SyncCubitCore;

#[derive(Clone, PartialEq)]
enum AuthState { Initial, Authenticated, Error(String) }

let auth = SyncCubitCore::new(AuthState::Initial);

let widget = SyncBlocListener::new(auth, Container::new())
    .listener(|old, new| {
        match new {
            AuthState::Authenticated => {
                // Navigate to home
            }
            AuthState::Error(msg) => {
                // Show error dialog
            }
            _ => {}
        }
    });
```

### SyncBlocConsumer

يجمع بين Builder و Listener:

```rust
use venom_widgets::SyncBlocConsumer;

let widget = SyncBlocConsumer::new(counter.clone())
    .listener(|old, new| {
        println!("Changed: {} -> {}", old, new);
    })
    .builder(|state| {
        Box::new(Text::new(format!("Count: {}", state)))
    });
```

### SyncBlocSelector

يعيد البناء فقط عند تغير جزء معين من الحالة:

```rust
use venom_widgets::SyncBlocSelector;

#[derive(Clone, PartialEq)]
struct AppState {
    count: i32,
    name: String,
}

let state = SyncCubitCore::new(AppState { count: 0, name: "".into() });

// يعيد البناء فقط عند تغير count
let widget = SyncBlocSelector::new(
    state.clone(),
    |s| s.count,            // selector
    |count| {               // builder
        Box::new(Text::new(format!("Count: {}", count)))
    }
);
```

---

## DevTools - أدوات التطوير

### تفعيل الـ Logging

```rust
use venom_core::{DevTools, ConsoleObserver};

// تفعيل الـ logging في بداية التطبيق
DevTools::set_observer(ConsoleObserver::new());

// الآن جميع التغييرات تُسجل
let counter = CubitCore::named("counter", 0);
counter.emit(1);  // Logs: [counter] 0 -> 1
```

### DebuggableCubit مع Undo/Redo

```rust
use venom_core::DebuggableCubit;

let counter = DebuggableCubit::new("counter", 0);

counter.emit(1);
counter.emit(2);
counter.emit(3);

// Undo
counter.undo();  // الآن 2
counter.undo();  // الآن 1

// Redo
counter.redo();  // الآن 2

// تاريخ الحالة
for entry in counter.history().entries() {
    println!("{:?} at {:?}", entry.state, entry.timestamp);
}
```

### Custom Observer

```rust
use venom_core::BlocObserver;

struct AnalyticsObserver;

impl BlocObserver for AnalyticsObserver {
    fn on_create(&self, name: Option<&str>) {
        // Send to analytics
    }

    fn on_change(&self, name: Option<&str>, change: &str) {
        // Log state change
    }

    fn on_event(&self, name: Option<&str>, event: &str) {
        // Track event
    }

    fn on_error(&self, name: Option<&str>, error: &str) {
        // Report error
    }

    fn on_close(&self, name: Option<&str>) {
        // Cleanup
    }
}

DevTools::set_observer(AnalyticsObserver);
```

---

## Async Operations - العمليات غير المتزامنة

### AsyncCubit

```rust
use venom_core::AsyncCubit;

#[derive(Clone, PartialEq)]
enum LoadState<T> {
    Initial,
    Loading,
    Loaded(T),
    Error(String),
}

let data_cubit = AsyncCubit::new(LoadState::<Vec<User>>::Initial);

data_cubit.execute(|emit| {
    emit(LoadState::Loading);
    
    // Simulate API call
    match fetch_users() {
        Ok(users) => emit(LoadState::Loaded(users)),
        Err(e) => emit(LoadState::Error(e.to_string())),
    }
});
```

### ThreadedBloc للعمليات الثقيلة

```rust
use venom_core::ThreadedBloc;

#[derive(Clone)]
enum ProcessEvent {
    Start(Vec<u8>),
    Cancel,
}

let processor = ThreadedBloc::new(ProcessState::Idle, |event, state, emit| {
    match event {
        ProcessEvent::Start(data) => {
            emit(ProcessState::Processing(0));
            
            // Heavy computation on background thread
            for (i, chunk) in data.chunks(1000).enumerate() {
                process_chunk(chunk);
                emit(ProcessState::Processing((i * 100) / data.len()));
            }
            
            emit(ProcessState::Done);
        }
        ProcessEvent::Cancel => {
            emit(ProcessState::Idle);
        }
    }
});
```

### DebouncedCubit للـ Search

```rust
use venom_core::DebouncedCubit;
use std::time::Duration;

let search = DebouncedCubit::new("".to_string(), Duration::from_millis(300));

// الكتابة السريعة لا تُصدر كل حرف
search.emit("h".into());
search.emit("he".into());
search.emit("hel".into());
search.emit("hell".into());
search.emit("hello".into());

// فقط "hello" يُصدر بعد 300ms
```

---

## Hydration - حفظ الحالة

### الإعداد

```rust
use venom_core::{HydratedCubit, Hydratable, MemoryStorage, FileStorage};

// 1. تنفيذ Hydratable للحالة
#[derive(Clone, PartialEq)]
struct Settings {
    theme: String,
    language: String,
    notifications: bool,
}

impl Hydratable for Settings {
    fn dehydrate(&self) -> Vec<u8> {
        // التحويل إلى bytes
        let mut data = Vec::new();
        data.push(if self.notifications { 1 } else { 0 });
        data.push(self.language.len() as u8);
        data.extend(self.language.as_bytes());
        data.extend(self.theme.as_bytes());
        data
    }

    fn hydrate(data: &[u8]) -> Option<Self> {
        if data.len() < 2 { return None; }
        
        let notifications = data[0] != 0;
        let lang_len = data[1] as usize;
        
        if data.len() < 2 + lang_len { return None; }
        
        let language = String::from_utf8(data[2..2+lang_len].to_vec()).ok()?;
        let theme = String::from_utf8(data[2+lang_len..].to_vec()).ok()?;
        
        Some(Self { theme, language, notifications })
    }
}
```

### الاستخدام

```rust
// Memory Storage (للاختبار)
let storage = MemoryStorage::new();

// File Storage (للإنتاج)
let storage = FileStorage::new("./app_state");

// إنشاء cubit مع حفظ تلقائي
let settings = HydratedCubit::auto_persisted(
    "settings",
    storage,
    Settings {
        theme: "light".into(),
        language: "ar".into(),
        notifications: true,
    }
);

// كل تغيير يُحفظ تلقائياً
settings.emit(Settings {
    theme: "dark".into(),
    ..settings.state()
});
```

### Storage مخصص

```rust
use venom_core::{HydrationStorage, HydrationError};

struct CloudStorage {
    api_key: String,
}

impl HydrationStorage for CloudStorage {
    fn save(&self, key: &str, data: &[u8]) -> Result<(), HydrationError> {
        // Save to cloud
        Ok(())
    }

    fn load(&self, key: &str) -> Result<Vec<u8>, HydrationError> {
        // Load from cloud
        Err(HydrationError::NotFound(key.into()))
    }

    fn delete(&self, key: &str) -> Result<(), HydrationError> {
        Ok(())
    }

    fn exists(&self, key: &str) -> bool {
        false
    }
}
```

---

## أفضل الممارسات

### 1. فصل الـ State عن الـ Logic

```rust
// ❌ خطأ: Logic داخل الـ emit
counter.emit(counter.state() + calculate_increment());

// ✅ صحيح: method منفصلة
impl CounterCubit {
    fn increment_by(&self, amount: i32) {
        self.core.update(|n| n + amount);
    }
}
```

### 2. استخدم update بدلاً من emit

```rust
// ❌ قد يسبب race condition
let current = counter.state();
counter.emit(current + 1);

// ✅ atomic update
counter.update(|n| n + 1);
```

### 3. الحالة الـ Immutable

```rust
// ❌ تعديل مباشر
let mut state = cubit.state();
state.items.push(item);
cubit.emit(state);

// ✅ نسخ ثم تعديل
cubit.update(|state| {
    let mut new_state = state.clone();
    new_state.items.push(item);
    new_state
});
```

### 4. استخدم Selectors للحالة المشتقة

```rust
// ❌ حساب في كل مكان
fn get_total(cart: &CartState) -> f64 {
    cart.items.iter().map(|i| i.price * i.qty as f64).sum()
}

// ✅ Selector مع caching
let total = Selector::new(&cart, |s| {
    s.items.iter().map(|i| i.price * i.qty as f64).sum()
});
```

### 5. الاشتراك في الـ Scope الصحيح

```rust
// ❌ subscription يُفقد فوراً
cubit.subscribe(|_, _| { /* ... */ });

// ✅ احفظ الـ handle
let _sub = cubit.subscribe(|_, _| { /* ... */ });

// أو في struct
struct Widget {
    subscriptions: Vec<SubscriptionHandle>,
}
```

### 6. اختر النوع المناسب

```rust
// بسيط ومتزامن -> CubitCore
let toggle = CubitCore::new(false);

// أحداث متعددة -> Bloc
let auth = BlocCore::new(AuthState::Initial, handler);

// Widget integration -> Sync
let ui_state = SyncCubitCore::new(UiState::default());

// حالة مشتقة -> Selector
let filtered = Selector::new(&items, |i| i.iter().filter(...).collect());
```

### 7. تجنب الحلقات اللانهائية

```rust
// ❌ subscription يُصدر مما يستدعي نفسه
cubit.subscribe(|_, new| {
    cubit.emit(*new + 1);  // حلقة لانهائية!
});

// ✅ استخدم شرط للخروج
cubit.subscribe(|old, new| {
    if *new < 10 {
        cubit.emit(*new + 1);
    }
});
```

---

## أمثلة عملية

### مثال 1: Counter App

```rust
use venom_core::{SyncCubitCore, DevTools, ConsoleObserver};
use venom_widgets::{SyncBlocBuilder, Row, Button, Text};

// تفعيل DevTools
DevTools::set_observer(ConsoleObserver::new());

// State
let counter = SyncCubitCore::named("counter", 0);

// UI
let app = Row::new()
    .child(Button::new("-")
        .on_tap({
            let c = counter.clone();
            move || { c.update(|n| n - 1); }
        }))
    .child(SyncBlocBuilder::new(counter.clone(), |state| {
        Box::new(Text::new(format!("{}", state)))
    }))
    .child(Button::new("+")
        .on_tap({
            let c = counter.clone();
            move || { c.update(|n| n + 1); }
        }));
```

### مثال 2: Todo App

```rust
use venom_core::{SyncCubitCore, Selector};

#[derive(Clone, PartialEq)]
struct TodoState {
    items: Vec<Todo>,
    filter: Filter,
}

#[derive(Clone, PartialEq)]
struct Todo { id: u32, text: String, done: bool }

#[derive(Clone, PartialEq)]
enum Filter { All, Active, Completed }

let todos = SyncCubitCore::new(TodoState {
    items: vec![],
    filter: Filter::All,
});

// Selectors
let filtered_todos = Selector::new(&todos, |s| {
    s.items.iter().filter(|t| match s.filter {
        Filter::All => true,
        Filter::Active => !t.done,
        Filter::Completed => t.done,
    }).cloned().collect::<Vec<_>>()
});

let remaining_count = Selector::new(&todos, |s| {
    s.items.iter().filter(|t| !t.done).count()
});

// Actions
fn add_todo(todos: &SyncCubitCore<TodoState>, text: &str) {
    todos.update(|s| {
        let mut new = s.clone();
        let id = new.items.len() as u32;
        new.items.push(Todo { id, text: text.to_string(), done: false });
        new
    });
}

fn toggle_todo(todos: &SyncCubitCore<TodoState>, id: u32) {
    todos.update(|s| {
        let mut new = s.clone();
        if let Some(t) = new.items.iter_mut().find(|t| t.id == id) {
            t.done = !t.done;
        }
        new
    });
}
```

### مثال 3: Auth Flow

```rust
use venom_core::{SyncBloc, SyncCubitCore};

#[derive(Clone)]
enum AuthEvent {
    Login { email: String, password: String },
    Logout,
    TokenRefresh,
}

#[derive(Clone, PartialEq)]
enum AuthState {
    Initial,
    Loading,
    Authenticated { user: User, token: String },
    Error(String),
}

let auth = SyncBloc::named("auth", AuthState::Initial, |event, state, emit| {
    match event {
        AuthEvent::Login { email, password } => {
            emit(AuthState::Loading);
            
            // API call (simplified)
            match api::login(&email, &password) {
                Ok((user, token)) => {
                    emit(AuthState::Authenticated { user, token });
                }
                Err(e) => {
                    emit(AuthState::Error(e.to_string()));
                }
            }
        }
        AuthEvent::Logout => {
            emit(AuthState::Initial);
        }
        AuthEvent::TokenRefresh => {
            if let AuthState::Authenticated { user, .. } = state {
                // Refresh logic
            }
        }
    }
});

// Usage
auth.add(AuthEvent::Login {
    email: "user@example.com".into(),
    password: "secret".into(),
});
```

---

## 📚 المراجع

- [venom-core API](file:///home/x/Desktop/VENOMIDE/venom-ui-rs/crates/venom-core/src/lib.rs)
- [venom-widgets API](file:///home/x/Desktop/VENOMIDE/venom-ui-rs/crates/venom-widgets/src/lib.rs)
- [Examples](file:///home/x/Desktop/VENOMIDE/venom-ui-rs/crates/venom-app/examples/)

---

> 💡 **نصيحة**: ابدأ بـ `CubitCore` للحالات البسيطة، وانتقل لـ `Bloc` عند الحاجة لإدارة أحداث معقدة.
