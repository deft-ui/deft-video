import "./app.css"
import React, {useContext, useEffect, useRef, useState} from 'react'
import {formatHumanShortTime} from "./util/time-util";
import Progress from "./progress";
import {Button, Container, Image, Label, PageContext, Row} from "deft-react";

const App = () => {

    const videoRef = useRef<VideoElement>(null);
    const [duration, setDuration] = useState(0);
    const [currentTime, setCurrentTime] = useState(0);
    const [videoPath, setVideoPath] = useState(process.argv[1] || "");
    const [playing, setPlaying] = useState(false);
    const pageContext = useContext(PageContext);
    const [fullscreen, setFullscreen] = useState(false);
    const [mouseMoving, setMouseMoving] = useState(false);
    const mouseMovingTimer = useRef<any>();

    async function onKeyDown(e) {
        const {keyStr, modifiers} = e.detail;
        // console.log("onKeyDown", e);
        if (keyStr === "s" && modifiers === 1) {

        }
    }

    function onMouseMove() {
        if (mouseMovingTimer.current) {
            clearTimeout(mouseMovingTimer.current);
        }
        setMouseMoving(true);
        mouseMovingTimer.current = setTimeout(() => {
            setMouseMoving(false);
            mouseMovingTimer.current = null;
        }, 1000);
    }

    function onFullscreen() {
        setFullscreen((fullscreen) => {
            if (fullscreen) {
                pageContext.window.exitFullscreen();
            } else {
                pageContext.window.requestFullscreen();
            }
            return !fullscreen;
        });
    }

    const wrapper = useRef<ContainerElement>();
    useEffect(() => {
        if (!videoPath) {
            return;
        }
        const video = videoRef.current = new VideoElement();
        let playing = false;
        function updatePlaying(value) {
            playing = value;
            setPlaying(value);
        }
        video.style = {
            width: '100%',
            height: '100%',
        };
        console.log('video', video);
        video.bindClick(e => {
            if (playing) {
                video.pause();
            } else {
                video.play();
            }
        })
        video.bindLoadedMetaData(e => {
            console.log('loaded', e.target == video);
            setDuration(e.detail.duration);
            video.play();
        })
        video.bindPlay(() => {
            console.log('playing');
            updatePlaying(true);
        })
        video.bindPause(() => {
            updatePlaying(false);
        })
        video.bindStop(() => {
            console.log("stopped")
            updatePlaying(false);
        })
        video.bindProgress(e => {
            setCurrentTime(e.detail);
        });
        wrapper.current.addChild(video);
        video.setSrc(videoPath);
        return () => {
            // videoRef.current = null;
            wrapper.current.removeChild(video);
        }
    }, [videoPath]);

    const progressPercent = (currentTime / duration) * 100;

    function onSeek(value: number) {
        const time = value / 100 * duration;
        console.log("seek", time);
        videoRef.current?.seek(time);
    }

    const playIcon = require('./assets/play.svg');
    const pauseIcon = require('./assets/pause.svg');
    const prevIcon = require("./assets/prev.svg");
    const nextIcon = require("./assets/next.svg");
    const stopIcon = require("./assets/stop.svg");
    const openIcon = require("./assets/open.svg");
    const fullscreenIcon = require("./assets/fullscreen.svg");


    const statusIcon = playing ? pauseIcon : playIcon;

    function togglePlayStatus() {
        if (playing) {
            videoRef.current?.pause();
        } else {
            videoRef.current?.play();
        }
    }

    async function onOpen() {
        //@ts-ignore
        const filePaths = await fileDialog.show({
            dialogType: 'single',
            window: pageContext.window,
        });
        console.log({filePaths});
        setVideoPath(filePaths[0]);
    }

    return <Container className="main" onKeyDown={onKeyDown} onMouseMove={onMouseMove}>
        <Container style={{flex: 1, position: 'relative'}}>
            <Container ref={wrapper} style={{width: '100%', height: '100%'}}></Container>
            <Container style={{
                position: 'absolute',
                width: '100%',
                height: '100%',
                justifyContent: 'center',
                alignItems: 'center',
                display: videoPath ? 'none' : 'flex',
            }}>
                <Button onClick={onOpen} className="open-btn">
                    <Image src={openIcon} className="open-icon" />
                    打开文件
                </Button>
            </Container>
        </Container>
        <Container className={`main-panel ${fullscreen ? (mouseMoving ? 'main-panel-fullscreen-moving' : 'main-panel-fullscreen') : ''}`}>
            <Progress value={progressPercent} onChange={onSeek} />
            <Row className="control-panel">
                <Container className="control-left-side-panel">
                    {formatHumanShortTime(currentTime) + ' / ' + formatHumanShortTime(duration)}
                </Container>
                <Row className="play-btn-group" style={{flex: 1}}>
                    {/*<Image src={stopIcon} className="stop-btn" cursor='pointer' />*/}
                    <Image src={prevIcon} className="default-btn" cursor='pointer' />
                    <Image src={statusIcon} className="primary-btn" cursor='pointer' onClick={togglePlayStatus} />
                    <Image src={nextIcon} className="default-btn" cursor='pointer' />
                </Row>
                {/*<Label text={formatHumanShortTime(duration)} />*/}
                <Row className="control-right-side-panel">
                    <Image onClick={onFullscreen} src={fullscreenIcon} className="tool-btn" cursor="pointer" />
                </Row>
            </Row>
        </Container>
    </Container>
}

export default App
