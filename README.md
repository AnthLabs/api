# api

A clean Rust API project generated with [`oxgen`](https://github.com/OxgeneratorLabs/oxgenerator).

## Getting started

Run the project:

```bash
cargo run
```

Run tests:

```bash
cargo test
```

Format the code:

```bash
cargo fmt
```

Lint the code:

```bash
cargo clippy
```

Run mongodb
```bash
docker compose -f docker-compose.yml up -d --build
```

Run project
```bash
docker compose -f docker-compose.prod.yml up -d --build
```

## Websocket message
change video url:
```bash
{
  "type": "change_video",
  "video_url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
}
```

play:
```bash
{
  "type": "play",
  "position_seconds": 0.0
}
```

pause:
```bash
{
  "type": "pause",
  "position_seconds": 15.8
}
```

seek:
```bash
{
  "type": "seek",
  "position_seconds": 120.0
}
```
