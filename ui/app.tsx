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

    async function onKeyDown(e) {
        const {keyStr, modifiers} = e.detail;
        // console.log("onKeyDown", e);
        if (keyStr === "s" && modifiers === 1) {

        }
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

    return <Container style={{flex: 1, color: "#F1F1F1"}} onKeyDown={onKeyDown}>
        <Container style={{flex: 1, background: '#000', position: 'relative'}}>
            <Container ref={wrapper} style={{width: '100%', height: '100%'}}></Container>
            <Container style={{
                position: 'absolute',
                width: '100%',
                height: '100%',
                justifyContent: 'center',
                alignItems: 'center',
                display: videoPath ? 'none' : 'flex',
            }}>
                <Button onClick={onOpen} style={{
                    padding: '10 20',
                    borderRadius: 8,
                    border: '#444 1'
                }}>打开文件</Button>
            </Container>
        </Container>
        <Progress value={progressPercent} onChange={onSeek} />
        <Row style={{justifyContent: 'space-between', alignItems: 'center', padding: '10', background: '#3C3F41'}}>
            <Label text={formatHumanShortTime(currentTime)} />
            <Row>
                <Image
                    src={statusIcon}
                    style={{
                        width: 36,
                        height: 36,
                    }}
                    cursor='pointer'
                    onClick={togglePlayStatus}
                />
            </Row>
            <Label text={formatHumanShortTime(duration)} />
        </Row>
    </Container>
}

export default App
