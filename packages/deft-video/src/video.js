//@ts-ignore
export class VideoElement extends Element {

    #backend;

    #eventBinder;

    constructor() {
        // @ts-ignore
        const [el, backend] = VideoBackend_new();
        super(el);
        this.#backend = backend;
        this.#eventBinder = this.createEventBinder(backend, VideoBackend_bind_js_event_listener);
    }

    setSrc(src) {
        VideoBackend_set_src(this.#backend, src);
    }

    play() {
        VideoBackend_play(this.#backend)
    }

    pause() {
        VideoBackend_pause(this.#backend)
    }

    stop() {
        VideoBackend_stop(this.#backend);
    }

    /**
     *
     * @param time {number}
     */
    seek(time) {
        VideoBackend_seek(this.#backend, time);
    }

    /**
     *
     * @param callback {(e: IEvent<{duration: number, width: number, height: number}>) => void}
     */
    bindLoadedMetaData(callback) {
        this.#eventBinder.bindEvent("loadedmetadata", callback);
    }

    /**
     *
     * @param callback {(e: IEvent<void>) => void}
     */
    bindPlay(callback) {
        this.#eventBinder.bindEvent('play', callback);
    }

    bindPause(callback) {
        this.#eventBinder.bindEvent('pause', callback);
    }

    /**
     *
     * @param callback {(e: IEvent<number>) => void}
     */
    bindProgress(callback) {
        this.#eventBinder.bindEvent('progress', callback);
    }

    /**
     *
     * @param callback {() => void}
     */
    bindStop(callback) {
        this.#eventBinder.bindEvent('stop', callback);
    }

}

globalThis.VideoElement = VideoElement;