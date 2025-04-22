declare class VideoElement extends Element{
    constructor();
    setSrc(src: any): void;
    play(): void;
    pause(): void;
    stop(): void;
    /**
     *
     * @param time {number}
     */
    seek(time: number): void;
    /**
     *
     * @param callback {(e: IEvent<{duration: number, width: number, height: number}>) => void}
     */
    bindLoadedMetaData(callback: (e: IEvent<{
        duration: number;
        width: number;
        height: number;
    }>) => void): void;
    /**
     *
     * @param callback {(e: IEvent<void>) => void}
     */
    bindPlay(callback: (e: IEvent<void>) => void): void;
    bindPause(callback: any): void;
    /**
     *
     * @param callback {(e: IEvent<number>) => void}
     */
    bindProgress(callback: (e: IEvent<number>) => void): void;
    /**
     *
     * @param callback {() => void}
     */
    bindStop(callback: () => void): void;
}
