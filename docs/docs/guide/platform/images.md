# Image Loading

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned image pipeline.
:::

## Pipeline

| Stage            | Detail                                                                  |
| ---------------- | ----------------------------------------------------------------------- |
| Formats          | PNG, JPEG, WebP via the `image` Rust crate                              |
| Decoding         | Async, decoded on a background thread pool                              |
| Main thread sync | Decode completion signalled via channel; GPU upload on main thread only |
| Cache            | LRU texture cache keyed by `src` string; configurable max size          |
| Placeholder      | Shows `<Spinner>` while loading; `<EmptyState>` on error                |
| Network images   | `src` starting with `https://` triggers async HTTP fetch (v1.1)         |
| Eviction         | `SDL_EVENT_LOW_MEMORY` triggers full cache eviction                     |
| Asset path       | Relative paths resolved from `assets/images/` via `AssetLoader`         |

## Channel-based decode signalling

Background image decode signals the main thread via a `crossbeam::channel`. The main loop drains the channel once per frame:

```rust
struct DecodedImage {
    owner_id: WidgetId,         // widget that requested the image
    src:      String,            // cache key
    pixels:   image::RgbaImage,  // decoded pixels, ready for upload
}

while let Ok(decoded) = image_channel.try_recv() {
    let texture = gpu.upload_texture(&decoded.pixels);
    image_cache.insert(decoded.src, texture);
    ctx.dirty.mark_dirty(decoded.owner_id);
}
```

## Usage

```xml
<Image src="logo.png" width="120" height="120" radius="12" fit="cover" />
<Image src="{{user.avatar}}" width="48" height="48" radius="24" />
```

Resolved from `assets/images/`.

See [`<Image>`](/api/components/display).
