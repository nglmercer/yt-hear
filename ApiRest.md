
# YouTube Music Bridge API Documentation

This project consists of a **Rust** backend (using `Axum`) that serves as an HTTP control server and a **TypeScript** event emitter for the frontend. The server exposes endpoints to control the music player, manage the queue, and retrieve playback status.

## 1. Base URL
The server runs on the specified port (defaulting to dynamic assignment or configuration).
`http://localhost:<PORT>/api/v1`

## 2. HTTP Endpoints

### Playback Control
| Method | Endpoint | Payload (JSON) | Description | Internal Action |
| :--- | :--- | :--- | :--- | :--- |
| `POST` | `/play` | - | Resumes playback | `play` |
| `POST` | `/pause` | - | Pauses playback | `pause` |
| `POST` | `/toggle-play` | - | Toggles play/pause | `playPause` |
| `POST` | `/next` | - | Skips to next track | `next` |
| `POST` | `/previous` | - | Returns to previous track | `previous` |
| `POST` | `/shuffle` | - | Toggles shuffle mode | `toggleShuffle` |
| `POST` | `/repeat` | - | Toggles repeat mode | `toggleRepeat` |
| `POST` | `/like` | - | Likes the current track | `like` |
| `POST` | `/dislike` | - | Dislikes the current track | `dislike` |

### Volume & Seeking
| Method | Endpoint | Payload (JSON) | Description | Internal Action |
| :--- | :--- | :--- | :--- | :--- |
| `GET` | `/volume` | - | Gets current volume | `get-volume` (Request) |
| `POST` | `/volume` | [`VolumePayload`](#volumepayload) | Sets volume (0-100) | `setVolume` |
| `POST` | `/toggle-mute` | - | Toggles mute | `toggleMute` |
| `POST` | `/seek-to` | [`SeekPayload`](#seekpayload) | Seeks to absolute time (seconds) | `seek` |
| `POST` | `/go-back` | [`SeekPayload`](#seekpayload) | Rewinds by X seconds | `goBack` |
| `POST` | `/go-forward` | [`SeekPayload`](#seekpayload) | Fast forwards by X seconds | `goForward` |

### Queue Management
| Method | Endpoint | Payload (JSON) | Description | Internal Action |
| :--- | :--- | :--- | :--- | :--- |
| `GET` | `/queue` | - | Retrieves current queue | `get-queue` (Request) |
| `POST` | `/queue` | [`QueueAddPayload`](#queueaddpayload) | Adds video to queue | `addToQueue` |
| `PATCH`| `/queue` | [`QueueIndexPayload`](#queueindexpayload) | Jumps to specific queue index | `setQueueIndex` |
| `POST` | `/queue/index` | [`QueueIndexPayload`](#queueindexpayload) | Alias for PATCH /queue | `setQueueIndex` |
| `POST` | `/queue/move` | [`QueueMovePayload`](#queuemovepayload) | Moves an item within the queue | `moveInQueue` |
| `DELETE`| `/queue/:index`| - | Removes item at index | `removeFromQueue` |
| `POST` | `/clear-queue` | - | Clears the entire queue | `clearQueue` |

### Info & Search
| Method | Endpoint | Payload (JSON) | Description | Internal Action |
| :--- | :--- | :--- | :--- | :--- |
| `GET` | `/song` | - | Gets current song info | `get-song-info` (Request) |
| `POST` | `/search` | [`SearchPayload`](#searchpayload) | Performs a search | `search` |

---

## 3. Data Structures (Payloads)

These are the JSON structures required for the `POST` and `PATCH` requests.

### `SeekPayload`
Used for seeking, rewinding, and fast-forwarding.
```json
{
  "seconds": 30.5
}
```

### `VolumePayload`
Used for setting volume.
```json
{
  "volume": 50.0
}
```

### `QueueAddPayload`
Used for adding items to the queue.
*   `insertPosition`: Optional. Defaults to `"INSERT_AT_END"`.
```json
{
  "videoId": "dQw4w9WgXcQ",
  "insertPosition": "INSERT_NEXT" 
}
```

### `QueueIndexPayload`
Used for jumping to a specific track index.
```json
{
  "index": 2
}
```

### `QueueMovePayload`
Used for reordering the queue.
```json
{
  "fromIndex": 3,
  "toIndex": 1
}
```

### `SearchPayload`
Used for search queries.
```json
{
  "query": "Never Gonna Give You Up"
}
```

---

## 4. TypeScript `Emitter` Class

The `Emitter` class is a robust implementation of the Pub/Sub pattern, used to manage events within the frontend application.

### Key Methods

| Method | Description |
| :--- | :--- |
| `on(event, callback)` | Adds a listener for the specified event. Returns a cleanup function. |
| `once(event, callback)` | Adds a one-time listener. |
| `onAny(callback)` | Adds a listener that triggers on *any* event emitted. |
| `emit(event, data)` | Synchronously calls each of the listeners registered for the event. |
| `emitAsync(event, data)` | Asynchronously calls listeners (via `setTimeout`). |
| `off(event, callback)` | Removes a specific listener. |
| `debug()` | Returns memory usage and listener counts for debugging leaks. |

---