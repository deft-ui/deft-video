# Deft Video Component

Video component for [Deft](https://github.com/deft-ui/deft) with [FFmpeg](https://ffmpeg.org/) backend


# Usage

**Cargo.toml**
```
[dependencies]
deft-video = 0.1.0
```

**Initialize**

```rust
use deft_video::deft_video_init;
impl IApp for YourApp {
    fn init_js_engine(&mut self, js_engine: &mut JsEngine) {
        ...
        deft_video_init(js_engine);
    }
}
```

**Use in js**

```javascript
const video = new VideoElement();
video.style = {
    width: '100%',
    height: '100%',
};
video.bindLoadedMetaData(e => {
    console.log('loaded', e.detail.duration);
    video.play();
})
video.bindPlay(() => {
    console.log('playing');
})
video.bindPause(() => {
    console.log('paused');
})
video.bindStop(() => {
    console.log("stopped")
})
video.bindProgress(e => {
    console.log('current time', e.detail);
});
someContainer.addChild(video);
video.setSrc(videoPath);
```

**TypeScript declaration**

See `packages/deft-video/ts/video.d.ts`

# How to build

## Dependencies

Make sure that `Rust` and `Node.js` are installed.

On *nix systems, `clang`, `pkg-config` and FFmpeg libraries (including development headers) are required.

On macOS:

    brew install pkg-config ffmpeg

On Debian-based systems:

    apt install -y clang libavcodec-dev libavformat-dev libavutil-dev libswscale-dev libavfilter-dev libavdevice-dev  pkg-config

On Windows

MSVC toolchain is required

- Install LLVM (through official installer, Visual Studio, Chocolatey, or any other means), and add LLVM's `bin` path to `PATH`, or set `LIBCLANG_PATH` to that (see [`clang-sys` documentation](https://github.com/KyleMayes/clang-sys#environment-variables) for additional info).
- Install FFmpeg (complete with headers) through any means, e.g. downloading a pre-built "full_build-shared" version from https://ffmpeg.org/download.html. Set `FFMPEG_DIR` to the directory containing `include` and `lib`.
- `cargo build`.
- Add FFmpeg's `bin` path to `PATH`

## Develop

```
npm run dev
```

## Build

```
npm run build
```

# Snapshot

<img src="https://github.com/deft-ui/deft-video/blob/main/snapshot.png?raw=true" width="450" />

# License

GPL



