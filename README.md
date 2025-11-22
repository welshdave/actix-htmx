# actix-htmx

> Comprehensive middleware for building dynamic web applications with [htmx](https://htmx.org) and [Actix Web](https://github.com/actix/actix-web).

[![crates.io](https://img.shields.io/crates/v/actix-htmx?label=latest)](https://crates.io/crates/actix-htmx)
![Apache 2.0 or MIT licensed](https://img.shields.io/crates/l/actix-htmx)
[![Documentation](https://docs.rs/actix-htmx/badge.svg)](https://docs.rs/actix-htmx)

`actix-htmx` provides a comprehensive solution for building dynamic web applications with htmx and Actix Web. It offers type-safe access to htmx request headers, easy response manipulation, and powerful event triggering capabilities.

## Features

- **Request Detection**: Automatically detect htmx requests, boosted requests, and history restore requests
- **Header Access**: Type-safe access to all htmx request headers (current URL, target, trigger, prompt, etc.)
- **Event Triggering**: Trigger custom JavaScript events with optional data at different lifecycle stages
- **Response Control**: Full control over htmx behavior with response headers (redirect, refresh, swap, retarget, etc.)
- **Type Safety**: Fully typed API leveraging Rust's type system for correctness
- **Zero Configuration**: Works out of the box with sensible defaults
- **Performance**: Minimal overhead with efficient header processing

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
actix-htmx = "0.3"
actix-web = "4"
```

## Quick Start

1. **Register the middleware** on your Actix Web app:

```rust
use actix_htmx::HtmxMiddleware;
use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(HtmxMiddleware)  // Add this line
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

2. **Use the `Htmx` extractor** in your handlers:

```rust
use actix_htmx::{Htmx, HxLocation, SwapType};
use actix_web::{HttpResponse, Responder};
use serde_json::json;

async fn index(htmx: Htmx) -> impl Responder {
    if htmx.is_htmx {
        // This is an htmx request - return partial HTML
        HttpResponse::Ok().body("<div>Partial content for htmx</div>")
    } else {
        // Regular request - return full page
        HttpResponse::Ok().body(r##"
            <!DOCTYPE html>
            <html>
                <head>
                    <script src="https://unpkg.com/htmx.org@2.0.7"></script>
                </head>
                <body>
                    <div id="content">
                        <button hx-get="/" hx-target="#content">
                            Click me for htmx!
                        </button>
                    </div>
                </body>
            </html>
        "##)
    }
}
```

## Usage Examples

### Accessing Request Information

```rust
use actix_htmx::Htmx;
use actix_web::{HttpResponse, Responder};

async fn handler(htmx: Htmx) -> impl Responder {
    // Check if this is an htmx request
    if htmx.is_htmx {
        println!("This is an htmx request!");
        
        // Access htmx-specific information
        if let Some(target) = htmx.target() {
            println!("Target element: {}", target);
        }
        
        if let Some(trigger) = htmx.trigger() {
            println!("Triggered by element: {}", trigger);
        }
        
        if let Some(current_url) = htmx.current_url() {
            println!("Current page URL: {}", current_url);
        }
    }
    
    // Check for boosted requests
    if htmx.boosted {
        println!("This is a boosted request!");
    }
    
    // Check for history restore
    if htmx.history_restore_request {
        println!("This is a history restore request!");
    }
    
    HttpResponse::Ok().body("Hello, htmx!")
}
```

### Controlling Response Behaviour

```rust
use actix_htmx::{Htmx, SwapType, TriggerPayload, TriggerType};
use actix_web::{HttpResponse, Responder};
use serde_json::json;

async fn create_item(htmx: Htmx) -> impl Responder {
    // Trigger a custom JavaScript event
    let payload = TriggerPayload::json(json!({ "id": 123, "name": "New Item" })).unwrap();
    htmx.trigger_event(
        "itemCreated",
        Some(payload),
        Some(TriggerType::Standard)
    );
    
    // Change how content is swapped
    htmx.reswap(SwapType::OuterHtml);
    
    // Update the URL without navigation
    htmx.push_url("/items/123");
    
    // Redirect after successful creation
    htmx.redirect("/items");
    
    HttpResponse::Ok().body("<div>Item created!</div>")
}
```

### Event Triggering

htmx supports triggering custom events at different lifecycle stages:

```rust
use actix_htmx::{Htmx, TriggerPayload, TriggerType};
use actix_web::{HttpResponse, Responder};
use serde_json::json;

async fn handler(htmx: Htmx) -> impl Responder {
    // Trigger immediately when response is received
    htmx.trigger_event(
        "dataLoaded",
        None,
        Some(TriggerType::Standard)
    );
    
    // Trigger after content is swapped into DOM
    let swapped_payload = TriggerPayload::json(json!({ "timestamp": "2024-01-01" })).unwrap();
    htmx.trigger_event(
        "contentSwapped",
        Some(swapped_payload),
        Some(TriggerType::AfterSwap)
    );
    
    // Trigger after htmx has settled (animations complete, etc.)
    htmx.trigger_event(
        "pageReady",
        None,
        Some(TriggerType::AfterSettle)
    );
    
    HttpResponse::Ok().body("<div>Content updated!</div>")
}
```

### Advanced Response Control

```rust
use actix_htmx::Htmx;
use actix_web::{HttpResponse, Responder};

async fn advanced_handler(htmx: Htmx) -> impl Responder {
    // Change the target element for this response
    htmx.retarget("#different-element");
    
    // Select specific content from response
    htmx.reselect(".important-content");
    
    // Replace URL in browser history (no new history entry)
    htmx.replace_url("/new-path");
    
    // Refresh the entire page
    htmx.refresh();
    
    // Redirect using htmx (no full page reload) with a custom HX-Location payload
    let location = HxLocation::new("/dashboard")
        .target("#content")
        .swap(SwapType::OuterHtml)
        .values(json!({ "message": "Welcome back!" }))
        .expect("static payload should serialize");
    htmx.redirect_with_location(location);
    
    HttpResponse::Ok().body(r#"
        <div class="important-content">
            This content will be selected and swapped!
        </div>
        <div class="other-content">
            This won't be swapped due to reselect.
        </div>
    "#)
}
```

## Swap Types

The `SwapType` enum provides all htmx swap strategies:

- `InnerHtml` - Replace inner HTML (default)
- `OuterHtml` - Replace entire element
- `BeforeBegin` - Insert before element
- `AfterBegin` - Insert at start of element
- `BeforeEnd` - Insert at end of element  
- `AfterEnd` - Insert after element
- `Delete` - Delete the element
- `None` - Don't swap content

## Trigger Types

Events can be triggered at different points:

- `Standard` - Trigger immediately when response received
- `AfterSwap` - Trigger after content swapped into DOM
- `AfterSettle` - Trigger after htmx settling (animations, etc.)

## Complete Example

Check out the [todo example](examples/todo/) for a complete working application that demonstrates:

- Setting up the middleware
- Handling both htmx and regular requests
- Using response headers for dynamic behaviour
- Event triggering

## Documentation

For detailed API documentation, visit [docs.rs/actix-htmx](https://docs.rs/actix-htmx).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
