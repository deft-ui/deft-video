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

# Demo snapshot

<img src="https://github.com/deft-ui/deft-video/blob/main/snapshot.png?raw=true" width="450" />

# License

GPL



