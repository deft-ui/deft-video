//@ts-ignore
export class VideoElement extends Element {

    constructor() {
        // @ts-ignore
        super("video");
    }

    setSrc(src) {
        VideoBackend_set_src(this.handle, src);
    }

    play() {
        VideoBackend_play(this.handle)
    }

    pause() {
        VideoBackend_pause(this.handle)
    }

    stop() {
        VideoBackend_stop(this.handle);
    }

    /**
     *
     * @param time {number}
     */
    seek(time) {
        VideoBackend_seek(this.handle, time);
    }

    /**
     *
     * @param callback {(e: IEvent<{duration: number, width: number, height: number}>) => void}
     */
    bindLoadedMetaData(callback) {
        this.bindEvent("loadedmetadata", callback);
    }

    /**
     *
     * @param callback {(e: IEvent<void>) => void}
     */
    bindPlay(callback) {
        this.bindEvent("play", callback);
    }

    bindPause(callback) {
        this.bindEvent("pause", callback);
    }

    /**
     *
     * @param callback {(e: IEvent<number>) => void}
     */
    bindProgress(callback) {
        this.bindEvent("progress", callback);
    }

    /**
     *
     * @param callback {() => void}
     */
    bindStop(callback) {
        this.bindEvent("stop", callback);
    }

}

globalThis.VideoElement = VideoElement;