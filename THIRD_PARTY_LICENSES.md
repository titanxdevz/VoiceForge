# Third-Party Dependency Audit & Licenses

All dependencies used in **VoiceForge** have been audited for permissive open-source licenses, active maintenance, real-time safety, and minimal latency impact.

| Crate | Version | License | Maintenance / Status | Real-Time / Latency Impact | Purpose |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **cpal** | `0.18.2` | Apache-2.0 | Actively maintained by RustAudio | Low latency WASAPI backend for direct hardware buffer streaming. | Audio device I/O |
| **eframe** / **egui** | `0.31.1` | MIT OR Apache-2.0 | Actively maintained by Emil Ernerfeldt | Zero GUI allocations in audio thread. Visible repaints capped at 60 FPS, 0 FPS when minimized. | Immediate mode desktop UI |
| **rtrb** | `0.4.0` | MIT OR Apache-2.0 | Maintained lock-free SPSC queue | Wait-free, zero-allocation ringbuffer for inter-thread audio sample queues. | Lock-free thread comms |
| **crossbeam-channel** | `0.5.17` | MIT OR Apache-2.0 | Industry standard | Bounded channel for control event messaging without blocking the UI. | Command & Telemetry queue |
| **rodio** | `0.23.0` | MIT OR Apache-2.0 | Maintained by RustAudio | Pure Rust audio file decoding (WAV, MP3, OGG, FLAC) off the audio thread. | Soundboard clip loader |
| **serde** / **serde_json** | `1.0.x` | MIT OR Apache-2.0 | Industry standard | Used strictly during user preset save/load and diagnostic export. | Data serialization |
| **toml** | `1.1.x` | MIT OR Apache-2.0 | Industry standard | Human-readable preset file formatting. | Preset file format |
| **tray-icon** / **muda** | `0.26` / `0.21`| Apache-2.0 / MIT | Maintained by Tauri team | Lightweight Windows system tray icon and context menu. | Background tray management |
| **windows-sys** | `0.52`/`0.61` | MIT OR Apache-2.0 | Official Microsoft bindings | Direct Win32 API calls for MMCSS priority, Mutex, and CurrentUser registry autostart. | Windows system integration |
| **dirs** | `7.0.0` | MIT OR Apache-2.0 | Standard directory resolution | Resolves `%APPDATA%` for atomic preset storage without elevated permissions. | Path management |

---

## License Texts Summary

### Apache License, Version 2.0
Licensed under the Apache License, Version 2.0 (the "License"); you may not use this software except in compliance with the License. You may obtain a copy of the License at:
http://www.apache.org/licenses/LICENSE-2.0

### MIT License
Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software.
