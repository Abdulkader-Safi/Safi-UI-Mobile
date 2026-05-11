# 17 — Image pipeline (background decode + channel-based main-thread upload)

**Phase:** 4 — Component Library
**PRD refs:** §8.4, §8.5, §13

## Goal

Decode images on a background thread pool, signal completion via channel, upload + cache on the main thread.

## Deliverables

- `safi-ui::image::{ImageLoader, ImageCache, DecodedImage}` per the §8.5 frame-loop sketch
- Background decoding via `image` crate on a worker pool
- `crossbeam::channel` carries `DecodedImage { owner_id, src, pixels }` to the main thread
- LRU cache keyed by `src` string; max size configurable
- `SDL_EVENT_LOW_MEMORY` → full cache eviction
- Image component shows `Spinner` while loading, `EmptyState` on error
- Network images (`https://`) stubbed with a clear "v1.1" warning

## Dependencies

- `09`, `12`

## Acceptance

- Decoding a 4K PNG never blocks the main thread
- `mark_dirty(owner_id)` only invalidates the requesting widget
- Low-memory event purges cache without crashing in-flight uploads
